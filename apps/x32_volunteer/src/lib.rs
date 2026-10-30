pub mod network;
pub mod state;
pub mod ui;

use anyhow::Result;
use clap::Parser;
use network::ChannelState;
use osc_lib::OscArg;
use serde::Deserialize;
use state::AppState;
use std::sync::Arc;
use std::time::{Duration, Instant};
use ui::{Tui, UIEvent};
use x32_lib::{MixerClient, MixerModel};
use x32_lib::transport::{MixerTransport, udp::UdpTransport};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Simplified TUI Dashboard (Volunteer Mode)")]
pub struct Args {
    /// IP address or hostname of the digital mixer
    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short = 'm', long, default_value = "X32")]
    pub model: MixerModel,

    /// Comma-separated list of channel/DCA numbers to monitor (e.g. "ch1,ch2,dca1"). If empty, defaults to ch1..ch8.
    #[arg(short, long, default_value = "")]
    pub channels: String,

    /// Path to JSON configuration file (e.g. x32_config.json)
    #[arg(short = 'c', long)]
    pub config: Option<String>,

    /// Maximum fader limit threshold in dBFS (e.g. -3.0)
    #[arg(long)]
    pub max_fader_limit_db: Option<f32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct VolunteerConfigSection {
    pub max_fader_limit_db: Option<f32>,
    pub visible_channels: Option<Vec<u32>>,
    pub visible_dcas: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    pub volunteer_mode: Option<VolunteerConfigSection>,
}

/// Returns the maximum number of input channels supported by the given mixer model.
pub fn max_channels_for_model(model: MixerModel) -> u32 {
    match model {
        MixerModel::Wing => 40,
        MixerModel::X32 => 32,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

/// Returns the maximum number of DCAs supported by the given mixer model.
pub fn max_dcas_for_model(model: MixerModel) -> u32 {
    match model {
        MixerModel::Wing => 16,
        MixerModel::X32 => 8,
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
    }
}

/// Resolves default target OSC port based on mixer model when port is omitted.
pub fn resolve_target_address(ip: &str, model: MixerModel) -> String {
    if ip.contains(':') {
        ip.to_string()
    } else {
        let port = match model {
            MixerModel::X32 => 10023,
            MixerModel::Wing => 2222,
            MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 10024,
        };
        format!("{}:{}", ip, port)
    }
}

/// Converts a 0.0-1.0 float fader value to approximate dBFS (-90 dB to +10 dB curve)
/// using standard X32/M32 4-segment piecewise scale.
pub fn fader_float_to_db(fader: f32) -> f32 {
    let f = fader.clamp(0.0, 1.0);
    if f <= 0.0001 {
        -90.0
    } else if f <= 0.25 {
        -90.0 + (f / 0.25) * 60.0
    } else if f <= 0.50 {
        -30.0 + ((f - 0.25) / 0.25) * 20.0
    } else if f <= 0.75 {
        -10.0 + ((f - 0.50) / 0.25) * 10.0
    } else {
        ((f - 0.75) / 0.25) * 10.0
    }
}

pub fn load_config_file(path: &str) -> Result<ConfigFile> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::Read::take(file, 1024 * 1024); // 1MB limit for safety
    let config: ConfigFile = serde_json::from_reader(&mut reader)?;
    Ok(config)
}

pub async fn run(args: Args) -> Result<()> {
    // Load config file if provided or if default x32_config.json exists
    let config_file = if let Some(ref config_path) = args.config {
        Some(load_config_file(config_path)?)
    } else if std::path::Path::new("x32_config.json").exists() {
        load_config_file("x32_config.json").ok()
    } else {
        None
    };

    let volunteer_sec = config_file.as_ref().and_then(|c| c.volunteer_mode.as_ref());

    let max_channels = max_channels_for_model(args.model);
    let max_dcas = max_dcas_for_model(args.model);

    // 1. Setup Network with resolved model-aware port
    let target_addr = resolve_target_address(&args.ip, args.model);
    let udp = UdpTransport::connect(&target_addr).await?;
    let transport: Arc<dyn MixerTransport> = Arc::new(udp);
    let network = MixerClient::new(transport, true);
    let mut rx = network.subscribe();

    // 2. Parse Channel Configuration merging CLI and Config file settings
    let mut configured_channels = vec![];

    if !args.channels.is_empty() {
        let parts = args.channels.split(',');
        for part in parts {
            let part = part.trim().to_lowercase();
            if let Some(stripped) = part.strip_prefix("ch") {
                if let Ok(num) = stripped.parse::<u32>() {
                    if num >= 1 && num <= max_channels {
                        configured_channels.push(ChannelState::new(
                            format!("/ch/{:02}", num),
                            false,
                            num,
                        ));
                    }
                }
            } else if let Some(stripped) = part.strip_prefix("dca") {
                if let Ok(num) = stripped.parse::<u32>() {
                    if num >= 1 && num <= max_dcas {
                        configured_channels.push(ChannelState::new(format!("/dca/{}", num), true, num));
                    }
                }
            }
        }
    } else if let Some(sec) = volunteer_sec {
        if let Some(ref vis_chs) = sec.visible_channels {
            for &num in vis_chs {
                if num >= 1 && num <= max_channels {
                    configured_channels.push(ChannelState::new(
                        format!("/ch/{:02}", num),
                        false,
                        num,
                    ));
                }
            }
        }
        if let Some(ref vis_dcas) = sec.visible_dcas {
            for &num in vis_dcas {
                if num >= 1 && num <= max_dcas {
                    configured_channels.push(ChannelState::new(format!("/dca/{}", num), true, num));
                }
            }
        }
    }

    // Default fallback if no channels specified or configured
    if configured_channels.is_empty() {
        let default_count = 8.min(max_channels);
        for i in 1..=default_count {
            configured_channels.push(ChannelState::new(format!("/ch/{:02}", i), false, i));
        }
    }

    let max_fader_limit = args
        .max_fader_limit_db
        .or_else(|| volunteer_sec.and_then(|s| s.max_fader_limit_db));

    let mut state = AppState::new(configured_channels);
    state.max_fader_limit_db = max_fader_limit;
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
                    // /meters/1 data length should be 4 (length) + n_channels * 4 (float32s)
                    for ch in &mut state.channels {
                        if !ch.is_dca {
                            let idx = (ch.num - 1) as usize;
                            let start = 4 + idx * 4;
                            if start + 4 <= data.len() {
                                let bytes: [u8; 4] =
                                    data[start..start + 4].try_into().unwrap_or([0; 4]);
                                let float_val = f32::from_le_bytes(bytes);
                                // The X32 returns level in linear format 0.0-1.0
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
            // Update alerts based on actual metered level and fader limit thresholds
            state.alerts.clear();
            state.alert_messages.clear();

            for (i, ch) in state.channels.iter().enumerate() {
                if !ch.muted && ch.level_db > -5.0 && !ch.is_dca {
                    state.alerts.push(i);
                    state.alert_messages.push(format!(
                        "• 🔴 {} signal near clipping ({:.1} dBFS)",
                        ch.name, ch.level_db
                    ));
                }

                if let Some(limit) = state.max_fader_limit_db {
                    let fader_db = fader_float_to_db(ch.fader);
                    if fader_db > limit {
                        if !state.alerts.contains(&i) {
                            state.alerts.push(i);
                        }
                        state.alert_messages.push(format!(
                            "• 🟡 {} fader {:.1} dB exceeds limit ({:.1} dB)",
                            ch.name, fader_db, limit
                        ));
                    }
                }
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

    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn test_args_parsing() {
        let args = Args {
            ip: "127.0.0.1".to_string(),
            model: MixerModel::X32,
            channels: "ch1,dca2".to_string(),
            config: None,
            max_fader_limit_db: Some(-3.0),
        };

        assert_eq!(args.ip, "127.0.0.1");
        assert_eq!(args.model, MixerModel::X32);
        assert_eq!(args.channels, "ch1,dca2");
        assert_eq!(args.max_fader_limit_db, Some(-3.0));
    }

    #[test]
    fn test_max_channels_and_dcas_for_model() {
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);

        assert_eq!(max_dcas_for_model(MixerModel::X32), 8);
        assert_eq!(max_dcas_for_model(MixerModel::Wing), 16);
        assert_eq!(max_dcas_for_model(MixerModel::XR18), 4);
        assert_eq!(max_dcas_for_model(MixerModel::XR16), 4);
        assert_eq!(max_dcas_for_model(MixerModel::XR12), 4);
    }

    #[test]
    fn test_resolve_target_address() {
        assert_eq!(resolve_target_address("192.168.1.100", MixerModel::X32), "192.168.1.100:10023");
        assert_eq!(resolve_target_address("192.168.1.100", MixerModel::Wing), "192.168.1.100:2222");
        assert_eq!(resolve_target_address("192.168.1.100", MixerModel::XR18), "192.168.1.100:10024");
        assert_eq!(resolve_target_address("192.168.1.100:10000", MixerModel::X32), "192.168.1.100:10000");
    }

    #[test]
    fn test_fader_float_to_db() {
        assert_eq!(fader_float_to_db(0.0), -90.0);
        assert_eq!(fader_float_to_db(0.25), -30.0);
        assert_eq!(fader_float_to_db(0.50), -10.0);
        assert_eq!(fader_float_to_db(0.75), 0.0);
        assert_eq!(fader_float_to_db(1.0), 10.0);

        // Verify continuous monotonic increase across intermediate steps
        let db_0_60 = fader_float_to_db(0.60);
        assert!(db_0_60 > -10.0 && db_0_60 < 0.0);
        let db_0_85 = fader_float_to_db(0.85);
        assert!(db_0_85 > 0.0 && db_0_85 < 10.0);
    }

    #[test]
    fn test_volunteer_config_deserialization() -> Result<()> {
        let json_data = r#"{
            "volunteer_mode": {
                "max_fader_limit_db": -3.0,
                "visible_channels": [1, 2, 3],
                "visible_dcas": [1]
            }
        }"#;

        let config: ConfigFile = serde_json::from_str(json_data)?;
        let sec = config.volunteer_mode.expect("Expected volunteer_mode section");
        assert_eq!(sec.max_fader_limit_db, Some(-3.0));
        assert_eq!(sec.visible_channels, Some(vec![1, 2, 3]));
        assert_eq!(sec.visible_dcas, Some(vec![1]));

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_connection() -> Result<()> {
        let server = UdpSocket::bind("127.0.0.1:0").await?;
        let addr = server.local_addr()?;

        // Just verify we can connect our transport to this address
        let transport = x32_lib::transport::udp::UdpTransport::connect(&addr.to_string()).await?;
        let _network = MixerClient::new(Arc::new(transport), false);

        Ok(())
    }
}
