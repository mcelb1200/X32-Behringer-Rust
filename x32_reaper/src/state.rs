use crate::config::Config;

#[derive(Debug, Clone)]
pub struct ChannelState {
    pub fader: f32,
    pub pan: f32,
    pub mixbus: [f32; 16],
    pub mute: f32,
    pub solo: f32,
    pub scribble: String, // 16 chars max
    pub color: i32,
    pub icon: i32,
    pub eq: [f32; 16],
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

pub struct AppState {
    pub bank_tracks: Vec<ChannelState>,
    pub x_selected: i32,
    pub r_selected: i32,
    pub ch_bank_offset: i32,
    pub loop_toggle: i32,
    pub play: bool,
    pub play_1: bool,
}

impl AppState {
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
            play_1: false,
        }
    }
}
