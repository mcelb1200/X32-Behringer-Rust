use crate::network::ChannelState;

pub struct AppState {
    pub channels: Vec<ChannelState>,
    pub main_fader: f32,
    pub main_muted: bool,
    pub alerts: Vec<String>,
    pub status: Status,
    pub message: String,
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
            status: Status::Ok,
            message: "Starting up...".to_string(),
        }
    }
}
