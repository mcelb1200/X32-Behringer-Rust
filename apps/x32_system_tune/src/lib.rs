use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use osc_lib::OscArg;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use tokio::time::{interval, Duration};
use x32_lib::MixerClient;

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Phase1Verification { output_idx: usize, outputs: Vec<String> },
    Phase2GainStaging,
    Phase3RoomTuning { freq_hz: f32 },
    Phase4MonitorRinging { bus: usize, level: f32, freq_hz: f32 },
}

pub struct TuiDropGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TuiDropGuard {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TuiDropGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

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

    /// Target destination index for the oscillator
    #[arg(short, long, default_value_t = 0)]
    pub dest: i32,

    /// Max level as a float 0.0 to 1.0 (to restrict max dBFS conservatively)
    #[arg(short, long, default_value_t = 0.5)]
    pub max_level: f32,

    /// Type of signal: 0 = Sine, 1 = Pink Noise, 2 = White Noise
    #[arg(short, long, default_value_t = 1)]
    pub signal_type: i32,

    /// Interval in milliseconds for ramping volume
    #[arg(short = 'r', long, default_value_t = 250)]
    pub ramp_interval_ms: u64,
}

pub async fn run(args: Args) -> Result<()> {
    let client = MixerClient::connect(&args.ip, true).await?;
    let mut tui_guard = TuiDropGuard::new()?;

    // 1. Initial configuration
    let _ = client
        .send_message("/config/osc/dest", vec![OscArg::Int(args.dest)])
        .await;
    let _ = client
        .send_message("/config/osc/type", vec![OscArg::Int(args.signal_type)])
        .await;
    let _ = client
        .send_message("/config/osc/level", vec![OscArg::Float(0.0)])
        .await;

    // 2. Start oscillator
    let _ = client
        .send_message("/config/osc", vec![OscArg::Int(1)])
        .await;

    let max_level = args.max_level.clamp(0.0, 1.0);
    let mut current_level = 0.0;
    let step_val = max_level / 20.0;

    let mut ticker = interval(Duration::from_millis(args.ramp_interval_ms));
    let mut is_active = true;
    let mut last_activity = std::time::Instant::now();
    let timeout_secs = 30;

    let mut phase = Phase::Phase1Verification {
        output_idx: 0,
        outputs: vec!["Main L/R".to_string(), "Matrix 1".to_string()],
    };

    while is_active {
        let timeout_future = tokio::time::sleep(Duration::from_millis(50));

        tokio::select! {
            _ = ticker.tick() => {
                if current_level < max_level {
                    current_level += step_val;
                    if current_level > max_level {
                        current_level = max_level;
                    }
                    let _ = client.send_message("/config/osc/level", vec![OscArg::Float(current_level)]).await;
                }

                // Progress sweeping logic based on phase
                match &mut phase {
                    Phase::Phase3RoomTuning { freq_hz } => {
                        *freq_hz += 10.0;
                        if *freq_hz > 20000.0 { *freq_hz = 20.0; }
                    }
                    Phase::Phase4MonitorRinging { bus: _, level, freq_hz } => {
                         *level += 0.5;
                         *freq_hz += 50.0;
                         if *level > max_level { *level = max_level; }
                         if *freq_hz > 20000.0 { *freq_hz = 20.0; }
                    }
                    _ => {}
                }
            }
            _ = timeout_future => {
                if last_activity.elapsed() > Duration::from_secs(timeout_secs) {
                    is_active = false;
                }

                if event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        last_activity = std::time::Instant::now();
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => is_active = false,
                            KeyCode::Char('n') => {
                                match &mut phase {
                                    Phase::Phase1Verification { output_idx, outputs } => {
                                        if *output_idx + 1 < outputs.len() {
                                            *output_idx += 1;
                                        } else {
                                            phase = Phase::Phase2GainStaging;
                                        }
                                    }
                                    Phase::Phase2GainStaging => {
                                        phase = Phase::Phase3RoomTuning { freq_hz: 20.0 };
                                    }
                                    Phase::Phase3RoomTuning { .. } => {
                                        phase = Phase::Phase4MonitorRinging { bus: 1, level: 0.0, freq_hz: 20.0 };
                                    }
                                    Phase::Phase4MonitorRinging { bus, .. } => {
                                        if *bus < 16 {
                                            *bus += 1;
                                        } else {
                                            is_active = false;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        tui_guard.terminal_mut().draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(10),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let header = Paragraph::new("Automated System Tuning")
                .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            let content = match &phase {
                Phase::Phase1Verification { output_idx, outputs } => {
                    format!("Phase 1: Guided Output Verification\nTesting Output: {}", outputs[*output_idx])
                }
                Phase::Phase2GainStaging => "Phase 2: Assisted Gain Staging\nAdjust amplifier levels now.".to_string(),
                Phase::Phase3RoomTuning { freq_hz } => {
                    format!("Phase 3: Room Tuning\nSweeping Sine: {:.1} Hz", freq_hz)
                }
                Phase::Phase4MonitorRinging { bus, level, freq_hz } => {
                    format!("Phase 4: Monitor Ringing\nBus: {}\nLevel: {:.2}\nFreq: {:.1} Hz", bus, level, freq_hz)
                }
            };

            let main_view = Paragraph::new(content)
                .block(Block::default().borders(Borders::ALL).title("Current Phase"));
            f.render_widget(main_view, chunks[1]);

            let footer = Paragraph::new("[N]ext Phase | [Q]uit")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;
    }

    // 4. Cleanup
    let _ = client
        .send_message("/config/osc/level", vec![OscArg::Float(0.0)])
        .await;
    let _ = client
        .send_message("/config/osc", vec![OscArg::Int(0)])
        .await;

    // Slight delay to ensure messages are sent
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
    async fn test_x32_system_tune() {
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

        let args = Args {
            ip: addr,
            dest: 2,
            max_level: 0.2,
            signal_type: 0,
            ramp_interval_ms: 10,
        };

        // Run the app in a background task
        let handle = tokio::spawn(async move {
            let _ = run(args).await;
        });

        // Let it run for a bit to initialize
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Since it relies on terminal input, we abort it to ensure it terminates
        // in environments without a TTY. A more complex test would simulate key events.
        handle.abort();
    }

    #[test]
    fn test_phase_transitions() {
        let mut phase = Phase::Phase1Verification {
            output_idx: 0,
            outputs: vec!["Main L/R".to_string(), "Matrix 1".to_string()],
        };

        // Simulating 'n' key press
        match &mut phase {
            Phase::Phase1Verification { output_idx, outputs } => {
                if *output_idx + 1 < outputs.len() {
                    *output_idx += 1;
                } else {
                    phase = Phase::Phase2GainStaging;
                }
            }
            _ => panic!("Expected Phase1"),
        }

        assert_eq!(phase, Phase::Phase1Verification {
            output_idx: 1,
            outputs: vec!["Main L/R".to_string(), "Matrix 1".to_string()],
        });

        match &mut phase {
            Phase::Phase1Verification { output_idx, outputs } => {
                if *output_idx + 1 < outputs.len() {
                    *output_idx += 1;
                } else {
                    phase = Phase::Phase2GainStaging;
                }
            }
            _ => panic!("Expected Phase1"),
        }

        assert_eq!(phase, Phase::Phase2GainStaging);

        match &mut phase {
             Phase::Phase2GainStaging => {
                  phase = Phase::Phase3RoomTuning { freq_hz: 20.0 };
             }
             _ => panic!("Expected Phase2"),
        }

        assert_eq!(phase, Phase::Phase3RoomTuning { freq_hz: 20.0 });
    }
}
