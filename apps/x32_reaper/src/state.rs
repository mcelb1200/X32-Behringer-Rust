use crate::config::Config;

/// Cached state for a single bridged channel/track.
#[derive(Debug, Clone)]
pub struct ChannelState {
    /// Fader level (`0.0..=1.0`).
    pub fader: f32,
    /// Pan position (`0.0..=1.0`).
    pub pan: f32,
    /// Mixbus send levels (`1..=16`).
    pub mixbus: [f32; 16],
    /// Mute state (`0.0` or `1.0`).
    pub mute: f32,
    /// Solo state (`0.0` or `1.0`).
    pub solo: f32,
    /// Scribble-strip label (16 chars max).
    pub scribble: String, // 16 chars max
    /// Scribble-strip color index.
    pub color: i32,
    /// Channel icon index.
    pub icon: i32,
    /// Cached 4-band EQ parameters.
    #[allow(dead_code)]
    pub eq: [f32; 16],
    /// EQ enable flag.
    #[allow(dead_code)]
    pub eq_on: i32,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            fader: 0.0,
            pan: 0.5,
            mixbus: [0.0; 16],
            mute: 0.0,
            solo: 0.0,
            scribble: String::new(),
            color: 0,
            icon: 1,
            eq: [0.0; 16],
            eq_on: 0,
        }
    }
}

/// Thread-safe handle to `AppState`.
#[allow(dead_code)]
pub type SharedState = std::sync::Arc<std::sync::Mutex<AppState>>;

/// Runtime synchronization state for the `x32_reaper` bridge.
pub struct AppState {
    /// Cached state for all banked tracks.
    pub bank_tracks: Vec<ChannelState>,
    /// Currently selected X32 channel index.
    pub x_selected: i32,
    /// Currently selected Reaper track index.
    pub r_selected: i32,
    /// Current channel bank offset.
    pub ch_bank_offset: i32,
    /// Transport loop toggle state.
    pub loop_toggle: i32,
    /// Whether Reaper transport is currently playing.
    pub play: bool,
    /// Whether the X32 shift modifier is active.
    pub shift: bool,
    /// Previous play state for edge detection.
    #[allow(dead_code)]
    pub play_1: bool,
}

impl AppState {
    /// Initializes `AppState` from the loaded `Config`.
    pub fn new(config: &Config) -> Self {
        let mut bank_tracks = Vec::new();
        if config.ch_bank_on {
            let count = if config.trk_max >= config.trk_min {
                let range = config.trk_max - config.trk_min + 1;
                // Round up to multiple of bank_size
                let bk_sz = config.bank_size;
                ((range + bk_sz - 1) / bk_sz) * bk_sz
            } else {
                0
            };

            for _ in 0..count {
                bank_tracks.push(ChannelState::default());
            }
        }

        Self {
            bank_tracks,
            x_selected: 1,
            r_selected: 1,
            ch_bank_offset: config.ch_bank_offset,
            loop_toggle: 0,
            play: false,
            shift: false,
            play_1: false,
        }
    }
}
