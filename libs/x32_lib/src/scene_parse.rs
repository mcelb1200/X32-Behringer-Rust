use osc_lib::{OscArg, OscMessage};
use x32_fxparse::MixerModel;

// --- FX Type and Source Arrays from C ---
pub const XFXTYP4: &[&str] = &[
    "HALL", "AMBI", "RPLT", "ROOM", "CHAM", "PLAT", "VREV", "VRM", "GATE", "RVRS", "DLY", "3TAP",
    "4TAP", "CRS", "FLNG", "PHAS", "DIMC", "FILT", "ROTA", "PAN", "SUB", "D/RV", "CR/R", "FL/R",
    "D/CR", "D/FL", "MODD", "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES", "P1A", "P1A2", "PQ5",
    "PQ5S", "WAVD", "LIM", "CMB", "CMB2", "FAC", "FAC1M", "FAC2", "LEC", "LEC2", "ULC", "ULC2",
    "ENH2", "ENH", "EXC2", "EXC", "IMG", "EDI", "SON", "AMP2", "AMP", "DRV2", "DRV", "PIT2", "PIT",
];

pub const XFXTYP5: &[&str] = &[
    "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES", "P1A", "P1A2", "PQ5", "PQ5S", "WAVD", "LIM",
    "FAC", "FAC1M", "FAC2", "LEC", "LEC2", "ULC", "ULC2", "ENH2", "ENH", "EXC2", "EXC", "IMG",
    "EDI", "SON", "AMP2", "AMP", "DRV2", "DRV", "PHAS", "FILT", "PAN", "SUB",
];

pub const XFXSRC4: &[&str] = &[
    "INS", "MIX1", "MIX2", "MIX3", "MIX4", "MIX5", "MIX6", "MIX7", "MIX8", "MIX9", "MIX10",
    "MIX11", "MIX12", "MIX13", "MIX14", "MIX15", "MIX16", "M/C",
];

/// A stateful parser for X32/XAir scene files.
/// Maintains internal state, such as currently loaded FX types, to correctly parse
/// effect parameters which have context-dependent types and scales.
#[derive(Debug, Clone)]
pub struct SceneParser {
    pub model: MixerModel,
    /// Tracks the current FX type for slots 1-8. Default initialized to 0 (HALL/GEQ2).
    pub fx_types: [i32; 8],
}

impl Default for SceneParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneParser {
    pub fn new() -> Self {
        Self {
            model: MixerModel::X32,
            fx_types: [0; 8],
        }
    }

    pub fn with_model(model: MixerModel) -> Self {
        Self {
            model,
            fx_types: [0; 8],
        }
    }

    /// Parses a line from an X32/XAir scene file into one or more OSC messages.
    /// Returns an empty vector if the line cannot be parsed.
    pub fn parse_scene_line(&mut self, line: &str) -> Vec<OscMessage> {
        parse_scene_line_internal(self, line)
    }
}

/// Parses a line from an X32/XAir scene file into one or more OSC messages.
///
/// Scene files contain commands like `/ch/01/config/name "MyName"` or `/config/chlink 1-2`.
/// This function translates those shorthand commands into fully-typed OSC messages that can
/// be sent to the mixer (e.g. `/ch/01/config/name ,s "MyName"`).
///
/// If a line cannot be parsed as a known scene command, it returns an empty vector,
/// allowing the caller to attempt parsing it as a raw OSC message.
#[deprecated(note = "Use SceneParser instead for proper FX parameter state tracking")]
pub fn parse_scene_line(line: &str) -> Vec<OscMessage> {
    let mut parser = SceneParser::new();
    parser.parse_scene_line(line)
}

fn parse_scene_line_internal(parser: &mut SceneParser, line: &str) -> Vec<OscMessage> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return vec![];
    }

    let (path, arg_str) = match line.split_once(|c: char| c.is_whitespace()) {
        Some((p, a)) => (p, a.trim()),
        None => (line, ""),
    };
    if arg_str.is_empty() {
        return vec![];
    }

    let mut messages = Vec::new();
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.is_empty() {
        return messages;
    }

    match parts.as_slice() {
        // --- FX Types and Sources ---
        ["fx", slot, "type"] => {
            if let Ok(s) = slot.parse::<usize>() {
                if (1..=8).contains(&s) {
                    let type_val = if s <= 4 {
                        parse_list(arg_str, XFXTYP4)
                    } else {
                        parse_list(arg_str, XFXTYP5)
                    };

                    if let Some(OscArg::Int(t)) = type_val {
                        parser.fx_types[s - 1] = t;
                        messages.push(OscMessage::new(
                            format!("/fx/{}/type", slot),
                            vec![OscArg::Int(t)],
                        ));
                    }
                }
            }
        }
        ["fx", slot, "source", side] => {
            if let Ok(s) = slot.parse::<usize>() {
                if (1..=4).contains(&s) {
                    if let Some(arg) = parse_list(arg_str, XFXSRC4) {
                        messages.push(OscMessage::new(
                            format!("/fx/{}/source/{}", slot, side),
                            vec![arg],
                        ));
                    }
                }
            }
        }
        ["fx", slot, "par"] => {
            if let Ok(s) = slot.parse::<usize>() {
                if (1..=8).contains(&s) {
                    let fx_type = parser.fx_types[s - 1];
                    if let Some(msg) = parse_fx_par(slot, fx_type, arg_str) {
                        messages.push(msg);
                    }
                }
            }
        }

        // --- Config ---
        ["config", "chlink"] => {
            for ch in (1..32).step_by(2) {
                let p = format!("/config/chlink/{}-{}", ch, ch + 1);
                if let Some(arg) = parse_onoff(arg_str, "OFF") {
                    messages.push(OscMessage::new(p, vec![arg]));
                }
            }
        }
        ["config", "auxlink"] => {
            for ch in (1..8).step_by(2) {
                let p = format!("/config/auxlink/{}-{}", ch, ch + 1);
                if let Some(arg) = parse_onoff(arg_str, "OFF") {
                    messages.push(OscMessage::new(p, vec![arg]));
                }
            }
        }
        ["config", "fxlink"] => {
            for ch in (1..8).step_by(2) {
                let p = format!("/config/fxlink/{}-{}", ch, ch + 1);
                if let Some(arg) = parse_onoff(arg_str, "OFF") {
                    messages.push(OscMessage::new(p, vec![arg]));
                }
            }
        }
        ["config", "buslink"] => {
            for ch in (1..16).step_by(2) {
                let p = format!("/config/buslink/{}-{}", ch, ch + 1);
                if let Some(arg) = parse_onoff(arg_str, "OFF") {
                    messages.push(OscMessage::new(p, vec![arg]));
                }
            }
        }
        ["config", "mtxlink"] => {
            for ch in (1..6).step_by(2) {
                let p = format!("/config/mtxlink/{}-{}", ch, ch + 1);
                if let Some(arg) = parse_onoff(arg_str, "OFF") {
                    messages.push(OscMessage::new(p, vec![arg]));
                }
            }
        }
        ["config", "mute"] => {
            for ch in 1..=6 {
                let p = format!("/config/mute/{}", ch);
                if let Some(arg) = parse_onoff(arg_str, "OFF") {
                    messages.push(OscMessage::new(p, vec![arg]));
                }
            }
        }
        ["config", "linkcfg"] => {
            let p_ha = "/config/linkcfg/ha".to_string();
            if let Some(arg) = parse_onoff(arg_str, "OFF") {
                messages.push(OscMessage::new(p_ha, vec![arg.clone()]));
                messages.push(OscMessage::new(
                    "/config/linkcfg/eq".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/linkcfg/dyn".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/linkcfg/fdrmute".to_string(),
                    vec![arg],
                ));
            }
        }
        ["config", "mono"] => {
            if let Some(arg) = parse_onoff(arg_str, "LR+M") {
                messages.push(OscMessage::new("/config/mono/mode".to_string(), vec![arg]));
            }
            if let Some(arg) = parse_onoff(arg_str, "OFF") {
                messages.push(OscMessage::new("/config/mono/link".to_string(), vec![arg]));
            }
        }
        ["config", "solo"] => {
            let p = "/config/solo/source".to_string();
            if let Some(arg) = parse_list(
                arg_str,
                &["OFF", "LR", "LR+C", "LRPFL", "LRAFL", "AUX56", "AUX78"],
            ) {
                messages.push(OscMessage::new(p, vec![arg.clone()]));
                messages.push(OscMessage::new(
                    "/config/solo/level".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/solo/chmode".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/solo/busmode".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/solo/dcamode".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/solo/exclusive".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/solo/followsel".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/solo/followfader".to_string(),
                    vec![arg.clone()],
                ));
                messages.push(OscMessage::new(
                    "/config/solo/dimatt".to_string(),
                    vec![arg],
                ));
            }
        }
        // General fallback handler for all specific channel/bus/etc settings
        _ => {
            if let Some(msg) = x32_fxparse::parse_parameter(parser.model, path, arg_str) {
                messages.push(msg);
            }
        }
    }

    messages
}

// --- Helper parsing functions ---

pub(crate) fn parse_onoff(val: &str, off_val: &str) -> Option<OscArg> {
    if val == off_val {
        Some(OscArg::Int(0))
    } else {
        Some(OscArg::Int(1))
    }
}

pub(crate) fn parse_list(val: &str, list: &[&str]) -> Option<OscArg> {
    list.iter()
        .position(|&s| s == val)
        .map(|i| OscArg::Int(i as i32))
}


#[allow(
    clippy::excessive_precision,
    clippy::manual_range_patterns,
    clippy::approx_constant,
    clippy::needless_range_loop,
    dead_code
)]
pub(crate) fn parse_fx_par(slot: &str, fx_type: i32, args_str: &str) -> Option<OscMessage> {
    x32_fxparse::parse_fx_par(fx_type as usize, args_str, &format!("/fx/{}/par", slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_parse_scene_line_string() {
        let msgs = parse_scene_line("/ch/01/config/name \"Lead Vox\"");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/config/name");
        assert_eq!(msgs[0].args, vec![OscArg::String("Lead Vox".to_string())]);
    }

    #[test]
    #[allow(deprecated)]
    fn test_parse_scene_line_level() {
        let msgs = parse_scene_line("/ch/01/mix/fader 0.5");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/mix/fader");
        assert!(matches!(msgs[0].args[0], OscArg::Float(_)));

        let msgs_oo = parse_scene_line("/ch/01/mix/fader -oo");
        assert_eq!(msgs_oo.len(), 1);
        assert_eq!(msgs_oo[0].args, vec![OscArg::Float(0.0)]);
    }

    #[test]
    #[allow(deprecated)]
    fn test_parse_scene_line_onoff() {
        let msgs = parse_scene_line("/ch/01/mix/on ON");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/mix/on");
        assert_eq!(msgs[0].args, vec![OscArg::Int(1)]);
    }

    #[test]
    #[allow(deprecated)]
    fn test_parse_scene_line_chlink() {
        let msgs = parse_scene_line("/config/chlink ON");
        assert_eq!(msgs.len(), 16);
        assert_eq!(msgs[0].path, "/config/chlink/1-2");
        assert_eq!(msgs[0].args, vec![OscArg::Int(1)]);
        assert_eq!(msgs[15].path, "/config/chlink/31-32");
    }

    #[test]
    #[allow(deprecated)]
    fn test_parse_scene_line_frequency() {
        let msgs = parse_scene_line("/ch/01/eq/1/f 1k2");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/eq/1/f");
        assert!(matches!(msgs[0].args[0], OscArg::Float(_)));
    }

    #[test]
    fn test_parse_fx_par_hall() {
        // HALL (0)
        let msg = parse_fx_par("1", 0, "50 1.5 50 5000 15 0 20 10000 1.0 25 125 50").unwrap();
        assert_eq!(msg.path, "/fx/1/par");
        assert_eq!(msg.args.len(), 12);
        // decay should be log scaled: 3.218895825 = log(5/0.2)
        if let OscArg::Float(f) = msg.args[1] {
            assert!(f >= 0.0 && f <= 1.0);
        }
    }

    #[test]
    fn test_parse_fx_par_ambi() {
        // AMBI (1)
        let msg = parse_fx_par("2", 1, "50 1.5 50 5000 15 0 20 10000 50 50").unwrap();
        assert_eq!(msg.path, "/fx/2/par");
        assert_eq!(msg.args.len(), 10);
    }

    #[test]
    fn test_parse_fx_par_dly() {
        // DLY (10)
        let msg = parse_fx_par("3", 10, "50 500 ST 1 1 0 0 20 10000 20 50 50 10000").unwrap();
        assert_eq!(msg.path, "/fx/3/par");
        assert_eq!(msg.args.len(), 12);
        if let OscArg::Int(i) = msg.args[2] {
            assert_eq!(i, 0); // "ST" index in mode list
        }
    }

    #[test]
    fn test_parse_fx_par_geq() {
        // GEQ (28)
        let msg = parse_fx_par("5", 28, "0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0").unwrap();
        assert_eq!(msg.path, "/fx/5/par");
        assert_eq!(msg.args.len(), 32);
    }

    #[test]
    fn test_parse_fx_par_unmapped() {
        // Some unmapped or out of bounds FX type
        let msg = parse_fx_par("1", 999, "0.5 0.5");
        assert!(msg.is_none());
    }
}
