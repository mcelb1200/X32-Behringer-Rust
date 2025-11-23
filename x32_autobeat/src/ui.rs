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
    widgets::{Block, Borders, Gauge, Paragraph},
};
use std::{io, time::Duration};

pub struct AppState {
    pub current_bpm: Option<f32>,
    pub input_level: f32,
    pub active_effect: String,
    pub subdivision: String,
    pub is_fallback: bool,
    pub is_panic: bool,
    pub message: String,
    pub algorithm: String,
}

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
                        Constraint::Percentage(20), // Header
                        Constraint::Percentage(40), // BPM Big
                        Constraint::Percentage(20), // Info
                        Constraint::Percentage(20), // Status
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Header
            let header = Block::default()
                .title("X32 AutoBeat (Press 'a' to switch algorithm)")
                .borders(Borders::ALL);
            f.render_widget(header, chunks[0]);

            // BPM Display
            // Fix lifetime issue by creating String first
            let bpm_string;
            let bpm_text = if state.is_panic {
                "PANIC"
            } else if let Some(bpm) = state.current_bpm {
                bpm_string = format!("{:.1} BPM", bpm);
                &bpm_string
            } else {
                "Detecting..."
            };

            let bpm_color = if state.is_panic {
                Color::Red
            } else {
                Color::Green
            };
            let bpm_para = Paragraph::new(bpm_text)
                .style(Style::default().fg(bpm_color).add_modifier(Modifier::BOLD))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Tempo ({})", state.algorithm)),
                )
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(bpm_para, chunks[1]);

            // Input Level Gauge
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Input Level"))
                .gauge_style(Style::default().fg(Color::Cyan))
                .ratio(state.input_level.clamp(0.0, 1.0) as f64);
            f.render_widget(gauge, chunks[2]);

            // Info / Status
            let mode_str = if state.is_fallback {
                "Fallback (OSC)"
            } else {
                "Primary (Audio)"
            };
            let info_text = vec![
                Line::from(vec![
                    Span::raw("Mode: "),
                    Span::styled(
                        mode_str,
                        Style::default().fg(if state.is_fallback {
                            Color::Yellow
                        } else {
                            Color::Blue
                        }),
                    ),
                ]),
                Line::from(format!("Effect: {}", state.active_effect)),
                Line::from(format!("Subdiv: {}", state.subdivision)),
                Line::from(format!("Algorithm: {}", state.algorithm)),
                Line::from(format!("Msg: {}", state.message)),
            ];
            let info = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(info, chunks[3]);
        })?;
        Ok(())
    }

    pub fn handle_events(&self) -> Result<Option<UIEvent>> {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(Some(UIEvent::Panic)),
                    KeyCode::Char('q') => return Ok(Some(UIEvent::Quit)),
                    KeyCode::Char('r') => return Ok(Some(UIEvent::Reset)),
                    KeyCode::Char('a') => return Ok(Some(UIEvent::SwitchAlgorithm)),
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
    Panic,
    Quit,
    Reset,
    SwitchAlgorithm,
}
