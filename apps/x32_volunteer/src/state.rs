use crate::network::ChannelState;
use std::time::Instant;

pub enum Alert {
    Clipping {
        channel_idx: usize,
        level_db: f32,
    },
    FaderLimit {
        channel_idx: usize,
        fader_db: f32,
        limit: f32,
    },
}

pub struct AppState {
    pub channels: Vec<ChannelState>,
    pub selected_idx: usize,
    pub main_fader: f32,
    pub main_muted: bool,
    pub alerts: Vec<usize>,
    pub active_alerts: Vec<Alert>,
    pub status: Status,
    pub message: String,
    pub max_fader_limit_db: Option<f32>,
    pub confirm_panic_at: Option<Instant>,
    pub last_rx_at: Option<Instant>,
    pub started_at: Instant,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Ok,
    Caution,
    Problem,
}

impl AppState {
    pub fn new(channels: Vec<ChannelState>) -> Self {
        Self {
            channels,
            selected_idx: 0,
            main_fader: 0.0,
            main_muted: false,
            alerts: vec![],
            active_alerts: vec![],
            status: Status::Ok,
            message: "Starting up...".to_string(),
            max_fader_limit_db: None,
            confirm_panic_at: None,
            last_rx_at: None,
            started_at: Instant::now(),
        }
    }

    /// Returns true if a panic confirmation prompt is currently active (within 3 seconds).
    pub fn is_panic_armed(&self) -> bool {
        self.confirm_panic_at
            .map(|t| t.elapsed().as_secs() < 3)
            .unwrap_or(false)
    }

    /// Returns true if the console has responded recently (within 3 seconds), or still in initial grace window.
    pub fn is_online(&self) -> bool {
        match self.last_rx_at {
            Some(t) => t.elapsed().as_secs() < 3,
            None => self.started_at.elapsed().as_secs() < 3,
        }
    }
}
