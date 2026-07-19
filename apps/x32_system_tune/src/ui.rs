use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use std::{io, time::Duration};

pub struct AppState<'a> {
    pub current_phase: &'a str,
    pub current_step: String,
    pub level: f32,
    pub status_message: &'a str,
}

pub struct TuiDropGuard;
impl Drop for TuiDropGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    _guard: TuiDropGuard,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).inspect_err(|_e| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
        })?;
        Ok(Self {
            terminal,
            _guard: TuiDropGuard,
        })
    }

    pub fn draw(&mut self, state: &AppState) -> Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3), // Title
                        Constraint::Length(5), // Current Phase
                        Constraint::Length(3), // Level Gauge
                        Constraint::Min(3),    // Status Message
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let header = Paragraph::new("X32 System Tune - Oscillator-Assisted Tuning")
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            let phase_text = vec![
                Line::from(vec![
                    Span::raw("Phase: "),
                    Span::styled(
                        state.current_phase,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Step: "),
                    Span::styled(&state.current_step, Style::default().fg(Color::White)),
                ]),
            ];
            let phase_para =
                Paragraph::new(phase_text).block(Block::default().borders(Borders::ALL));
            f.render_widget(phase_para, chunks[1]);

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Oscillator Level"),
                )
                .gauge_style(Style::default().fg(Color::Green))
                .ratio(state.level.clamp(0.0, 1.0) as f64);
            f.render_widget(gauge, chunks[2]);

            let status = Paragraph::new(state.status_message).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Status / Action"),
            );
            f.render_widget(status, chunks[3]);
        })?;
        Ok(())
    }

    pub fn handle_events(&self) -> Result<Option<UIEvent>> {
        // Drain events to avoid backing up the queue if mouse is moved or keys mashed
        let mut last_event = None;
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => last_event = Some(UIEvent::Quit),
                    KeyCode::Char('y') => last_event = Some(UIEvent::Yes),
                    KeyCode::Char('n') => last_event = Some(UIEvent::No),
                    KeyCode::Enter => last_event = Some(UIEvent::Next),
                    _ => {}
                }
            } else {
                let _ = event::read()?; // drop other events like resize/mouse
            }
        }

        Ok(last_event)
    }
}

pub enum UIEvent {
    Quit,
    Yes,
    No,
    Next,
}
