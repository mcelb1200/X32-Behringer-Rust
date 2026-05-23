use crate::effects::EffectConfig;
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
    widgets::{Block, Borders, Gauge, Paragraph, Tabs},
};
use std::{io, time::Duration};

pub struct AppState {
    pub current_bpm: Option<f32>,
    pub input_level: f32,
    pub active_effects: [String; 8], // Names of loaded effects
    pub effect_configs: [EffectConfig; 8],
    pub selected_slot: usize, // 0-7 (FX 1-8)
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
                        Constraint::Length(3), // Title
                        Constraint::Length(3), // Slots Tabs
                        Constraint::Min(5),    // Active Slot Details
                        Constraint::Length(3), // BPM
                        Constraint::Length(3), // Input Gauge
                        Constraint::Length(3), // Status
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // 1. Title
            let header = Paragraph::new("X32 AutoBeat - Multi-FX Control")
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // 2. Slot Tabs
            let titles: Vec<Line> = state
                .active_effects
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let style = if i == state.selected_slot {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let short_name = if name.is_empty() { "EMPTY" } else { name };
                    Line::styled(format!("FX{}: {}", i + 1, short_name), style)
                })
                .collect();

            let tabs = Tabs::new(titles)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Effect Slots (Use Left/Right to Select)"),
                )
                .select(state.selected_slot)
                .highlight_style(Style::default().fg(Color::Cyan));
            f.render_widget(tabs, chunks[1]);

            // 3. Active Slot Details
            let cfg = &state.effect_configs[state.selected_slot];
            let detail_text = vec![
                Line::from(vec![
                    Span::raw("Current Effect: "),
                    Span::styled(
                        &state.active_effects[state.selected_slot],
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Subdivision (Up/Down): "),
                    Span::styled(&cfg.subdivision, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("Style (PgUp/PgDn):     "),
                    Span::styled(&cfg.style, Style::default().fg(Color::Magenta)),
                ]),
                Line::from(vec![
                    Span::raw("Status:                "),
                    Span::styled(
                        if cfg.enabled { "SYNCED" } else { "BYPASS" },
                        Style::default().fg(if cfg.enabled {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                    ),
                ]),
            ];
            let details = Paragraph::new(detail_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Slot Configuration"),
            );
            f.render_widget(details, chunks[2]);

            // 4. BPM Display
            let bpm_string;
            let bpm_text = if state.is_panic {
                "PANIC"
            } else if let Some(bpm) = state.current_bpm {
                bpm_string = format!("{:.1} BPM ({})", bpm, state.algorithm);
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
                .block(Block::default().borders(Borders::ALL).title("Global Tempo"));
            f.render_widget(bpm_para, chunks[3]);

            // 5. Input Level Gauge
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Input Level"))
                .gauge_style(Style::default().fg(Color::Cyan))
                .ratio(state.input_level.clamp(0.0, 1.0) as f64);
            f.render_widget(gauge, chunks[4]);

            // 6. Status Bar
            let status_text = format!(
                "Mode: {} | Msg: {} | Controls: Arrow Keys, PgUp/Dn, 'a'lgo, 'r'eset, 'q'uit",
                if state.is_fallback {
                    "OSC Fallback"
                } else {
                    "Audio"
                },
                state.message
            );
            let status = Paragraph::new(status_text).block(Block::default().borders(Borders::ALL));
            f.render_widget(status, chunks[5]);
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
                    KeyCode::Right => return Ok(Some(UIEvent::NextSlot)),
                    KeyCode::Left => return Ok(Some(UIEvent::PrevSlot)),
                    KeyCode::Up => return Ok(Some(UIEvent::NextSubdiv)),
                    KeyCode::Down => return Ok(Some(UIEvent::PrevSubdiv)),
                    KeyCode::PageUp => return Ok(Some(UIEvent::NextStyle)),
                    KeyCode::PageDown => return Ok(Some(UIEvent::PrevStyle)),
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
    NextSlot,
    PrevSlot,
    NextSubdiv,
    PrevSubdiv,
    NextStyle,
    PrevStyle,
}
