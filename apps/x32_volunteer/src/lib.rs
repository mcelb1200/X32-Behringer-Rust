pub mod network;
pub mod state;
pub mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use network::ChannelState;
use osc_lib::OscArg;
use serde::Deserialize;
use state::AppState;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use ui::{Tui, UIEvent};
use x32_lib::transport::{MixerTransport, udp::UdpTransport};
use x32_lib::{MixerClient, MixerModel};

/// Configuration structure for volunteer mode settings
#[derive(Debug, Deserialize, Clone, Default)]
pub struct VolunteerConfig {
    /// Maximum safe fader limit in dB (e.g. -3.0)
    pub max_fader_limit_db: Option<f32>,
    /// List of visible channel numbers (1-indexed)
    pub visible_channels: Option<Vec<u32>>,
    /// List of visible DCA group numbers (1-indexed)
    pub visible_dcas: Option<Vec<u32>>,
}

/// Top-level application configuration structure (`x32_config.json`)
#[derive(Debug, Deserialize, Clone, Default)]
pub struct AppConfig {
    /// Volunteer mode section
    pub volunteer_mode: Option<VolunteerConfig>,
}

/// CLI arguments for x32_volunteer
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Simplified TUI Dashboard (Volunteer Mode)")]
pub struct Args {
    /// IP address (and optional port) of the X32/Wing/XAir console
    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short, long, default_value = "X32")]
    pub model: MixerModel,

    /// Path to JSON configuration file (defaults to x32_config.json if present)
    #[arg(short, long)]
    pub config: Option<String>,

    /// Comma-separated list of channel/DCA numbers to monitor (e.g. "ch1,ch2,dca1"). If empty, defaults to config or ch1..ch8.
    #[arg(short, long, default_value = "")]
    pub channels: String,
}

/// Returns the maximum channel count for the given mixer model
pub fn max_channels_for_model(model: MixerModel) -> u32 {
    match model {
        MixerModel::X32 => 32,
        MixerModel::Wing => 40,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

/// Returns the maximum DCA count for the given mixer model
pub fn max_dcas_for_model(model: MixerModel) -> u32 {
    match model {
        MixerModel::X32 => 8,
        MixerModel::Wing => 16,
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
    }
}

/// Resolves the target UDP address with model-specific OSC port defaults
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

/// Converts a normalized fader position (0.0 to 1.0) into logarithmic dB
pub fn fader_position_to_db(fader: f32) -> f32 {
    if fader <= 0.0 {
        return -144.0;
    }
    if fader < 0.25 {
        -90.0 + (fader / 0.25) * 60.0
    } else if fader < 0.50 {
        -30.0 + ((fader - 0.25) / 0.25) * 20.0
    } else if fader < 0.75 {
        -10.0 + ((fader - 0.50) / 0.25) * 10.0
    } else {
        ((fader - 0.75) / 0.25) * 10.0
    }
}

/// Loads configuration from file if specified or if default `x32_config.json` exists
pub fn load_config(config_path: Option<&str>) -> Result<AppConfig> {
    let path_to_open = match config_path {
        Some(path) => Some(path.to_string()),
        None => {
            if Path::new("x32_config.json").exists() {
                Some("x32_config.json".to_string())
            } else {
                None
            }
        }
    };

    if let Some(path) = path_to_open {
        let file =
            File::open(&path).with_context(|| format!("Failed to open config file: {}", path))?;
        let mut buf = String::new();
        file.take(64 * 1024).read_to_string(&mut buf)?;
        let cfg: AppConfig = serde_json::from_str(&buf)
            .with_context(|| format!("Failed to parse JSON config in {}", path))?;
        Ok(cfg)
    } else {
        Ok(AppConfig::default())
    }
}

/// Constructs configured channel states considering CLI args, JSON config, and model bounds
pub fn build_channels(args: &Args, config: &AppConfig) -> Result<Vec<ChannelState>> {
    let max_ch = max_channels_for_model(args.model);
    let max_dca = max_dcas_for_model(args.model);
    let mut configured_channels = vec![];

    if !args.channels.is_empty() {
        let parts = args.channels.split(',');
        for part in parts {
            let part = part.trim().to_lowercase();
            if let Some(stripped) = part.strip_prefix("ch") {
                if let Ok(num) = stripped.parse::<u32>() {
                    if num >= 1 && num <= max_ch {
                        configured_channels.push(ChannelState::new(
                            format!("/ch/{:02}", num),
                            false,
                            num,
                        ));
                    }
                }
            } else if let Some(stripped) = part.strip_prefix("dca") {
                if let Ok(num) = stripped.parse::<u32>() {
                    if num >= 1 && num <= max_dca {
                        configured_channels.push(ChannelState::new(
                            format!("/dca/{}", num),
                            true,
                            num,
                        ));
                    }
                }
            }
        }
    } else if let Some(ref v_cfg) = config.volunteer_mode {
        if let Some(ref chs) = v_cfg.visible_channels {
            for &num in chs {
                if num >= 1 && num <= max_ch {
                    configured_channels.push(ChannelState::new(
                        format!("/ch/{:02}", num),
                        false,
                        num,
                    ));
                }
            }
        }
        if let Some(ref dcas) = v_cfg.visible_dcas {
            for &num in dcas {
                if num >= 1 && num <= max_dca {
                    configured_channels.push(ChannelState::new(format!("/dca/{}", num), true, num));
                }
            }
        }
    }

    // Default fallback if no channels specified or parsed
    if configured_channels.is_empty() {
        let default_count = std::cmp::min(8, max_ch);
        for i in 1..=default_count {
            configured_channels.push(ChannelState::new(format!("/ch/{:02}", i), false, i));
        }
    }

    if configured_channels.is_empty() {
        anyhow::bail!("No valid channels specified for model {}", args.model);
    }

    Ok(configured_channels)
}

pub async fn run(args: Args) -> Result<()> {
    let target_addr = resolve_target_address(&args.ip, args.model);
    let config = load_config(args.config.as_deref())?;

    // 1. Setup Network
    let udp = UdpTransport::connect(&target_addr).await?;
    let transport: Arc<dyn MixerTransport> = Arc::new(udp);
    let network = MixerClient::new(transport, true);
    let mut rx = network.subscribe();

    // 2. Parse Channel Configuration
    let configured_channels = build_channels(&args, &config)?;
    let mut state = AppState::new(configured_channels);
    if let Some(ref v_cfg) = config.volunteer_mode {
        state.max_fader_limit_db = v_cfg.max_fader_limit_db;
    }

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
            // Update alerts based on actual metered level and fader limits
            state.alerts.clear();
            for (i, ch) in state.channels.iter().enumerate() {
                let fader_db = fader_position_to_db(ch.fader);
                let exceeds_limit = match state.max_fader_limit_db {
                    Some(limit) => fader_db > limit,
                    None => false,
                };
                if (!ch.muted && ch.level_db > -5.0 && !ch.is_dca) || exceeds_limit {
                    state.alerts.push(i);
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
            config: None,
            channels: "ch1,dca2".to_string(),
        };

        assert_eq!(args.ip, "127.0.0.1");
        assert_eq!(args.channels, "ch1,dca2");
        assert_eq!(args.model, MixerModel::X32);
    }

    #[test]
    fn test_load_config_and_parsing() {
        let json = r#"{
            "volunteer_mode": {
                "max_fader_limit_db": -3.0,
                "visible_channels": [1, 2, 5],
                "visible_dcas": [1, 2]
            }
        }"#;
        let tmp_file = std::env::temp_dir().join("test_x32_config.json");
        std::fs::write(&tmp_file, json).unwrap();

        let cfg = load_config(Some(tmp_file.to_str().unwrap())).unwrap();
        let v_cfg = cfg.volunteer_mode.unwrap();
        assert_eq!(v_cfg.max_fader_limit_db, Some(-3.0));
        assert_eq!(v_cfg.visible_channels, Some(vec![1, 2, 5]));
        assert_eq!(v_cfg.visible_dcas, Some(vec![1, 2]));

        let _ = std::fs::remove_file(tmp_file);
    }

    #[test]
    fn test_model_bounds() {
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);

        assert_eq!(max_dcas_for_model(MixerModel::X32), 8);
        assert_eq!(max_dcas_for_model(MixerModel::Wing), 16);
        assert_eq!(max_dcas_for_model(MixerModel::XR18), 4);
        assert_eq!(max_dcas_for_model(MixerModel::XR12), 4);
    }

    #[test]
    fn test_resolve_target_address() {
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::X32),
            "192.168.1.50:10023"
        );
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::Wing),
            "192.168.1.50:2222"
        );
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::XR18),
            "192.168.1.50:10024"
        );
        assert_eq!(
            resolve_target_address("192.168.1.50:9000", MixerModel::X32),
            "192.168.1.50:9000"
        );
    }

    #[test]
    fn test_fader_position_to_db() {
        assert_eq!(fader_position_to_db(0.0), -144.0);
        assert!((fader_position_to_db(0.25) - -30.0).abs() < 0.01);
        assert!((fader_position_to_db(0.50) - -10.0).abs() < 0.01);
        assert!((fader_position_to_db(0.75) - 0.0).abs() < 0.01);
        assert!((fader_position_to_db(1.00) - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_build_channels_with_cli_args_and_bounds() {
        let args = Args {
            ip: "127.0.0.1".to_string(),
            model: MixerModel::XR12,
            config: None,
            channels: "ch1,ch12,ch13,dca4,dca5".to_string(),
        };
        let cfg = AppConfig::default();
        let chs = build_channels(&args, &cfg).unwrap();
        // ch13 and dca5 are out of bounds for XR12
        assert_eq!(chs.len(), 3);
        assert_eq!(chs[0].osc_prefix, "/ch/01");
        assert_eq!(chs[1].osc_prefix, "/ch/12");
        assert_eq!(chs[2].osc_prefix, "/dca/4");
    }

    #[test]
    fn test_build_channels_from_config() {
        let args = Args {
            ip: "127.0.0.1".to_string(),
            model: MixerModel::X32,
            config: None,
            channels: "".to_string(),
        };
        let cfg = AppConfig {
            volunteer_mode: Some(VolunteerConfig {
                max_fader_limit_db: Some(-3.0),
                visible_channels: Some(vec![1, 2, 3]),
                visible_dcas: Some(vec![1]),
            }),
        };
        let chs = build_channels(&args, &cfg).unwrap();
        assert_eq!(chs.len(), 4);
        assert_eq!(chs[0].osc_prefix, "/ch/01");
        assert_eq!(chs[1].osc_prefix, "/ch/02");
        assert_eq!(chs[2].osc_prefix, "/ch/03");
        assert_eq!(chs[3].osc_prefix, "/dca/1");
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
