pub mod app;

use anyhow::{Result, anyhow};
use app::{AppState, InputMode};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use osc_lib::{OscArg, OscMessage};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use x32_lib::MixerModel;

/// Returns the maximum number of FX slots supported by the mixer model.
pub fn max_fx_slots_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 => 8,
        MixerModel::Wing => 16,
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
    }
}

/// Returns the maximum number of input channels supported by the mixer model.
pub fn max_channels_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::Wing => 40,
        MixerModel::X32 => 32,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

/// A Rust implementation of the X32Tap utility with a Text User Interface.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Digital mixer model (X32, Wing, XR18, XR16, XR12)
    #[arg(short = 'M', long, default_value = "X32")]
    pub model: MixerModel,

    /// The IP address of the mixer.
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    /// The FX slot number containing the delay effect.
    #[arg(short, long, default_value_t = 1)]
    pub slot: u8,

    /// Enable auto-tap mode by monitoring a channel's meter level.
    #[arg(short, long)]
    pub auto: bool,

    /// In auto mode, the channel number to monitor.
    #[arg(short, long, default_value_t = 1)]
    pub channel: u8,

    /// In auto mode, the gate meter threshold level (e.g. 0.5) to trigger a tap.
    #[arg(short = 't', long, default_value_t = 0.5)]
    pub threshold: f32,
}

pub async fn run(args: Args) -> Result<()> {
    let max_slots = max_fx_slots_for_model(args.model);
    if args.slot < 1 || args.slot > max_slots {
        return Err(anyhow!(
            "FX slot must be between 1 and {} for model {}.",
            max_slots,
            args.model
        ));
    }

    let max_ch = max_channels_for_model(args.model);
    if args.channel < 1 || args.channel > max_ch {
        return Err(anyhow!(
            "Channel must be between 1 and {} for model {}.",
            max_ch,
            args.model
        ));
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let app = Arc::new(Mutex::new(AppState::with_options(
        args.model,
        &args.ip,
        args.slot,
        args.channel,
        args.threshold,
        args.auto,
    )));

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
        // ⚡ Bolt: Avoid unconditionally cloning `state.ip_input` (a String) on every 50ms iteration.
        // We only clone it if it has definitively changed compared to `current_ip`.
        let (new_ip, model, is_auto, channel) = {
            let state = app.lock().unwrap_or_else(|e| e.into_inner());
            let new_ip = if state.ip_input != current_ip {
                Some(state.ip_input.clone())
            } else {
                None
            };
            (new_ip, state.model, state.is_auto, state.channel)
        };

        if let Some(ip) = new_ip {
            current_ip = ip;
            let target_addr_str = model.resolve_target_address(&current_ip);
            if let Ok(parsed_addr) = target_addr_str.parse::<std::net::SocketAddr>() {
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
                                app.lock().unwrap_or_else(|e| e.into_inner()).is_connected = true;
                                app.lock().unwrap_or_else(|e| e.into_inner()).log("Connected!".to_string());
                            } else if msg.path == "/meters/6" {
                                if let Some(OscArg::Blob(data)) = msg.args.first() {
                                    if let Some(level) = AppState::parse_meter_blob(data) {
                                        let maybe_msg = {
                                            let mut state = app.lock().unwrap_or_else(|e| e.into_inner());
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
                                    app.lock().unwrap_or_else(|e| e.into_inner()).delay_type = format!("Type ID: {}", t);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Discard outgoing if not connected
            while rx.try_recv().is_ok() {}
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

#[derive(Default)]
struct LayoutCache {
    area: ratatui::layout::Rect,
    chunks: std::rc::Rc<[ratatui::layout::Rect]>,
    controls_chunks: std::rc::Rc<[ratatui::layout::Rect]>,
    left_chunks: std::rc::Rc<[ratatui::layout::Rect]>,
    right_chunks: std::rc::Rc<[ratatui::layout::Rect]>,
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<AppState>>,
    tx: mpsc::Sender<OscMessage>,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    let mut layout_cache = LayoutCache::default();

    loop {
        let mut app_state = app.lock().unwrap_or_else(|e| e.into_inner());
        app_state.update_display_strings();
        terminal.draw(|f| ui(f, &app_state, &mut layout_cache))?;

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
                            KeyCode::Char('c') => {
                                app_state.active_input = InputMode::EditingChannel
                            }
                            KeyCode::Char('e') => {
                                app_state.active_input = InputMode::EditingSensitivity
                            }
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
                                        let msg =
                                            OscMessage::new(address, vec![OscArg::Float(f_val)]);
                                        let _ = tx.try_send(msg);
                                    } else {
                                        app_state.log("First tap...".to_string());
                                    }
                                }
                            }
                            _ => {}
                        },
                        InputMode::EditingIp => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app_state.active_input = InputMode::Normal
                            }
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
                            KeyCode::Char(c) if c.is_ascii_digit() => app_state.slot_input.push(c),
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
                            KeyCode::Char(c) if c.is_ascii_digit() => app_state.ch_input.push(c),
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
                            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
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

fn ui(f: &mut Frame, app: &AppState, cache: &mut LayoutCache) {
    let size = f.size();

    // ⚡ Bolt: Cache layout chunk splits inside the draw closure to avoid
    // per-frame Layout allocations in the hot render loop.
    if cache.area != size || cache.chunks.is_empty() {
        cache.area = size;

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
            .split(size);
        cache.chunks = chunks;

        if cache.chunks.len() >= 2 {
            let controls_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(cache.chunks[1]);
            cache.controls_chunks = controls_chunks;

            if cache.controls_chunks.len() >= 2 {
                let left_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                    .split(cache.controls_chunks[0]);
                cache.left_chunks = left_chunks;

                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                    .split(cache.controls_chunks[1]);
                cache.right_chunks = right_chunks;
            }
        }
    }

    let chunks = &*cache.chunks;
    let left_chunks = &*cache.left_chunks;
    let right_chunks = &*cache.right_chunks;

    if chunks.len() < 3 || left_chunks.len() < 2 || right_chunks.len() < 2 {
        return;
    }

    // Help block
    let help_msg = match app.active_input {
        InputMode::Normal => {
            "q/Esc: Quit | i: IP | s: Slot | c: Channel | e: Sensitivity | a: Toggle Auto | Enter: Tap"
        }
        _ => "Enter/Esc: Confirm/Stop Editing",
    };
    let help = Paragraph::new(help_msg).block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[0]);

    // IP Input
    // ⚡ Bolt: Cache formatted Strings in AppState instead of allocating
    // per-frame during terminal.draw.
    let ip_style = if app.active_input == InputMode::EditingIp {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let ip_p = Paragraph::new(app.display_ip_text.as_str())
        .style(ip_style)
        .block(Block::default().borders(Borders::ALL).title("Connection"));
    f.render_widget(ip_p, left_chunks[0]);

    // Mode / Settings
    let mode_p = Paragraph::new(app.display_mode_text.as_str())
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(mode_p, left_chunks[1]);

    // Delay Slot
    let slot_style = if app.active_input == InputMode::EditingSlot {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let slot_p = Paragraph::new(app.display_slot_text.as_str())
        .style(slot_style)
        .block(Block::default().borders(Borders::ALL).title("FX Slot"));
    f.render_widget(slot_p, right_chunks[0]);

    // Channel Settings
    let ch_style = if app.active_input == InputMode::EditingChannel {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let ch_line = Line::from(vec![
        Span::raw("Channel: "),
        Span::styled(app.ch_input.as_str(), ch_style),
    ]);

    let sens_style = if app.active_input == InputMode::EditingSensitivity {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let sens_line = Line::from(vec![
        Span::raw("Sens: "),
        Span::styled(app.sens_input.as_str(), sens_style),
    ]);

    let ch_p = Paragraph::new(vec![ch_line, sens_line]).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Auto-Tap Settings"),
    );
    f.render_widget(ch_p, right_chunks[1]);

    // Log window
    // ⚡ Bolt: Use a pre-formatted string stored in AppState instead of
    // mapping and collecting a Vec<Line> on every single frame.
    let log_p = Paragraph::new(app.display_logs_text.as_str())
        .block(Block::default().borders(Borders::ALL).title("Logs"));
    f.render_widget(log_p, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_fx_slots_for_model() {
        assert_eq!(max_fx_slots_for_model(MixerModel::X32), 8);
        assert_eq!(max_fx_slots_for_model(MixerModel::Wing), 16);
        assert_eq!(max_fx_slots_for_model(MixerModel::XR18), 4);
        assert_eq!(max_fx_slots_for_model(MixerModel::XR16), 4);
        assert_eq!(max_fx_slots_for_model(MixerModel::XR12), 4);
    }

    #[test]
    fn test_max_channels_for_model() {
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);
    }

    #[test]
    fn test_args_default_model() {
        let args = Args::parse_from(&["x32_tapw", "--ip", "192.168.1.10"]);
        assert_eq!(args.model, MixerModel::X32);
        assert_eq!(
            args.model.resolve_target_address(&args.ip),
            "192.168.1.10:10023"
        );
    }

    #[test]
    fn test_args_model_parsing() {
        let models = [
            ("X32", MixerModel::X32, "10023"),
            ("Wing", MixerModel::Wing, "2222"),
            ("XR18", MixerModel::XR18, "10024"),
            ("XR16", MixerModel::XR16, "10024"),
            ("XR12", MixerModel::XR12, "10024"),
        ];

        for (model_str, expected_model, expected_port) in models {
            let args_short = Args::parse_from(&["x32_tapw", "-M", model_str, "--ip", "10.0.0.1"]);
            assert_eq!(args_short.model, expected_model);
            assert_eq!(
                args_short.model.resolve_target_address(&args_short.ip),
                format!("10.0.0.1:{}", expected_port)
            );

            let args_long =
                Args::parse_from(&["x32_tapw", "--model", model_str, "--ip", "10.0.0.1"]);
            assert_eq!(args_long.model, expected_model);
        }
    }

    #[test]
    fn test_target_address_with_explicit_port() {
        let args = Args::parse_from(&["x32_tapw", "-M", "Wing", "--ip", "192.168.1.55:9999"]);
        assert_eq!(args.model, MixerModel::Wing);
        assert_eq!(
            args.model.resolve_target_address(&args.ip),
            "192.168.1.55:9999"
        );
    }
}
