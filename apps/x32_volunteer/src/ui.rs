use crate::state::{AppState, Status};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{fmt::Write, io, time::Duration};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
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
        if self.alert_bufs.len() != state.alerts.len() {
            self.alert_bufs
                .resize_with(state.alerts.len(), || String::with_capacity(128));
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
        write!(
            self.header_buf,
            "  🎛️  SOUND DESK — Volunteer Mode              {}",
            status_text
        )
        .expect("Write to header buffer failed");

        for i in 0..state.channels.len() {
            self.fader_bufs[i].clear();
            write!(self.fader_bufs[i], "Fader: {:.2}", state.channels[i].fader)
                .expect("Write fader buffer failed");
            self.level_bufs[i].clear();
            write!(self.level_bufs[i], "{:.0} dB", state.channels[i].level_db)
                .expect("Write level buffer failed");
        }

        for i in 0..state.alerts.len() {
            self.alert_bufs[i].clear();
            write!(
                self.alert_bufs[i],
                "• 🟡 {} level is high — consider lowering fader.",
                state.channels[state.alerts[i]].name
            )
            .expect("Write alert buffer failed");
        }

        self.terminal.draw(|f| {
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
                .split(f.size());

            // 1. Header
            let header = Paragraph::new(self.header_buf.as_str())
                .style(
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // 2. Channels (Grid layout ideally, simplify for now to horizontal chunks)
            let channel_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(self.cached_constraints.as_slice())
                .split(chunks[1]);

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

                    let ch_para = Paragraph::new(ch_text)
                        .block(Block::default().borders(Borders::ALL))
                        .style(Style::default().fg(level_color));

                    f.render_widget(ch_para, channel_chunks[i]);
                }
            }

            // 3. Alerts
            let mut alert_lines = vec![Line::from(Span::styled(
                "ALERTS",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))];
            for alert_buf in &self.alert_bufs {
                alert_lines.push(Line::from(alert_buf.as_str()));
            }
            if state.alerts.is_empty() {
                alert_lines.push(Line::from(Span::styled(
                    "No active alerts",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            let alerts = Paragraph::new(alert_lines).block(Block::default().borders(Borders::ALL));
            f.render_widget(alerts, chunks[2]);

            // 4. Footer
            let footer = Paragraph::new("  [M]ute all  [P]anic  [Q]uit")
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
                    _ => {}
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
}
