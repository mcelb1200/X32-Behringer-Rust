use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::fmt::Write;
use std::{io, time::Duration};

use crate::mixer::AppliedNotch;

pub struct AppTui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

pub enum TuiEvent {
    Quit,
    ResetNotches,
    None,
}

impl AppTui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn draw(
        &mut self,
        status: &str,
        notches: &std::collections::HashMap<u8, AppliedNotch>,
    ) -> Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let status_color = if status.contains("Feedback") {
                Color::Red
            } else {
                Color::Green
            };

            let status_p = Paragraph::new(status)
                .block(Block::default().title("Status").borders(Borders::ALL))
                .style(Style::default().fg(status_color));
            f.render_widget(status_p, chunks[0]);

            let mut notch_text = String::new();
            if notches.is_empty() {
                notch_text.push_str("No active notches.");
            } else {
                for (band, notch) in notches {
                    writeln!(
                        notch_text,
                        "Band {}: {:.1} Hz | {:.1} dB",
                        band, notch.frequency, notch.depth
                    )
                    .expect("Failed to write to string buffer");
                }
            }

            let notch_p = Paragraph::new(notch_text).block(
                Block::default()
                    .title("Active Notches")
                    .borders(Borders::ALL),
            );
            f.render_widget(notch_p, chunks[1]);
        })?;
        Ok(())
    }

    pub fn handle_events(&self) -> Result<TuiEvent> {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    return Ok(TuiEvent::Quit);
                } else if key.code == KeyCode::Char('r') {
                    return Ok(TuiEvent::ResetNotches);
                }
            }
        }
        Ok(TuiEvent::None)
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
