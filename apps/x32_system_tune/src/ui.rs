//! Terminal UI rendering and input handling for `x32_system_tune`.

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
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};
use std::{io, time::Duration};

/// Snapshot of UI state rendered by [`Tui::draw`].
pub struct AppState<'a> {
    /// Name of the active calibration phase.
    pub current_phase: &'a str,
    /// Description of the current step within the phase.
    pub current_step: &'a str,
    /// Summary of the selected measurement microphone profile.
    pub mic_summary: &'a str,
    /// Current oscillator level (`0.0..=1.0`).
    pub level: f32,
    /// Status or prompt message displayed in the main panel.
    pub status_message: &'a str,
}

/// RAII guard that restores terminal raw mode and alternate screen on drop.
pub struct TuiDropGuard;
impl Drop for TuiDropGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

/// Terminal UI controller for `x32_system_tune`.
pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    _guard: TuiDropGuard,
    cached_chunks: std::rc::Rc<[Rect]>,
    cached_area: Rect,
}

impl Tui {
    /// Initializes raw mode, enters the alternate screen, and creates a [`Tui`] instance.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).inspect_err(|_e| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        })?;
        Ok(Self {
            terminal,
            _guard: TuiDropGuard,
            cached_chunks: std::rc::Rc::from([]),
            cached_area: Rect::default(),
        })
    }

    /// Renders the current [`AppState`] to the terminal.
    pub fn draw(&mut self, state: &AppState) -> Result<()> {
        self.terminal.draw(|f| {
            let size = f.size();

            // ⚡ Bolt: Cache layout chunk splits inside the draw closure to avoid
            // an extra ioctl size fetch per frame, eliminating Vec<Rect> allocations.
            if self.cached_area != size {
                self.cached_area = size;
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3), // Title
                            Constraint::Length(6), // Current Phase & Mic
                            Constraint::Length(3), // Level Gauge
                            Constraint::Min(4),    // Status Message
                            Constraint::Length(3), // Footer Hints
                        ]
                        .as_ref(),
                    )
                    .split(size);
                self.cached_chunks = chunks;
            }

            let chunks = &self.cached_chunks;

            let header = Paragraph::new("X32 System Tune - Guided Room & Mic Calibration")
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));

            if chunks.len() >= 5 {
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
                    Line::from(vec![
                        Span::raw("Step:  "),
                        Span::styled(state.current_step, Style::default().fg(Color::White)),
                    ]),
                    Line::from(vec![
                        Span::raw("Mic:   "),
                        Span::styled(state.mic_summary, Style::default().fg(Color::Green)),
                    ]),
                ];
                let phase_para =
                    Paragraph::new(phase_text).block(Block::default().borders(Borders::ALL));
                f.render_widget(phase_para, chunks[1]);

                let gauge = Gauge::default()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Oscillator Level (EBU R128 Capped)"),
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

                let footer = Paragraph::new(
                    "  [↑/↓] Mic  [ [ / ] ] MaxCut  [Space] Sample  [y/n] Apply  [b/t] A/B  [u] Undo  [Enter] Next  [q] Quit",
                )
                .block(Block::default().borders(Borders::ALL));
                f.render_widget(footer, chunks[4]);
            }
        })?;
        Ok(())
    }

    /// Polls for keyboard events and maps them to [`UIEvent`] variants.
    pub fn handle_events(&self) -> Result<Option<UIEvent>> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(Some(UIEvent::Quit)),
                    KeyCode::Char('y') => return Ok(Some(UIEvent::Yes)),
                    KeyCode::Char('n') => return Ok(Some(UIEvent::No)),
                    KeyCode::Enter => return Ok(Some(UIEvent::Next)),
                    KeyCode::Up | KeyCode::Char('k') => return Ok(Some(UIEvent::Up)),
                    KeyCode::Down | KeyCode::Char('j') => return Ok(Some(UIEvent::Down)),
                    KeyCode::Char(' ') => return Ok(Some(UIEvent::Sample)),
                    KeyCode::Char('b') => return Ok(Some(UIEvent::Bypass)),
                    KeyCode::Char('t') => return Ok(Some(UIEvent::Tuned)),
                    KeyCode::Char('u') => return Ok(Some(UIEvent::Undo)),
                    KeyCode::Char('[') => return Ok(Some(UIEvent::TightenGuardrail)),
                    KeyCode::Char(']') => return Ok(Some(UIEvent::RelaxGuardrail)),
                    _ => {}
                }
            }
        }
        Ok(None)
    }
}

/// User input events produced by [`Tui::handle_events`].
pub enum UIEvent {
    /// Exit the application.
    Quit,
    /// Confirm an action (`y`).
    Yes,
    /// Decline an action (`n`).
    No,
    /// Advance to the next step (`Enter`).
    Next,
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Capture an RTA sample (`Space`).
    Sample,
    /// Switch to bypassed EQ (`b`).
    Bypass,
    /// Switch to tuned EQ (`t`).
    Tuned,
    /// Undo the last applied change (`u`).
    Undo,
    /// Tighten the maximum cut guardrail (`[`).
    TightenGuardrail,
    /// Relax the maximum cut guardrail (`]`).
    RelaxGuardrail,
}
