use crate::network::ChannelState;

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
    pub main_fader: f32,
    pub main_muted: bool,
    pub alerts: Vec<usize>,
    pub active_alerts: Vec<Alert>,
    pub status: Status,
    pub message: String,
    pub max_fader_limit_db: Option<f32>,
}

#[derive(PartialEq)]
pub enum Status {
    Ok,
    Caution,
    Problem,
}

impl AppState {
    pub fn new(channels: Vec<ChannelState>) -> Self {
        Self {
            channels,
            main_fader: 0.0,
            main_muted: false,
            alerts: vec![],
            active_alerts: vec![],
            status: Status::Ok,
            message: "Starting up...".to_string(),
            max_fader_limit_db: None,
        }
    }
}
