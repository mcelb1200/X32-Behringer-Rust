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
use std::{io, time::Duration};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn draw(&mut self, state: &AppState) -> Result<()> {
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

            let header_text = format!("  🎛️  SOUND DESK — Volunteer Mode              {}", status_text);
            let header = Paragraph::new(header_text)
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
                .constraints(
                    state.channels.iter().map(|_| Constraint::Ratio(1, state.channels.len() as u32)).collect::<Vec<_>>()
                )
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
                        Line::from(ch.name.clone()),
                        Line::from(format!("Fader: {:.2}", ch.fader)),
                        Line::from(format!("{:.0} dB", ch.level_db)),
                        Line::from(mute_text),
                    ];

                    let ch_para = Paragraph::new(ch_text)
                        .block(Block::default().borders(Borders::ALL))
                        .style(Style::default().fg(level_color));

                    f.render_widget(ch_para, channel_chunks[i]);
                }
            }

            // 3. Alerts
            let mut alert_lines = vec![Line::from(Span::styled("ALERTS", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)))];
            for alert in &state.alerts {
                alert_lines.push(Line::from(format!("• {}", alert)));
            }
            if state.alerts.is_empty() {
                alert_lines.push(Line::from(Span::styled("No active alerts", Style::default().fg(Color::DarkGray))));
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
