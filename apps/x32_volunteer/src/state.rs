use crate::network::ChannelState;

pub struct AppState {
    pub channels: Vec<ChannelState>,
    pub main_fader: f32,
    pub main_muted: bool,
    pub alerts: Vec<usize>,
    pub fader_alerts: Vec<usize>,
    pub max_fader_limit_db: Option<f32>,
    pub status: Status,
    pub message: String,
}

#[derive(PartialEq, Debug, Clone, Copy)]
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
            fader_alerts: vec![],
            max_fader_limit_db: None,
            status: Status::Ok,
            message: "Starting up...".to_string(),
        }
    }

    pub fn with_fader_limit(mut self, limit_db: Option<f32>) -> Self {
        self.max_fader_limit_db = limit_db;
        self
    }
}
