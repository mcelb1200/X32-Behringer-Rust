#[derive(Clone, Debug)]
pub struct ChannelState {
    pub osc_prefix: String, // e.g., "/ch/01" or "/dca/1"
    pub name: String,
    pub fader: f32, // 0.0 to 1.0
    pub muted: bool,
    pub level_db: f32, // -144.0 to 0.0
    pub is_dca: bool,
    pub num: u32,
    pub fader_path: String,
    pub mute_path: String,
    pub name_path: String,
}

impl ChannelState {
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
