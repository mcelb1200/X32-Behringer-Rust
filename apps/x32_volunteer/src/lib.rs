pub mod config;
pub mod network;
pub mod state;
pub mod ui;

use anyhow::Result;
use clap::Parser;
use config::{AppConfig, load_config};
use network::ChannelState;
use osc_lib::OscArg;
use state::{AppState, Status};
use std::sync::Arc;
use std::time::{Duration, Instant};
use ui::{Tui, UIEvent};
use x32_lib::transport::{MixerTransport, udp::UdpTransport};
use x32_lib::{MixerClient, MixerModel};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Simplified TUI Dashboard (Volunteer Mode)")]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: String,

    /// Comma-separated list of channel/DCA numbers to monitor (e.g. "ch1,ch2,dca1").
    /// If empty, defaults to visible_channels/visible_dcas from config or ch1..ch8.
    #[arg(short, long, default_value = "")]
    pub channels: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short, long, default_value = "X32")]
    pub model: MixerModel,

    /// Path to configuration JSON file
    #[arg(short, long)]
    pub config: Option<String>,
}

/// Converts a linear fader level (0.0 to 1.0) to decibels on the X32 scale.
pub fn fader_to_db(level: f32) -> f32 {
    if level >= 0.5 {
        40.0 * level - 30.0
    } else if level >= 0.25 {
        80.0 * level - 50.0
    } else if level >= 0.0625 {
        160.0 * level - 70.0
    } else {
        480.0 * level - 90.0
    }
}

/// Returns (max_channels, max_dcas) for a given MixerModel.
pub fn model_limits(model: MixerModel) -> (u32, u32) {
    match model {
        MixerModel::X32 => (32, 8),
        MixerModel::Wing => (40, 16),
        MixerModel::XR18 => (16, 4),
        MixerModel::XR16 => (16, 4),
        MixerModel::XR12 => (12, 4),
    }
}

/// Resolves channel configuration from CLI args, config file, or defaults for the specified model.
pub fn resolve_channels(
    channels_arg: &str,
    config: Option<&AppConfig>,
    model: MixerModel,
) -> Vec<ChannelState> {
    let (max_ch, max_dca) = model_limits(model);
    let mut channels = Vec::new();

    if !channels_arg.is_empty() {
        for part in channels_arg.split(',') {
            let part = part.trim().to_lowercase();
            if let Some(stripped) = part.strip_prefix("ch") {
                if let Ok(num) = stripped.parse::<u32>() {
                    if num >= 1 && num <= max_ch {
                        channels.push(ChannelState::new(format!("/ch/{:02}", num), false, num));
                    }
                }
            } else if let Some(stripped) = part.strip_prefix("dca") {
                if let Ok(num) = stripped.parse::<u32>() {
                    if num >= 1 && num <= max_dca {
                        channels.push(ChannelState::new(format!("/dca/{}", num), true, num));
                    }
                }
            }
        }
    } else if let Some(cfg) = config.and_then(|c| c.volunteer_mode.as_ref()) {
        if let Some(v_ch) = &cfg.visible_channels {
            for &num in v_ch {
                if num >= 1 && num <= max_ch {
                    channels.push(ChannelState::new(format!("/ch/{:02}", num), false, num));
                }
            }
        }
        if let Some(v_dca) = &cfg.visible_dcas {
            for &num in v_dca {
                if num >= 1 && num <= max_dca {
                    channels.push(ChannelState::new(format!("/dca/{}", num), true, num));
                }
            }
        }
    }

    if channels.is_empty() {
        let count = std::cmp::min(8, max_ch);
        for i in 1..=count {
            channels.push(ChannelState::new(format!("/ch/{:02}", i), false, i));
        }
    }

    channels
}

pub async fn run(args: Args) -> Result<()> {
    // 1. Load Configuration if provided or default exists
    let config = if let Some(cfg_path) = &args.config {
        load_config(cfg_path).ok()
    } else if std::path::Path::new("x32_config.json").exists() {
        load_config("x32_config.json").ok()
    } else {
        None
    };

    let fader_limit = config
        .as_ref()
        .and_then(|c| c.volunteer_mode.as_ref())
        .and_then(|v| v.max_fader_limit_db);

    // 2. Setup Network
    let udp = UdpTransport::connect(&args.ip).await?;
    let transport: Arc<dyn MixerTransport> = Arc::new(udp);
    let network = MixerClient::new(transport, true);
    let mut rx = network.subscribe();

    // 3. Parse Channel Configuration
    let configured_channels = resolve_channels(&args.channels, config.as_ref(), args.model);

    if configured_channels.is_empty() {
        anyhow::bail!("No valid channels specified");
    }

    let state = AppState::new(configured_channels).with_fader_limit(fader_limit);
    let mut tui = Tui::new()?;

    // Wrap the main execution in a function returning Result so we can easily cleanup on error
    let result = run_tui_loop(&network, &mut rx, state, &mut tui).await;

    // Always cleanup the TUI
    let _ = tui.cleanup();

    result
}

async fn run_tui_loop(
    network: &MixerClient,
    rx: &mut tokio::sync::broadcast::Receiver<osc_lib::OscMessage>,
    mut state: AppState,
    tui: &mut Tui,
) -> Result<()> {
    // Initial state request
    for ch in &state.channels {
        network.send_message(ch.fader_path.as_str(), vec![]).await?;
        network.send_message(ch.mute_path.as_str(), vec![]).await?;
        network.send_message(ch.name_path.as_str(), vec![]).await?;
    }
    network
        .send_message("/meters", vec![OscArg::String("/meters/1".to_string())])
        .await?;

    let mut last_ui_update = Instant::now();
    let mut last_meter_req = Instant::now();

    loop {
        // Handle incoming OSC
        while let Ok(msg) = rx.try_recv() {
            let path = msg.path;

            // Fader / Mute updates
            for ch in &mut state.channels {
                if path == ch.fader_path {
                    if let Some(OscArg::Float(v)) = msg.args.first() {
                        ch.fader = *v;
                    }
                } else if path == ch.mute_path {
                    if let Some(OscArg::Int(v)) = msg.args.first() {
                        ch.muted = *v == 0;
                    }
                } else if path == ch.name_path {
                    if let Some(OscArg::String(s)) = msg.args.first() {
                        if !s.is_empty() {
                            ch.name = s.clone();
                        }
                    }
                }
            }

            // Metering updates using actual blob data from /meters/1
            if path == "/meters/1" {
                if let Some(OscArg::Blob(data)) = msg.args.first() {
                    for ch in &mut state.channels {
                        if !ch.is_dca {
                            let idx = (ch.num - 1) as usize;
                            let start = 4 + idx * 4;
                            if start + 4 <= data.len() {
                                let bytes: [u8; 4] =
                                    data[start..start + 4].try_into().unwrap_or([0; 4]);
                                let float_val = f32::from_le_bytes(bytes);
                                let mut db = -144.0;
                                if float_val > 0.000001 {
                                    db = 20.0 * float_val.log10();
                                }
                                ch.level_db = db;
                            }
                        }
                    }
                }
            }
        }

        // Request meters periodically
        if last_meter_req.elapsed() > Duration::from_millis(50) {
            network
                .send_message("/meters", vec![OscArg::String("/meters/1".to_string())])
                .await?;
            last_meter_req = Instant::now();
        }

        // Draw UI
        if last_ui_update.elapsed() > Duration::from_millis(30) {
            // Update alerts based on actual metered level and fader limits
            state.alerts.clear();
            state.fader_alerts.clear();

            for (i, ch) in state.channels.iter().enumerate() {
                if !ch.muted && ch.level_db > -5.0 && !ch.is_dca {
                    state.alerts.push(i);
                }
                if let Some(limit_db) = state.max_fader_limit_db {
                    let current_fader_db = fader_to_db(ch.fader);
                    if current_fader_db > limit_db {
                        state.fader_alerts.push(i);
                    }
                }
            }

            if !state.fader_alerts.is_empty() {
                state.status = Status::Problem;
            } else if !state.alerts.is_empty() {
                state.status = Status::Caution;
            } else {
                state.status = Status::Ok;
            }

            tui.draw(&state)?;

            if let Some(event) = tui.handle_events()? {
                match event {
                    UIEvent::Quit => break,
                    UIEvent::MuteAll => {
                        for ch in &state.channels {
                            network
                                .send_message(&ch.mute_path, vec![OscArg::Int(0)])
                                .await?;
                        }
                    }
                    UIEvent::Panic => {
                        for ch in &state.channels {
                            network
                                .send_message(&ch.mute_path, vec![OscArg::Int(0)])
                                .await?;
                            network
                                .send_message(&ch.fader_path, vec![OscArg::Float(0.0)])
                                .await?;
                        }
                    }
                }
            }

            last_ui_update = Instant::now();
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::net::UdpSocket;

    #[test]
    fn test_load_config() {
        let json_data = r#"{
            "volunteer_mode": {
                "max_fader_limit_db": -3.0,
                "visible_channels": [1, 2, 3],
                "visible_dcas": [1]
            }
        }"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "{}", json_data).expect("Failed to write temp file");

        let cfg = load_config(temp_file.path()).expect("Failed to load config");
        let v_cfg = cfg.volunteer_mode.expect("Missing volunteer_mode");
        assert_eq!(v_cfg.max_fader_limit_db, Some(-3.0));
        assert_eq!(v_cfg.visible_channels, Some(vec![1, 2, 3]));
        assert_eq!(v_cfg.visible_dcas, Some(vec![1]));
    }

    #[test]
    fn test_model_channel_bounds() {
        // XR12: max 12 channels, 4 DCAs
        let chs_xr12 = resolve_channels("ch1,ch12,ch13,dca4,dca5", None, MixerModel::XR12);
        assert_eq!(chs_xr12.len(), 3); // ch1, ch12, dca4
        assert_eq!(chs_xr12[0].num, 1);
        assert_eq!(chs_xr12[1].num, 12);
        assert_eq!(chs_xr12[2].num, 4);
        assert!(chs_xr12[2].is_dca);

        // Wing: max 40 channels, 16 DCAs
        let chs_wing = resolve_channels("ch40,dca16", None, MixerModel::Wing);
        assert_eq!(chs_wing.len(), 2);
        assert_eq!(chs_wing[0].num, 40);
        assert_eq!(chs_wing[1].num, 16);

        // Config resolution fallback
        let cfg = AppConfig {
            volunteer_mode: Some(config::VolunteerModeConfig {
                max_fader_limit_db: Some(-5.0),
                visible_channels: Some(vec![1, 15, 20]),
                visible_dcas: Some(vec![1, 2]),
            }),
        };

        // XR18: max 16 channels, 4 DCAs. So channel 20 should be ignored.
        let chs_cfg_xr18 = resolve_channels("", Some(&cfg), MixerModel::XR18);
        assert_eq!(chs_cfg_xr18.len(), 4); // ch1, ch15, dca1, dca2
    }

    #[test]
    fn test_max_fader_limit_alert() {
        let ch = ChannelState::new("/ch/01".to_string(), false, 1);
        let mut state = AppState::new(vec![ch]).with_fader_limit(Some(-3.0));

        // 0.75 fader float = 0.0 dB on X32 curve
        state.channels[0].fader = 0.75;
        assert!(fader_to_db(state.channels[0].fader) > -3.0);

        // Simulate alert evaluation
        state.fader_alerts.clear();
        for (i, ch) in state.channels.iter().enumerate() {
            if let Some(limit) = state.max_fader_limit_db {
                if fader_to_db(ch.fader) > limit {
                    state.fader_alerts.push(i);
                }
            }
        }
        assert_eq!(state.fader_alerts.len(), 1);
        assert_eq!(state.fader_alerts[0], 0);

        // Safe fader level (0.25 fader float = -30.0 dB)
        state.channels[0].fader = 0.25;
        state.fader_alerts.clear();
        for (i, ch) in state.channels.iter().enumerate() {
            if let Some(limit) = state.max_fader_limit_db {
                if fader_to_db(ch.fader) > limit {
                    state.fader_alerts.push(i);
                }
            }
        }
        assert!(state.fader_alerts.is_empty());
    }

    #[tokio::test]
    async fn test_args_parsing() {
        let args = Args {
            ip: "127.0.0.1".to_string(),
            channels: "ch1,dca2".to_string(),
            model: MixerModel::XR18,
            config: Some("x32_config.json".to_string()),
        };

        assert_eq!(args.ip, "127.0.0.1");
        assert_eq!(args.channels, "ch1,dca2");
        assert_eq!(args.model, MixerModel::XR18);
        assert_eq!(args.config, Some("x32_config.json".to_string()));
    }

    #[tokio::test]
    async fn test_mock_connection() -> Result<()> {
        let server = UdpSocket::bind("127.0.0.1:0").await?;
        let addr = server.local_addr()?;

        let transport = x32_lib::transport::udp::UdpTransport::connect(&addr.to_string()).await?;
        let _network = MixerClient::new(Arc::new(transport), false);

        Ok(())
    }
}
