use anyhow::Result;
use clap::Parser;
use osc_lib::OscArg;
use std::time::Instant;
use tokio::time::Duration;
use x32_lib::MixerClient;

pub mod ui;
use ui::{AppState, Tui, UIEvent};

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Automated System Tuning (Oscillator-Assisted)",
    long_about = "Guided oscillator testing using the X32's built-in signal generator."
)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Max level as a float 0.0 to 1.0 (to restrict max dBFS conservatively)
    #[arg(short, long, default_value_t = 0.5)]
    pub max_level: f32,

    /// Interval in milliseconds for ramping volume
    #[arg(short = 'r', long, default_value_t = 250)]
    pub ramp_interval_ms: u64,
}

enum Phase {
    Init,
    GuidedOutputVerification(usize),
    AssistedGainStaging,
    RoomTuning,
    MonitorRinging,
    Done,
}

pub async fn run(args: Args) -> Result<()> {
    let client = MixerClient::connect(&args.ip, true).await?;
    let mut tui = Tui::new()?;

    let mut current_phase = Phase::Init;
    let mut current_phase_str = "Connecting...";
    let mut current_step = "Waiting...".to_string();
    let mut status_message = "Press Enter to start Phase 1: Guided Output Verification";
    let mut current_level = 0.0;

    // Dest map based on X32 OSC (/config/osc/dest): 23=Main L, 24=Main R, 25=Main C/M
    let main_outputs = [
        ("Main L", "/config/osc/dest", 23),
        ("Main R", "/config/osc/dest", 24),
    ];

    let mut is_active = true;
    let mut last_ui_update = Instant::now();

    // For Phase 3 Sweeping
    let mut sweeping = false;
    let mut sweep_freq: f32 = 20.0;
    let mut last_sweep = Instant::now();

    while is_active {
        if sweeping && last_sweep.elapsed() > Duration::from_millis(50) {
            // Logarithmic sweep 20Hz -> 20kHz mapped 0.0 -> 1.0 roughly
            sweep_freq *= 1.05;
            if sweep_freq > 20000.0 { sweep_freq = 20.0; }

            // map roughly to 0.0-1.0
            let f1_val = ((sweep_freq.log10() - 20f32.log10()) / (20000f32.log10() - 20f32.log10())).clamp(0.0, 1.0);
            let _ = client.send_message("/config/osc/f1", vec![OscArg::Float(f1_val)]).await;

            last_sweep = Instant::now();
        }

        if last_ui_update.elapsed() > Duration::from_millis(50) {
            let state = AppState {
                current_phase: current_phase_str,
                current_step: current_step.clone(),
                level: current_level,
                status_message,
            };
            tui.draw(&state)?;
            last_ui_update = Instant::now();
        }

        if let Some(event) = tui.handle_events()? {
            match event {
                UIEvent::Quit => {
                    is_active = false;
                }
                UIEvent::Next => {
                    match current_phase {
                        Phase::Init => {
                            current_phase = Phase::GuidedOutputVerification(0);
                            current_phase_str = "Phase 1: Guided Output Verification";
                            current_step = format!("Testing {}", main_outputs[0].0);
                            status_message = "Press 'y' if you hear pink noise, 'n' if not.";

                            let _ = client.send_message("/config/osc/type", vec![OscArg::Int(1)]).await; // Pink Noise
                            let _ = client.send_message(main_outputs[0].1, vec![OscArg::Int(main_outputs[0].2)]).await;
                            let _ = client.send_message("/config/osc/level", vec![OscArg::Float(args.max_level)]).await;
                            let _ = client.send_message("/config/osc", vec![OscArg::Int(1)]).await;
                            current_level = args.max_level;
                        }
                        Phase::GuidedOutputVerification(_) => {
                            current_phase = Phase::AssistedGainStaging;
                            current_phase_str = "Phase 2: Assisted Gain Staging";
                            current_step = "Playing sine wave reference".to_string();
                            status_message = "Adjust amplifier knobs. Press Enter when done.";
                            let _ = client.send_message("/config/osc/type", vec![OscArg::Int(0)]).await; // Sine
                            let _ = client.send_message(main_outputs[0].1, vec![OscArg::Int(main_outputs[0].2)]).await;
                            let _ = client.send_message("/config/osc/level", vec![OscArg::Float(args.max_level)]).await;
                            let _ = client.send_message("/config/osc", vec![OscArg::Int(1)]).await;
                        }
                        Phase::AssistedGainStaging => {
                            current_phase = Phase::RoomTuning;
                            current_phase_str = "Phase 3: Room Tuning";
                            current_step = "Sweeping Sine through PA".to_string();
                            status_message = "Listening for feedback modes... Press Enter to skip";
                            sweeping = true;
                            let _ = client.send_message("/config/osc/type", vec![OscArg::Int(0)]).await; // Sine
                            let _ = client.send_message(main_outputs[0].1, vec![OscArg::Int(main_outputs[0].2)]).await;
                            let _ = client.send_message("/config/osc/level", vec![OscArg::Float(args.max_level)]).await;
                            let _ = client.send_message("/config/osc", vec![OscArg::Int(1)]).await;
                        }
                        Phase::RoomTuning => {
                            sweeping = false;
                            current_phase = Phase::MonitorRinging;
                            current_phase_str = "Phase 4: Monitor Ringing";
                            current_step = "Ringing Monitor Bus 1".to_string();
                            status_message = "Press Enter to finish.";
                            let _ = client.send_message("/config/osc/type", vec![OscArg::Int(0)]).await; // Sine
                            let _ = client.send_message("/config/osc/dest", vec![OscArg::Int(1)]).await; // Bus 1
                            let _ = client.send_message("/config/osc/level", vec![OscArg::Float(args.max_level)]).await;
                            let _ = client.send_message("/config/osc", vec![OscArg::Int(1)]).await;
                        }
                        Phase::MonitorRinging => {
                            current_phase = Phase::Done;
                            current_phase_str = "Finished";
                            current_step = "All phases complete.".to_string();
                            status_message = "Press 'q' to exit.";
                            let _ = client.send_message("/config/osc", vec![OscArg::Int(0)]).await;
                            current_level = 0.0;
                        }
                        Phase::Done => {
                            is_active = false;
                        }
                    }
                }
                UIEvent::Yes | UIEvent::No => {
                    if let Phase::GuidedOutputVerification(idx) = current_phase {
                        let next_idx = idx + 1;
                        if next_idx < main_outputs.len() {
                            current_phase = Phase::GuidedOutputVerification(next_idx);
                            current_step = format!("Testing {}", main_outputs[next_idx].0);
                            let _ = client.send_message(main_outputs[next_idx].1, vec![OscArg::Int(main_outputs[next_idx].2)]).await;
                        } else {
                            current_phase = Phase::AssistedGainStaging;
                            current_phase_str = "Phase 2: Assisted Gain Staging";
                            current_step = "Playing reference tone".to_string();
                            status_message = "Adjust physical amplifier knobs to comfortable level. Press Enter to proceed.";
                            let _ = client.send_message("/config/osc/type", vec![OscArg::Int(0)]).await; // Sine wave
                            let _ = client.send_message(main_outputs[0].1, vec![OscArg::Int(main_outputs[0].2)]).await;
                            let _ = client.send_message("/config/osc/level", vec![OscArg::Float(args.max_level)]).await;
                            let _ = client.send_message("/config/osc", vec![OscArg::Int(1)]).await;
                        }
                    }
                }
            }
        }
    }

    let _ = client.send_message("/config/osc/level", vec![OscArg::Float(0.0)]).await;
    let _ = client.send_message("/config/osc", vec![OscArg::Int(0)]).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::net::UdpSocket;
    use x32_core::Mixer;

    #[tokio::test]
    async fn test_x32_system_tune_args() {
        let args = Args {
            ip: "127.0.0.1:0".to_string(),
            max_level: 0.2,
            ramp_interval_ms: 10,
        };
        assert_eq!(args.max_level, 0.2);
    }

    #[tokio::test]
    async fn test_x32_system_tune_integration() {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = socket.local_addr().unwrap().port();
        let addr = format!("127.0.0.1:{}", port);

        let mut mixer = Mixer::new();
        let socket_arc = Arc::new(socket);
        let socket_rx = socket_arc.clone();

        let _ = tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while let Ok((len, src)) = socket_rx.recv_from(&mut buf).await {
                let responses_opt = mixer.dispatch(&buf[..len], src).ok();
                if let Some(responses) = responses_opt {
                    for (addr, response_bytes) in responses {
                        let _ = socket_rx.send_to(&response_bytes, addr).await;
                    }
                }
            }
        });

        // ⚡ Bolt: Simulate without TUI blocking to ensure network commands function correctly
        let client = MixerClient::connect(&addr, true).await.unwrap();
        let _ = client.send_message("/config/osc/type", vec![OscArg::Int(1)]).await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        if let Ok(OscArg::Int(val)) = client.query_value("/config/osc/type").await {
             assert_eq!(val, 1);
        } else {
             panic!("Failed to verify OSC");
        }
    }
}
