/// Runtime state and cached OSC paths for a monitored channel or DCA.
#[derive(Clone, Debug)]
pub struct ChannelState {
    /// Base OSC address prefix (e.g. `/ch/01` or `/dca/1`).
    pub osc_prefix: String, // e.g., "/ch/01" or "/dca/1"
    /// Channel or DCA display name.
    pub name: String,
    /// Current normalized fader position (`0.0..=1.0`).
    pub fader: f32, // 0.0 to 1.0
    /// Whether the channel or DCA is muted.
    pub muted: bool,
    /// Current meter level in dBFS (`-144.0..=0.0`).
    pub level_db: f32, // -144.0 to 0.0
    /// Whether this entry represents a DCA rather than an input channel.
    pub is_dca: bool,
    /// 1-based channel or DCA number.
    pub num: u32,
    /// Cached OSC path for the fader parameter.
    pub fader_path: String,
    /// Cached OSC path for the mute/on parameter.
    pub mute_path: String,
    /// Cached OSC path for the name parameter.
    pub name_path: String,
}

impl ChannelState {
    /// Creates a new `ChannelState` and precomputes its OSC parameter paths.
    pub fn new(osc_prefix: String, is_dca: bool, num: u32) -> Self {
        let fader_path = if is_dca {
            format!("{}/fader", osc_prefix)
        } else {
            format!("{}/mix/fader", osc_prefix)
        };
        let mute_path = if is_dca {
            format!("{}/on", osc_prefix)
        } else {
            format!("{}/mix/on", osc_prefix)
        };
        let name_path = format!("{}/config/name", osc_prefix);

        Self {
            osc_prefix: osc_prefix.clone(),
            name: if is_dca {
                "DCA".to_string()
            } else {
                "CH".to_string()
            },
            fader: 0.0,
            muted: false,
            level_db: -144.0,
            is_dca,
            num,
            fader_path,
            mute_path,
            name_path,
        }
    }
}
