mod app;

use anyhow::Result;
use app::{AppState, InputMode};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use clap::Parser;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use osc_lib::{OscArg, OscMessage};

/// A Rust implementation of the X32Tap utility with a Text User Interface.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    let _args = Args::parse();
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let app = Arc::new(Mutex::new(AppState::new()));

    // Channel for sending OSC messages to the network task
    let (tx, rx) = mpsc::channel::<OscMessage>(100);

    // Spawn network task
    let app_clone = app.clone();
    let network_task = tokio::spawn(async move {
        let _ = run_network(app_clone, rx).await;
    });

    // Run application UI in the current thread
    let res = run_app(&mut terminal, app.clone(), tx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    // Abort network task if UI exits
    network_task.abort();

    Ok(())
}

async fn run_network(app: Arc<Mutex<AppState>>, mut rx: mpsc::Receiver<OscMessage>) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mut current_ip = String::new();
    let mut addr = None;
    let mut buf = [0u8; 1024];

    let mut last_keepalive = Instant::now() - Duration::from_secs(10);

    loop {
        let (ip, is_auto, channel) = {
            let state = app.lock().unwrap();
            (state.ip_input.clone(), state.is_auto, state.channel)
        };

        if ip != current_ip {
            current_ip = ip.clone();
            if let Ok(parsed_addr) = format!("{}:10023", current_ip).parse::<std::net::SocketAddr>() {
                addr = Some(parsed_addr);
                // Connect check
                if let Some(a) = addr {
                    let info_msg = OscMessage::new("/info".to_string(), vec![]);
                    if let Ok(bytes) = info_msg.to_bytes() {
                        let _ = socket.send_to(&bytes, a).await;
                    }
                }
            } else {
                addr = None;
            }
        }

        if let Some(target_addr) = addr {
            // Keepalive / meter subscription in auto mode
            if is_auto {
                let now = Instant::now();
                if now.duration_since(last_keepalive).as_secs() >= 9 {
                    let xremote = OscMessage::new("/xremote".to_string(), vec![]);
                    if let Ok(bytes) = xremote.to_bytes() {
                        let _ = socket.send_to(&bytes, target_addr).await;
                    }

                    let meter_req = OscMessage::new(
                        "/meters".to_string(),
                        vec![
                            OscArg::String("/meters/6".to_string()),
                            OscArg::Int(0),
                            OscArg::Int(0),
                            OscArg::Int((channel - 1) as i32),
                        ],
                    );
                    if let Ok(bytes) = meter_req.to_bytes() {
                        let _ = socket.send_to(&bytes, target_addr).await;
                    }
                    last_keepalive = now;
                }
            }

            tokio::select! {
                // Process outgoing messages from UI
                Some(msg) = rx.recv() => {
                    if let Ok(bytes) = msg.to_bytes() {
                        let _ = socket.send_to(&bytes, target_addr).await;
                    }
                }
                // Process incoming UDP packets
                result = tokio::time::timeout(Duration::from_millis(50), socket.recv_from(&mut buf)) => {
                    if let Ok(Ok((len, _))) = result {
                        if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                            if msg.path == "/info" {
                                app.lock().unwrap().is_connected = true;
                                app.lock().unwrap().log("Connected!".to_string());
                            } else if msg.path == "/meters/6" {
                                if let Some(OscArg::Blob(data)) = msg.args.first() {
                                    if let Some(level) = AppState::parse_meter_blob(data) {
                                        let maybe_msg = {
                                            let mut state = app.lock().unwrap();
                                            if state.is_auto {
                                                let now = Instant::now();
                                                if let Some(f_val) = state.process_meter_data(level, now) {
                                                    let delay_ms = state.current_delay_ms.unwrap_or(0);
                                                    state.log(format!("Auto Tap: {}ms (level: {:.2})", delay_ms, level));

                                                    let slot = state.slot;
                                                    let address = format!("/fx/{}/par/02", slot);
                                                    Some(OscMessage::new(address, vec![OscArg::Float(f_val)]))
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        };
                                        if let Some(msg) = maybe_msg {
                                            if let Ok(bytes) = msg.to_bytes() {
                                                let _ = socket.send_to(&bytes, target_addr).await;
                                            }
                                        }
                                    }
                                }
                            } else if msg.path.starts_with("/fx/") && msg.path.ends_with("/type") {
                                if let Some(OscArg::Int(t)) = msg.args.first() {
                                    app.lock().unwrap().delay_type = format!("Type ID: {}", t);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Discard outgoing if not connected
            while let Ok(_) = rx.try_recv() {}
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<AppState>>, tx: mpsc::Sender<OscMessage>) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        let mut app_state = app.lock().unwrap();
        terminal.draw(|f| ui(f, &app_state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app_state.active_input {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app_state.should_quit = true,
                            KeyCode::Char('i') => app_state.active_input = InputMode::EditingIp,
                            KeyCode::Char('s') => app_state.active_input = InputMode::EditingSlot,
                            KeyCode::Char('c') => app_state.active_input = InputMode::EditingChannel,
                            KeyCode::Char('e') => app_state.active_input = InputMode::EditingSensitivity,
                            KeyCode::Char('a') => {
                                app_state.is_auto = !app_state.is_auto;
                                let status = if app_state.is_auto { "Auto" } else { "Manual" };
                                app_state.log(format!("Switched to {} mode", status));
                            }
                            KeyCode::Enter => {
                                // Manual tap
                                if !app_state.is_auto {
                                    let now = Instant::now();
                                    if let Some(f_val) = app_state.handle_manual_tap(now) {
                                        let delay_ms = app_state.current_delay_ms.unwrap_or(0);
                                        app_state.log(format!("Tapped: {}ms", delay_ms));

                                        let slot = app_state.slot;
                                        let address = format!("/fx/{}/par/02", slot);
                                        let msg = OscMessage::new(address, vec![OscArg::Float(f_val)]);
                                        let _ = tx.try_send(msg);
                                    } else {
                                        app_state.log("First tap...".to_string());
                                    }
                                }
                            }
                            _ => {}
                        },
                        InputMode::EditingIp => match key.code {
                            KeyCode::Enter | KeyCode::Esc => app_state.active_input = InputMode::Normal,
                            KeyCode::Char(c) => app_state.ip_input.push(c),
                            KeyCode::Backspace => {
                                app_state.ip_input.pop();
                            }
                            _ => {}
                        },
                        InputMode::EditingSlot => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app_state.active_input = InputMode::Normal;
                                if let Ok(val) = app_state.slot_input.parse() {
                                    app_state.slot = val;
                                }
                            }
                            KeyCode::Char(c) if c.is_digit(10) => app_state.slot_input.push(c),
                            KeyCode::Backspace => {
                                app_state.slot_input.pop();
                            }
                            _ => {}
                        },
                        InputMode::EditingChannel => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app_state.active_input = InputMode::Normal;
                                if let Ok(val) = app_state.ch_input.parse() {
                                    app_state.channel = val;
                                }
                            }
                            KeyCode::Char(c) if c.is_digit(10) => app_state.ch_input.push(c),
                            KeyCode::Backspace => {
                                app_state.ch_input.pop();
                            }
                            _ => {}
                        },
                        InputMode::EditingSensitivity => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app_state.active_input = InputMode::Normal;
                                if let Ok(val) = app_state.sens_input.parse() {
                                    app_state.threshold = val;
                                }
                            }
                            KeyCode::Char(c) if c.is_digit(10) || c == '.' => {
                                app_state.sens_input.push(c)
                            }
                            KeyCode::Backspace => {
                                app_state.sens_input.pop();
                            }
                            _ => {}
                        },
                    }
                }
            }
        }

        if app_state.should_quit {
            return Ok(());
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Help
                Constraint::Length(8), // Controls
                Constraint::Min(5),    // Log
            ]
            .as_ref(),
        )
        .split(f.size());

    // Help block
    let help_msg = match app.active_input {
        InputMode::Normal => "q/Esc: Quit | i: IP | s: Slot | c: Channel | e: Sensitivity | a: Toggle Auto | Enter: Tap",
        _ => "Enter/Esc: Confirm/Stop Editing",
    };
    let help = Paragraph::new(help_msg).block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[0]);

    // Controls block
    let controls_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
        .split(controls_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
        .split(controls_chunks[1]);

    // IP Input
    let ip_style = if app.active_input == InputMode::EditingIp {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let ip_text = format!("IP: {}", app.ip_input);
    let ip_p = Paragraph::new(ip_text)
        .style(ip_style)
        .block(Block::default().borders(Borders::ALL).title("Connection"));
    f.render_widget(ip_p, left_chunks[0]);

    // Mode / Settings
    let mode_text = format!(
        "Mode: {}\nCheck: {}",
        if app.is_auto { "Auto" } else { "Manual" },
        app.delay_type
    );
    let mode_p = Paragraph::new(mode_text)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(mode_p, left_chunks[1]);

    // Delay Slot
    let slot_style = if app.active_input == InputMode::EditingSlot {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let slot_text = format!("Delay Slot: {}", app.slot_input);
    let slot_p = Paragraph::new(slot_text)
        .style(slot_style)
        .block(Block::default().borders(Borders::ALL).title("FX Slot"));
    f.render_widget(slot_p, right_chunks[0]);

    // Channel Settings
    let mut ch_text = vec![];
    let mut ch_line = vec![Span::raw("Channel: ")];
    if app.active_input == InputMode::EditingChannel {
        ch_line.push(Span::styled(&app.ch_input, Style::default().fg(Color::Yellow)));
    } else {
        ch_line.push(Span::raw(&app.ch_input));
    }
    ch_text.push(Line::from(ch_line));

    let mut sens_line = vec![Span::raw("Sens: ")];
    if app.active_input == InputMode::EditingSensitivity {
        sens_line.push(Span::styled(&app.sens_input, Style::default().fg(Color::Yellow)));
    } else {
        sens_line.push(Span::raw(&app.sens_input));
    }
    ch_text.push(Line::from(sens_line));

    let ch_p = Paragraph::new(ch_text)
        .block(Block::default().borders(Borders::ALL).title("Auto-Tap Settings"));
    f.render_widget(ch_p, right_chunks[1]);

    // Log window
    let logs: Vec<Line> = app
        .logs
        .iter()
        .map(|msg| Line::from(Span::raw(msg.clone())))
        .collect();
    let log_p = Paragraph::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));
    f.render_widget(log_p, chunks[2]);
}
