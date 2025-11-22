use std::sync::{Arc, Mutex};
use crate::config::Config;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChannelState {
    pub fader: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub scribble_name: String,
    pub scribble_color: i32,
    pub scribble_icon: i32,
    pub mixbus_sends: [f32; 16],
    pub eq: [f32; 16],
    pub eq_on: bool,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            fader: 0.0,
            pan: 0.5,
            mute: false,
            solo: false,
            scribble_name: String::new(),
            scribble_color: 0,
            scribble_icon: 1,
            mixbus_sends: [0.0; 16],
            eq: [0.0; 16],
            eq_on: false,
        }
    }
}

#[allow(dead_code)]
pub struct AppState {
    pub config: Config,
    pub bank_tracks: Vec<ChannelState>, // Dynamically sized based on track range
    pub x_selected: i32,
    pub r_selected: i32,
    pub bank_offset: i32, // Xchbkof
    pub loop_toggle: i32,
    pub play_state: bool,
    pub play_state_change: bool, // play_1
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let track_count = (config.map.track_max - config.map.track_min + 1) as usize;
        // Logic from C: alloc memory for blocks of 32
        // ((Xtrk_max - Xtrk_min + 1 + bkchsz - 1) / bkchsz) * bkchsz
        let bank_size = config.bank.bank_size as usize;
        let alloc_size = track_count.div_ceil(bank_size) * bank_size;

        Self {
            bank_tracks: vec![ChannelState::default(); alloc_size],
            x_selected: 1,
            r_selected: 1,
            bank_offset: config.bank.initial_bank_offset,
            loop_toggle: 0,
            play_state: false,
            play_state_change: false,
            config,
        }
    }
}

pub type SharedState = Arc<Mutex<AppState>>;
