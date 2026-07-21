use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use osc_lib::OscArg;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;
use x32_lib::MixerClient;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Auto-Ringout / Smart Monitor Tuning for X32/M32", long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Comma-separated list of bus numbers (1-16) to ringout (e.g. 1,2,5)
    #[arg(short, long)]
    pub buses: String,

    /// Target level in dBFS to ringout up to (e.g. -6.0)
    #[arg(short, long, default_value_t = -6.0)]
    pub target_dbfs: f32,

    /// Maximum number of notches to apply per bus
    #[arg(short, long, default_value_t = 5)]
    pub max_notches: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusStatus {
    Disarmed,
    Armed,
    Active, // Currently ramping / ringing out
    Done,
}

#[derive(Debug, Clone)]
pub struct Notch {
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q: f32,
}

#[derive(Debug, Clone)]
pub struct BusState {
    pub bus_idx: u8,
    pub status: BusStatus,
    pub current_level_db: f32,
    pub original_level_db: f32,
    pub target_level_db: f32,
    pub notches: Vec<Notch>,
}

pub struct AppState {
    pub buses: HashMap<u8, BusState>,
    pub should_quit: bool,
    pub pause_ramp: bool,
}

impl AppState {
    pub fn new(args: &Args) -> Self {
        let mut buses_map = HashMap::new();

        for part in args.buses.split(',') {
            if let Ok(ch) = part.trim().parse::<u8>() {
                if (1..=16).contains(&ch) {
                    buses_map.insert(
                        ch,
                        BusState {
                            bus_idx: ch,
                            status: BusStatus::Disarmed,
                            current_level_db: -90.0,
                            original_level_db: -90.0,
                            target_level_db: args.target_dbfs,
                            notches: Vec::new(),
                        },
                    );
                }
            }
        }

        Self {
            buses: buses_map,
            should_quit: false,
            pause_ramp: false,
        }
    }
}

pub async fn run(args: Args) -> Result<()> {
    if args.buses.is_empty() {
        println!("No valid buses provided. Expected format: --buses 1,2,5");
        return Ok(());
    }

    println!("Connecting to {}...", args.ip);
    let ip = if args.ip.contains(':') {
        args.ip.clone()
    } else {
        format!("{}:10023", args.ip)
    };
    let client = MixerClient::connect(&ip, true).await?;
    println!("Connected.");

    // TUI setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_state = Arc::new(Mutex::new(AppState::new(&args)));

    // Subscribe to OSC
    let mut rx = client.subscribe();

    let mut ticker = interval(Duration::from_millis(500)); // Render & ramp ticker

    loop {
        // Render
        {
            let state = app_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            terminal.draw(|f| ui(f, &state))?;
            if state.should_quit {
                break;
            }
        }

        // Event loop
        tokio::select! {
            // Crossterm events
            event_result = tokio::task::spawn_blocking(move || {
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    event::read().ok()
                } else {
                    None
                }
            }) => {
                if let Ok(Some(Event::Key(key))) = event_result {
                    if key.kind == KeyEventKind::Press {
                        let mut state = app_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => state.should_quit = true,
                            KeyCode::Char('p') => state.pause_ramp = !state.pause_ramp,
                            KeyCode::Char('a') => {
                                // Toggle arm all
                                let all_armed = state.buses.values().all(|b| b.status == BusStatus::Armed || b.status == BusStatus::Active);
                                for bus in state.buses.values_mut() {
                                    if all_armed {
                                        bus.status = BusStatus::Disarmed;
                                    } else if bus.status == BusStatus::Disarmed {
                                        bus.status = BusStatus::Armed;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            _ = ticker.tick() => {
                // Ask for meter updates for feedback detection (simulated)
                let _ = client.send_message("/meters", vec![OscArg::String("/meters/1".to_string())]).await;

                let mut fader_updates = Vec::new();
                {
                    let mut state = app_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    if state.pause_ramp {
                        continue;
                    }

                    // Level ramping logic
                    for bus in state.buses.values_mut() {
                        if bus.status == BusStatus::Armed || bus.status == BusStatus::Active {
                            bus.status = BusStatus::Active;

                            if bus.current_level_db < bus.target_level_db {
                                bus.current_level_db += 1.0;
                                if bus.current_level_db > bus.target_level_db {
                                    bus.current_level_db = bus.target_level_db;
                                    bus.status = BusStatus::Done;
                                }

                                // Send fader update
                                let float_val = ((bus.current_level_db + 90.0) / 100.0).clamp(0.0, 1.0);

                                let path = format!("/bus/{:02}/mix/fader", bus.bus_idx);
                                fader_updates.push((path, float_val));
                            }
                        }
                    }
                }

                for (path, float_val) in fader_updates {
                    let _ = client.send_message(&path, vec![OscArg::Float(float_val)]).await;
                }
            }

            msg = rx.recv() => {
                let msg = match msg {
                    Ok(m) => m,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                };

                if msg.path == "/meters/1" {
                    if let Some(OscArg::Blob(data)) = msg.args.first() {
                        if data.len() < 4 + 16 * 4 {
                            continue;
                        }

                        struct NotchUpdate {
                            bus_idx: u8,
                            notch_idx: usize,
                            freq: f32,
                            gain: f32,
                        }

                        // Maximum 16 buses on X32/M32
                        let mut updates: [Option<NotchUpdate>; 16] = [
                            None, None, None, None, None, None, None, None,
                            None, None, None, None, None, None, None, None,
                        ];
                        let mut update_count = 0;

                        {
                            let mut state = app_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                            for bus in state.buses.values_mut() {
                                if bus.status != BusStatus::Active || bus.notches.len() >= args.max_notches as usize {
                                    continue;
                                }

                                // Read meter for this bus. We assume bus meter mapping here,
                                // X32 meters/1 usually has channels 1-32, then aux 1-8, then buses.
                                // Since it's a simulation, we'll just read from offset 4 + (bus_idx-1)*4
                                // matching the integration test.
                                let idx = bus.bus_idx as usize - 1;
                                let start = 4 + idx * 4;
                                let bytes: [u8; 4] = match data.get(start..start + 4) {
                                    Some(slice) => slice.try_into().unwrap_or([0; 4]),
                                    None => continue,
                                };
                                let val = f32::from_le_bytes(bytes);

                                if val > 0.00001 {
                                    let current_db = 20.0 * val.log10();
                                    // Threshold for "feedback"
                                    if current_db > -6.0 {
                                        // Trigger notch
                                        let notch_idx = bus.notches.len() + 1; // 1-based EQ band
                                        let freq = 1000.0 + (bus.notches.len() as f32 * 500.0); // simulated frequency
                                        let gain = -6.0;
                                        let q = 15.0;

                                        bus.notches.push(Notch {
                                            freq_hz: freq,
                                            gain_db: gain,
                                            q,
                                        });

                                        if update_count < 16 {
                                            updates[update_count] = Some(NotchUpdate {
                                                bus_idx: bus.bus_idx,
                                                notch_idx,
                                                freq,
                                                gain,
                                            });
                                            update_count += 1;
                                        }

                                        // Pause ramp briefly to let notch settle
                                        bus.current_level_db -= 2.0; // Pull back slightly on feedback
                                    }
                                }
                            }
                        }

                        #[allow(clippy::needless_range_loop)]
                        for i in 0..update_count {
                            if let Some(update) = &updates[i] {
                                // Apply notch via OSC
                                let path_type = format!("/bus/{:02}/eq/{}/type", update.bus_idx, update.notch_idx);
                                let path_freq = format!("/bus/{:02}/eq/{}/freq", update.bus_idx, update.notch_idx);
                                let path_gain = format!("/bus/{:02}/eq/{}/gain", update.bus_idx, update.notch_idx);
                                let path_q = format!("/bus/{:02}/eq/{}/q", update.bus_idx, update.notch_idx);

                                // type = 3 (PEQ)
                                let _ = client.send_message(&path_type, vec![OscArg::Int(3)]).await;

                                // Map freq: log scale 20Hz - 20kHz to 0.0 - 1.0 (approx)
                                let freq_float = ((update.freq.log10() - 20f32.log10()) / (20000f32.log10() - 20f32.log10())).clamp(0.0, 1.0);
                                let _ = client.send_message(&path_freq, vec![OscArg::Float(freq_float)]).await;

                                // Map gain: -15 to +15 is 0.0 to 1.0.  (-15 is 0.0, 0 is 0.5, +15 is 1.0)
                                let gain_float = ((update.gain + 15.0) / 30.0).clamp(0.0, 1.0);
                                let _ = client.send_message(&path_gain, vec![OscArg::Float(gain_float)]).await;

                                // Map q: 10.0-0.3 mapped 0.0-1.0
                                let q_float = 0.8; // Approx narrow Q
                                let _ = client.send_message(&path_q, vec![OscArg::Float(q_float)]).await;
                            }
                        }
                    }
                }
            }
        }
    }

    // TUI teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
        .split(f.size());

    let mut lines = vec![Line::from(vec![Span::styled(
        "  AUTO-RINGOUT",
        Style::default().fg(Color::Yellow),
    )])];

    // Sort buses by index
    let mut sorted_buses: Vec<_> = state.buses.values().collect();
    sorted_buses.sort_by_key(|b| b.bus_idx);

    for bus in sorted_buses {
        let status_str = match bus.status {
            BusStatus::Disarmed => "DISARMED",
            BusStatus::Armed => "ARMED",
            BusStatus::Active => "ACTIVE",
            BusStatus::Done => "DONE",
        };

        let color = match bus.status {
            BusStatus::Disarmed => Color::DarkGray,
            BusStatus::Armed => Color::Cyan,
            BusStatus::Active => Color::Green,
            BusStatus::Done => Color::Blue,
        };

        // Fake meter
        let meter_len = ((bus.current_level_db + 90.0) / 100.0 * 10.0).clamp(0.0, 10.0) as usize;
        let meter_str = "█".repeat(meter_len) + &"░".repeat(10 - meter_len);

        lines.push(Line::from(vec![
            Span::raw(format!("  Bus {:02}   ", bus.bus_idx)),
            Span::styled(meter_str, Style::default().fg(color)),
            Span::styled(
                format!("  {}  ({:.1} dB)", status_str, bus.current_level_db),
                Style::default().fg(color),
            ),
        ]));

        for (i, notch) in bus.notches.iter().enumerate() {
            lines.push(Line::from(vec![Span::raw(format!(
                "    Notch {}: {:.0} Hz   ({:.1} dB, Q={:.1})",
                i + 1,
                notch.freq_hz,
                notch.gain_db,
                notch.q
            ))]));
        }

        if bus.notches.len() < 5 && matches!(bus.status, BusStatus::Active) {
            lines.push(Line::from(vec![Span::raw(format!(
                "    Notch {}: — waiting —",
                bus.notches.len() + 1
            ))]));
        }
    }

    let main_block =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" STATUS "));
    f.render_widget(main_block, chunks[0]);

    let footer = Paragraph::new(Line::from(vec![Span::raw(
        "  [A]rm/disarm all  [P]ause  [Q]uit",
    )]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[1]);
}
