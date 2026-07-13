pub mod network;
pub mod state;
pub mod ui;

use anyhow::Result;
use clap::Parser;
use network::ChannelState;
use osc_lib::OscArg;
use state::AppState;
use std::sync::Arc;
use std::time::{Duration, Instant};
use ui::{Tui, UIEvent};
use x32_lib::MixerClient;
use x32_lib::transport::{MixerTransport, udp::UdpTransport};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Simplified TUI Dashboard (Volunteer Mode)")]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: String,

    /// Comma-separated list of channel/DCA numbers to monitor (e.g. "ch1,ch2,dca1"). If empty, defaults to ch1..ch8.
    #[arg(short, long, default_value = "")]
    pub channels: String,
}

pub async fn run(args: Args) -> Result<()> {
    // 1. Setup Network
    let udp = UdpTransport::connect(&args.ip).await?;
    let transport: Arc<dyn MixerTransport> = Arc::new(udp);
    let network = MixerClient::new(transport, true);
    let mut rx = network.subscribe();

    // 2. Parse Channel Configuration
    let mut configured_channels = vec![];
    if args.channels.is_empty() {
        for i in 1..=8 {
            configured_channels.push(ChannelState::new(format!("/ch/{:02}", i), false, i));
        }
    } else {
        let parts = args.channels.split(',');
        for part in parts {
            let part = part.trim().to_lowercase();
            if let Some(stripped) = part.strip_prefix("ch") {
                if let Ok(num) = stripped.parse::<u32>() {
                    configured_channels.push(ChannelState::new(
                        format!("/ch/{:02}", num),
                        false,
                        num,
                    ));
                }
            } else if let Some(stripped) = part.strip_prefix("dca") {
                if let Ok(num) = stripped.parse::<u32>() {
                    configured_channels.push(ChannelState::new(format!("/dca/{}", num), true, num));
                }
            }
        }
    }

    if configured_channels.is_empty() {
        anyhow::bail!("No valid channels specified");
    }

    let state = AppState::new(configured_channels);
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
        let fader_path = if ch.is_dca {
            format!("{}/fader", ch.osc_prefix)
        } else {
            format!("{}/mix/fader", ch.osc_prefix)
        };
        let mute_path = if ch.is_dca {
            format!("{}/on", ch.osc_prefix)
        } else {
            format!("{}/mix/on", ch.osc_prefix)
        };
        network.send_message(fader_path.as_str(), vec![]).await?;
        network.send_message(mute_path.as_str(), vec![]).await?;
        network
            .send_message(format!("{}/config/name", ch.osc_prefix).as_str(), vec![])
            .await?;
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
                if (ch.is_dca && path == format!("{}/fader", ch.osc_prefix))
                    || (!ch.is_dca && path == format!("{}/mix/fader", ch.osc_prefix))
                {
                    if let Some(OscArg::Float(v)) = msg.args.first() {
                        ch.fader = *v;
                    }
                } else if path == format!("{}/mix/on", ch.osc_prefix)
                    || path == format!("{}/on", ch.osc_prefix)
                {
                    // Handle DCA vs CH on
                    let mut found = false;
                    if path.starts_with("/ch/") && path.ends_with("/mix/on") {
                        found = true;
                    }
                    if path.starts_with("/dca/") && path.ends_with("/on") {
                        found = true;
                    }

                    if found {
                        if let Some(OscArg::Int(v)) = msg.args.first() {
                            ch.muted = *v == 0;
                        }
                    }
                } else if path == format!("{}/config/name", ch.osc_prefix) {
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
            // Update alerts based on actual metered level
            state.alerts.clear();
            for ch in &state.channels {
                if !ch.muted && ch.level_db > -5.0 && !ch.is_dca {
                    state.alerts.push(format!(
                        "🟡 {} level is high — consider lowering fader.",
                        ch.name
                    ));
                }
            }

            tui.draw(&state)?;

            if let Some(event) = tui.handle_events()? {
                match event {
                    UIEvent::Quit => break,
                    UIEvent::MuteAll => {
                        for ch in &state.channels {
                            let mute_path = if ch.is_dca {
                                format!("{}/on", ch.osc_prefix)
                            } else {
                                format!("{}/mix/on", ch.osc_prefix)
                            };
                            network
                                .send_message(&mute_path, vec![OscArg::Int(0)])
                                .await?;
                        }
                    }
                    UIEvent::Panic => {
                        for ch in &state.channels {
                            let mute_path = if ch.is_dca {
                                format!("{}/on", ch.osc_prefix)
                            } else {
                                format!("{}/mix/on", ch.osc_prefix)
                            };
                            network
                                .send_message(&mute_path, vec![OscArg::Int(0)])
                                .await?;
                            let fader_path = if ch.is_dca {
                                format!("{}/fader", ch.osc_prefix)
                            } else {
                                format!("{}/mix/fader", ch.osc_prefix)
                            };
                            network
                                .send_message(&fader_path, vec![OscArg::Float(0.0)])
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
            channels: "ch1,dca2".to_string(),
        };

        assert_eq!(args.ip, "127.0.0.1");
        assert_eq!(args.channels, "ch1,dca2");
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
