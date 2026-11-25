use crate::state::{AppState, Status};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{fmt::Write, io, time::Duration};
use x32_lib::streaming::StreamStatus;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    cached_area: Rect,
    cached_chunks: Vec<Rect>,
    cached_channel_chunks: Vec<Rect>,
    cached_constraints: Vec<Constraint>,
    header_buf: String,
    fader_bufs: Vec<String>,
    level_bufs: Vec<String>,
    alert_bufs: Vec<String>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            cached_area: Rect::default(),
            cached_chunks: Vec::new(),
            cached_channel_chunks: Vec::new(),
            cached_constraints: Vec::new(),
            header_buf: String::with_capacity(128),
            fader_bufs: Vec::new(),
            level_bufs: Vec::new(),
            alert_bufs: Vec::new(),
        })
    }

    pub fn draw(&mut self, state: &AppState) -> Result<()> {
        if self.cached_constraints.len() != state.channels.len() {
            self.cached_constraints = state
                .channels
                .iter()
                .map(|_| Constraint::Ratio(1, state.channels.len() as u32))
                .collect();
        }

        // ⚡ Bolt: Resize buffers to match dynamic state length using resize_with to ensure
        // each new String actually starts with the requested capacity, preventing allocations.
        if self.fader_bufs.len() != state.channels.len() {
            self.fader_bufs
                .resize_with(state.channels.len(), || String::with_capacity(32));
            self.level_bufs
                .resize_with(state.channels.len(), || String::with_capacity(32));
        }
        if self.alert_bufs.len() != state.active_alerts.len() {
            self.alert_bufs
                .resize_with(state.active_alerts.len(), || String::with_capacity(128));
        }

        // ⚡ Bolt: Clear and populate stateful string buffers using `write!` instead
        // of `format!` to completely eliminate per-frame allocations in the hot render loop.
        self.header_buf.clear();
        let status_color = match state.status {
            Status::Ok => Color::Green,
            Status::Caution => Color::Yellow,
            Status::Problem => Color::Red,
        };
        let status_text = match state.status {
            Status::Ok => "🟢 ALL OK",
            Status::Caution => "🟡 CAUTION",
            Status::Problem => "🔴 PROBLEM",
        };
        let conn_text = if state.last_rx_at.is_none() {
            "CONNECTING"
        } else if state.is_online() {
            "ONLINE"
        } else {
            "OFFLINE"
        };
        let main_mute_text = if state.main_muted { "MUTED" } else { "LIVE" };
        write!(
            self.header_buf,
            "  🎛️  VOLUNTEER ({:?}) | {} | Main: {} ({:.2})",
            state.model, conn_text, main_mute_text, state.main_fader
        )
        .expect("Write to header buffer failed");
        if let Some(ref am_state) = state.automixer {
            let am = if state.automix_enabled {
                if am_state.is_rta_active() {
                    "ON[RTA]"
                } else {
                    "ON"
                }
            } else {
                "OFF"
            };
            write!(self.header_buf, " | Automix: {}", am).expect("Write automix header failed");
        }
        if let Some(st) = state.stream_telemetry {
            let label = match st.status {
                StreamStatus::Silent => "SILENT",
                StreamStatus::TooQuiet => "QUIET",
                StreamStatus::Optimal => "GOOD",
                StreamStatus::Hot => "HOT",
                StreamStatus::Clipping => "CLIP",
            };
            write!(self.header_buf, " | Stream: {} ({:.1}dB)", label, st.rms_db)
                .expect("Write stream header failed");
        }
        write!(self.header_buf, " | {}", status_text).expect("Write status header failed");

        for i in 0..state.channels.len() {
            self.fader_bufs[i].clear();
            write!(self.fader_bufs[i], "Fader: {:.2}", state.channels[i].fader)
                .expect("Write fader buffer failed");
            self.level_bufs[i].clear();
            write!(self.level_bufs[i], "{:.0} dB", state.channels[i].level_db)
                .expect("Write level buffer failed");
        }

        for (i, alert) in state.active_alerts.iter().enumerate() {
            self.alert_bufs[i].clear();
            match alert {
                crate::state::Alert::Clipping {
                    channel_idx,
                    level_db,
                } => {
                    write!(
                        self.alert_bufs[i],
                        "• 🔴 {} signal near clipping ({:.1} dBFS)",
                        state.channels[*channel_idx].name, level_db
                    )
                    .expect("Write alert buffer failed");
                }
                crate::state::Alert::FaderLimit {
                    channel_idx,
                    fader_db,
                    limit,
                } => {
                    write!(
                        self.alert_bufs[i],
                        "• 🟡 {} fader {:.1} dB exceeds limit ({:.1} dB)",
                        state.channels[*channel_idx].name, fader_db, limit
                    )
                    .expect("Write alert buffer failed");
                }
            }
        }

        self.terminal.draw(|f| {
            let size = f.size();

            // ⚡ Bolt: Cache layout chunk splits inside the draw closure to avoid
            // per-frame Layout allocations in the hot render loop. We check if the
            // terminal size changed or if our constraint array size changed.
            if self.cached_area != size
                || self.cached_chunks.is_empty()
                || self.cached_channel_chunks.len() != self.cached_constraints.len()
            {
                self.cached_area = size;

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3), // Header
                            Constraint::Min(10),   // Main View (Channels)
                            Constraint::Length(5), // Alerts
                            Constraint::Length(3), // Footer (Shortcuts)
                        ]
                        .as_ref(),
                    )
                    .split(size);
                self.cached_chunks = chunks.to_vec();

                if self.cached_chunks.len() > 1 {
                    let channel_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(self.cached_constraints.as_slice())
                        .split(self.cached_chunks[1]);
                    self.cached_channel_chunks = channel_chunks.to_vec();
                }
            }

            let chunks = &self.cached_chunks;
            let channel_chunks = &self.cached_channel_chunks;

            if chunks.len() < 4 {
                return;
            }

            // 1. Header
            let header = Paragraph::new(self.header_buf.as_str())
                .style(
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // 2. Channels
            for (i, ch) in state.channels.iter().enumerate() {
                if i < channel_chunks.len() {
                    let level_color = if ch.level_db > -10.0 {
                        Color::Red
                    } else if ch.level_db > -25.0 {
                        Color::Yellow
                    } else {
                        Color::Green
                    };

                    let mute_text = if ch.muted {
                        Span::styled("[MUTED]", Style::default().fg(Color::Red))
                    } else {
                        Span::styled("[LIVE]", Style::default().fg(Color::Green))
                    };

                    let ch_text = vec![
                        Line::from(ch.name.as_str()),
                        Line::from(self.fader_bufs[i].as_str()),
                        Line::from(self.level_bufs[i].as_str()),
                        Line::from(mute_text),
                    ];

                    let border_style = if i == state.selected_idx {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let ch_para = Paragraph::new(ch_text)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(border_style),
                        )
                        .style(Style::default().fg(level_color));

                    f.render_widget(ch_para, channel_chunks[i]);
                }
            }

            // 3. Alerts
            let mut alert_lines = vec![Line::from(Span::styled(
                "ALERTS",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))];
            if state.is_panic_armed() {
                alert_lines.push(Line::from(Span::styled(
                    "⚠️  PANIC ARMED — Press [P] again within 3s to mute & zero all channels, or any key to cancel",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if !state.is_online() {
                alert_lines.push(Line::from(Span::styled(
                    "🔴 Mixer not responding — check network connection or console IP",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
            }
            if let Some(ref msg) = state.check_status_msg {
                alert_lines.push(Line::from(Span::styled(
                    msg.as_str(),
                    Style::default().fg(Color::Cyan),
                )));
            }
            if let Some(st) = state.stream_telemetry {
                match st.status {
                    StreamStatus::Silent => alert_lines.push(Line::from(Span::styled(
                        "🔴 Stream output is SILENT (< -60 dBFS)",
                        Style::default().fg(Color::Red),
                    ))),
                    StreamStatus::Clipping => alert_lines.push(Line::from(Span::styled(
                        "🔴 Stream output CLIPPING — lower stream send/trim",
                        Style::default().fg(Color::Red),
                    ))),
                    StreamStatus::TooQuiet => alert_lines.push(Line::from(Span::styled(
                        "🟡 Stream output too quiet — consider nudging stream feed up",
                        Style::default().fg(Color::Yellow),
                    ))),
                    _ => {}
                }
            }
            for alert_buf in &self.alert_bufs {
                alert_lines.push(Line::from(alert_buf.as_str()));
            }
            if state.alerts.is_empty()
                && !state.is_panic_armed()
                && state.is_online()
                && state.check_status_msg.is_none()
            {
                alert_lines.push(Line::from(Span::styled(
                    "No active alerts",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            let alerts = Paragraph::new(alert_lines).block(Block::default().borders(Borders::ALL));
            f.render_widget(alerts, chunks[2]);

            // 4. Footer
            let footer = Paragraph::new(
                "  [←/→] Sel  [Space] Mute  [↑/↓] Fader  [A]utomix  [C]heck  [U]ndo  [M]uteAll  [P]anic(2x)  [Q]uit",
            )
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;
        Ok(())
    }

    pub fn handle_events(&self) -> Result<Option<UIEvent>> {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(Some(UIEvent::Quit)),
                    KeyCode::Char('m') => return Ok(Some(UIEvent::MuteAll)),
                    KeyCode::Char('p') => return Ok(Some(UIEvent::Panic)),
                    KeyCode::Char('a') => return Ok(Some(UIEvent::ToggleAutomix)),
                    KeyCode::Char('c') => return Ok(Some(UIEvent::PreServiceCheck)),
                    KeyCode::Char('u') => return Ok(Some(UIEvent::Undo)),
                    KeyCode::Left | KeyCode::Char('h') => return Ok(Some(UIEvent::SelectPrev)),
                    KeyCode::Right | KeyCode::Char('l') => return Ok(Some(UIEvent::SelectNext)),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        return Ok(Some(UIEvent::ToggleSelectedMute))
                    }
                    KeyCode::Up | KeyCode::Char('k') => return Ok(Some(UIEvent::FaderUp)),
                    KeyCode::Down | KeyCode::Char('j') => return Ok(Some(UIEvent::FaderDown)),
                    _ => return Ok(Some(UIEvent::CancelConfirm)),
                }
            }
        }
        Ok(None)
    }

    pub fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

pub enum UIEvent {
    Quit,
    MuteAll,
    Panic,
    ToggleAutomix,
    PreServiceCheck,
    Undo,
    SelectPrev,
    SelectNext,
    ToggleSelectedMute,
    FaderUp,
    FaderDown,
    CancelConfirm,
}
