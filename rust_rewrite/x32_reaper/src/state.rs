//! # State Module
//!
//! Defines the data structures for the application's state.

use serde::{Deserialize, Serialize};
use crate::config::Config;

/// Holds the state of a single track.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackState {
    /// Fader level (0.0 to 1.0).
    pub fader: f32,
    /// Pan position (-1.0 to 1.0).
    pub pan: f32,
    /// Mix bus send levels.
    pub mixbus: [f32; 16],
    /// Mute state.
    pub mute: bool,
    /// Solo state.
    pub solo: bool,
    /// Scribble strip text.
    pub scribble: String,
    /// Scribble strip color.
    pub color: u8,
    /// Scribble strip icon.
    pub icon: u8,
    /// EQ parameters.
    pub eq: [f32; 16],
    /// EQ on/off state.
    pub eq_on: bool,
}

impl Default for TrackState {
    fn default() -> Self {
        Self {
            fader: 0.0,
            pan: 0.5,
            mixbus: [0.0; 16],
            mute: false,
            solo: false,
            scribble: String::new(),
            color: 0,
            icon: 1,
            eq: [0.0; 16],
            eq_on: false,
        }
    }
}

/// Holds the overall state of the application.
#[derive(Debug)]
pub struct AppState {
    /// The state of all tracks.
    pub track_states: Vec<TrackState>,
    /// The current channel bank offset.
    pub channel_bank_offset: u8,
    /// The currently selected channel on the X32.
    pub x32_selected_channel: u8,
    /// The currently selected track in Reaper.
    pub reaper_selected_track: u16,
}

impl AppState {
    /// Creates a new `AppState`.
    pub fn new(config: &Config) -> Self {
        let num_tracks = (config.channel_map.max - config.channel_map.min + 1) as usize;
        let bank_size = config.bank_size as usize;
        let total_tracks = ((num_tracks + bank_size - 1) / bank_size) * bank_size;

        Self {
            track_states: vec![TrackState::default(); total_tracks],
            channel_bank_offset: 0,
            x32_selected_channel: 1,
            reaper_selected_track: 1,
        }
    }
}
