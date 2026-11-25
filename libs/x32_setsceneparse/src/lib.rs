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
    let mut parts_array = [""; 16];
    let mut parts_len = 0;
    for p in path.trim_start_matches('/').split('/') {
        if parts_len < 16 {
            parts_array[parts_len] = p;
            parts_len += 1;
        }
    }
    let parts = &parts_array[..parts_len];
    if parts.is_empty() {
        return messages;
    }

    match parts {
        // --- FX Types and Sources ---
        ["fx", slot, "type"] => {
            let max_fx = match parser.model {
                MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
                _ => 8,
            };
            if let Ok(s) = slot.parse::<usize>() {
                if (1..=max_fx).contains(&s) {
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
            let max_fx = match parser.model {
                MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
                _ => 8,
            };
            if let Ok(s) = slot.parse::<usize>() {
                if (1..=max_fx).contains(&s) {
                    let fx_type = parser.fx_types[s - 1];
                    if let Some(msg) = parse_fx_par(slot, fx_type, arg_str) {
                        messages.push(msg);
                    }
                }
            }
        }

        // --- X-Air Specific Paths (/lr, /rtn, /fxsend, /headamp) ---
        ["lr", rest @ ..] => {
            if let Some(msg) = parse_xair_strip_path(path, rest, arg_str, 6) {
                messages.push(msg);
            }
        }
        ["rtn", "aux", rest @ ..] => {
            if let Some(msg) = parse_xair_strip_path(path, rest, arg_str, 4) {
                messages.push(msg);
            }
        }
        ["rtn", slot, rest @ ..] => {
            if let Ok(s) = slot.parse::<usize>() {
                if (1..=4).contains(&s) {
                    if let Some(msg) = parse_xair_strip_path(path, rest, arg_str, 4) {
                        messages.push(msg);
                    }
                }
            }
        }
        ["fxsend", slot, rest @ ..] => {
            if let Ok(s) = slot.parse::<usize>() {
                if (1..=4).contains(&s) {
                    if let Some(msg) = parse_xair_strip_path(path, rest, arg_str, 0) {
                        messages.push(msg);
                    }
                }
            }
        }
        ["headamp", idx, param] => {
            let max_ha = match parser.model {
                MixerModel::XR12 => 12,
                MixerModel::XR16 | MixerModel::XR18 => 16,
                _ => 128,
            };
            if let Ok(h) = idx.parse::<usize>() {
                if (1..=max_ha).contains(&h) {
                    let arg = match *param {
                        "gain" => parse_linf_arg(arg_str, -12.0, 60.0),
                        "phantom" => parse_onoff(arg_str, "OFF"),
                        _ => None,
                    };
                    if let Some(a) = arg {
                        messages.push(OscMessage::new(path.to_string(), vec![a]));
                    }
                }
            }
        }
        ["routing", bus_type, _idx, param] => {
            let arg = match (*bus_type, *param) {
                ("main", "src") => parse_list(arg_str, XAIR_MAIN_SRC),
                ("aux" | "p16", "src") => parse_list(arg_str, XAIR_AUX_SRC),
                ("usb", "src") => parse_list(arg_str, XAIR_USB_SRC),
                ("aux" | "p16" | "usb", "pos") => parse_list(arg_str, XAIR_SRC_POS),
                _ => None,
            };
            if let Some(a) = arg {
                messages.push(OscMessage::new(path.to_string(), vec![a]));
            }
        }

        // --- Config ---
        ["config", "chlink"] => {
            let max_ch = match parser.model {
                MixerModel::XR12 => 12,
                MixerModel::XR16 | MixerModel::XR18 => 16,
                MixerModel::Wing => 40,
                MixerModel::X32 => 32,
            };
            for ch in (1..max_ch).step_by(2) {
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
            let max_bus = match parser.model {
                MixerModel::XR12 => 2,
                MixerModel::XR16 => 4,
                MixerModel::XR18 => 6,
                MixerModel::Wing => 28,
                MixerModel::X32 => 16,
            };
            for ch in (1..max_bus).step_by(2) {
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
            let max_mute = match parser.model {
                MixerModel::XR12 | MixerModel::XR16 | MixerModel::XR18 => 4,
                _ => 6,
            };
            for ch in 1..=max_mute {
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

const XAIR_EQ_TYPES: &[&str] = &["LCut", "LShv", "PEQ", "VEQ", "HShv", "HCut"];
const XAIR_DYN_RATIOS: &[&str] = &[
    "1.1", "1.3", "1.5", "2.0", "2.5", "3.0", "4.0", "5.0", "7.0", "10", "20", "100",
];
const XAIR_MIX_TAPS: &[&str] = &["IN", "PREEQ", "POSTEQ", "PRE", "POST", "GRP"];
const XAIR_RTN_SRC: &[&str] = &[
    "U0102", "U0304", "U0506", "U0708", "U0910", "U1112", "U1314", "U1516", "U1718",
];
const XAIR_MAIN_SRC: &[&str] = &[
    "LR", "MON", "U0102", "U0304", "U0506", "U0708", "U0910", "U1112", "U1314", "U1516", "U1718",
];
const XAIR_AUX_SRC: &[&str] = &[
    "Ch01", "Ch02", "Ch03", "Ch04", "Ch05", "Ch06", "Ch07", "Ch08", "Ch09", "Ch10", "Ch11", "Ch12",
    "Ch13", "Ch14", "Ch15", "Ch16", "AuxL", "AuxR", "Fx1L", "Fx1R", "Fx2L", "Fx2R", "Fx3L", "Fx3R",
    "Fx4L", "Fx4R", "Bus1", "Bus2", "Bus3", "Bus4", "Bus5", "Bus6", "Send1", "Send2", "Send3",
    "Send4", "L", "R", "U01", "U02", "U03", "U04", "U05", "U06", "U07", "U08", "U09", "U10", "U11",
    "U12", "U13", "U14", "U15", "U16", "U17", "U18",
];
const XAIR_USB_SRC: &[&str] = &[
    "Ch01", "Ch02", "Ch03", "Ch04", "Ch05", "Ch06", "Ch07", "Ch08", "Ch09", "Ch10", "Ch11", "Ch12",
    "Ch13", "Ch14", "Ch15", "Ch16", "AuxL", "AuxR", "Fx1L", "Fx1R", "Fx2L", "Fx2R", "Fx3L", "Fx3R",
    "Fx4L", "Fx4R", "Bus1", "Bus2", "Bus3", "Bus4", "Bus5", "Bus6", "Send1", "Send2", "Send3",
    "Send4", "L", "R",
];
const XAIR_SRC_POS: &[&str] = &[
    "AIN", "AIN+M", "IN", "IN+M", "PREEQ", "PREEQ+M", "POSTEQ", "POSTEQ+M", "PRE", "PRE+M", "POST",
];

fn parse_linf_arg(val: &str, min: f32, max: f32) -> Option<OscArg> {
    let f = val.trim().parse::<f32>().ok()?;
    Some(OscArg::Float(((f - min) / (max - min)).clamp(0.0, 1.0)))
}

fn parse_level_arg(val: &str) -> Option<OscArg> {
    let v = val.trim();
    if v == "-oo" {
        return Some(OscArg::Float(0.0));
    }
    let fval = v.parse::<f32>().ok()?;
    let scaled = if fval < -60.0 {
        (fval * 0.002_083_333_4 + 0.1875).max(0.0)
    } else if fval < -30.0 {
        0.00625 * fval + 0.4375
    } else if fval < -10.0 {
        0.0125 * fval + 0.625
    } else {
        (fval * 0.025 + 0.75).min(1.0)
    };
    Some(OscArg::Float(scaled.clamp(0.0, 1.0)))
}

fn parse_bits_arg(val: &str) -> Option<OscArg> {
    let v = val.trim().strip_prefix('%')?;
    let mut res = 0i32;
    for (i, c) in v.chars().rev().enumerate() {
        if c == '1' {
            res |= 1 << i;
        }
    }
    Some(OscArg::Int(res))
}

/// Parses X-Air-specific strip parameters (`/lr/...`, `/rtn/aux/...`, `/rtn/1..4/...`, `/fxsend/1..4/...`)
/// matching `XAirSetSceneParse.c`.
fn parse_xair_strip_path(
    full_path: &str,
    rest: &[&str],
    arg_str: &str,
    max_eq_bands: usize,
) -> Option<OscMessage> {
    let arg = match rest {
        ["config", "name"] => {
            let v = arg_str.trim();
            let stripped = v
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(v);
            Some(OscArg::String(stripped.to_string()))
        }
        ["config", "color"] => arg_str.trim().parse::<i32>().ok().map(OscArg::Int),
        ["config", "rtnsrc"] => parse_list(arg_str, XAIR_RTN_SRC),
        ["preamp", "rtntrim"] => parse_linf_arg(arg_str, -18.0, 18.0),
        ["preamp", "rtnsw"] => parse_onoff(arg_str, "OFF"),
        ["eq", "on"] => parse_onoff(arg_str, "OFF"),
        ["eq", band, param] => {
            let b = band.parse::<usize>().ok()?;
            if b < 1 || b > max_eq_bands {
                return None;
            }
            match *param {
                "type" => parse_list(arg_str, XAIR_EQ_TYPES),
                "f" => {
                    // Delegate frequency parsing via x32_fxparse
                    return x32_fxparse::parse_parameter(
                        MixerModel::XR18,
                        &format!("/ch/01/eq/1/f"),
                        arg_str,
                    )
                    .map(|m| OscMessage::new(full_path.to_string(), m.args));
                }
                "g" => parse_linf_arg(arg_str, -15.0, 15.0),
                "q" => {
                    let f = arg_str.trim().parse::<f32>().ok()?;
                    let norm = ((f / 10.0).ln() / -3.506_558).clamp(0.0, 1.0);
                    Some(OscArg::Float(norm))
                }
                _ => None,
            }
        }
        ["dyn", "on"] => parse_onoff(arg_str, "OFF"),
        ["dyn", "mode"] => parse_onoff(arg_str, "COMP"),
        ["dyn", "det"] => parse_onoff(arg_str, "PEAK"),
        ["dyn", "env"] => parse_onoff(arg_str, "LIN"),
        ["dyn", "thr"] => parse_linf_arg(arg_str, -60.0, 0.0),
        ["dyn", "ratio"] => parse_list(arg_str, XAIR_DYN_RATIOS),
        ["dyn", "knee"] => parse_linf_arg(arg_str, 0.0, 5.0),
        ["dyn", "mgain"] => parse_linf_arg(arg_str, 0.0, 24.0),
        ["dyn", "attack"] => parse_linf_arg(arg_str, 0.0, 120.0),
        ["dyn", "hold"] => {
            let f = arg_str.trim().parse::<f32>().ok()?;
            Some(OscArg::Float(
                ((f / 0.02).ln() / 11.512_925).clamp(0.0, 1.0),
            ))
        }
        ["dyn", "release"] => {
            let f = arg_str.trim().parse::<f32>().ok()?;
            Some(OscArg::Float(((f / 5.0).ln() / 6.684_612).clamp(0.0, 1.0)))
        }
        ["dyn", "mix"] => parse_linf_arg(arg_str, 0.0, 100.0),
        ["mix", "on"] | ["mix", "lr"] => parse_onoff(arg_str, "OFF"),
        ["mix", "fader"] => parse_level_arg(arg_str),
        ["mix", "pan"] => parse_linf_arg(arg_str, -100.0, 100.0),
        ["mix", _, "level"] => parse_level_arg(arg_str),
        ["mix", _, "grpon"] => parse_onoff(arg_str, "OFF"),
        ["mix", _, "tap"] => parse_list(arg_str, XAIR_MIX_TAPS),
        ["mix", _, "pan"] => parse_linf_arg(arg_str, -100.0, 100.0),
        ["grp", "dca"] | ["grp", "mute"] => parse_bits_arg(arg_str),
        _ => None,
    }?;
    Some(OscMessage::new(full_path.to_string(), vec![arg]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_parser_basic_line() {
        let mut parser = SceneParser::new();
        let msgs = parser.parse_scene_line("/ch/01/config/name \"Lead Vox\"");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/config/name");
        assert_eq!(msgs[0].args, vec![OscArg::String("Lead Vox".to_string())]);
    }

    #[test]
    fn test_scene_parser_stateful_fx() {
        let mut parser = SceneParser::new();
        // Set FX 1 type to AMBI
        let msgs = parser.parse_scene_line("/fx/1/type AMBI");
        assert_eq!(msgs.len(), 1);
        assert_eq!(parser.fx_types[0], 1); // AMBI is index 1 in XFXTYP4

        // Now parse parameters for FX 1
        let msgs = parser.parse_scene_line("/fx/1/par 50 1.5 50 5000 15 0 20 10000 50 50");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/fx/1/par");
        assert_eq!(msgs[0].args.len(), 10); // AMBI has 10 parameters
    }

    #[test]
    fn test_scene_parser_empty_and_invalid() {
        let mut parser = SceneParser::new();
        let msgs = parser.parse_scene_line("");
        assert!(msgs.is_empty());

        let msgs = parser.parse_scene_line("   ");
        assert!(msgs.is_empty());

        let msgs = parser.parse_scene_line("# This is a comment");
        assert!(msgs.is_empty());

        let msgs = parser.parse_scene_line("invalid_line_without_space");
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_scene_parser_whitebox_out_of_bounds() {
        let mut parser = SceneParser::new();

        // Out of bounds channel (not 1..=32)
        let msgs = parser.parse_scene_line("/ch/99/config/name \"Lead Vox\"");
        assert!(msgs.is_empty(), "Should ignore channel 99");

        // Out of bounds FX slot for source (1..=4)
        let msgs = parser.parse_scene_line("/fx/5/source/L MIX1");
        assert!(msgs.is_empty(), "Should ignore FX slot 5 for source");

        // Unparseable channel ID (not an integer)
        let msgs = parser.parse_scene_line("/ch/abc/config/name \"Lead Vox\"");
        assert!(
            msgs.is_empty(),
            "Should gracefully handle non-integer channel IDs"
        );

        // Invalid FX source enum argument
        let msgs = parser.parse_scene_line("/fx/1/source/L INVALID_SRC");
        assert!(msgs.is_empty(), "Should ignore invalid enum values");
    }

    #[test]
    fn test_scene_parser_whitebox_limits() {
        let mut parser = SceneParser::new();

        // Path with more than 16 segments. Only first 16 are parsed.
        // Let's ensure it doesn't panic.
        let long_path = format!("/{}/value 123", "a/".repeat(20));
        let msgs = parser.parse_scene_line(&long_path);
        // It won't match any known patterns, so it returns empty without panic
        assert!(msgs.is_empty());

        // Empty parts after trim
        let msgs = parser.parse_scene_line("/ 123");
        assert!(
            msgs.is_empty(),
            "Should ignore paths that result in empty parts array"
        );
    }

    #[test]
    fn test_scene_parser_with_model() {
        let parser_xr18 = SceneParser::with_model(MixerModel::XR18);
        assert!(matches!(parser_xr18.model, MixerModel::XR18));

        let parser_x32 = SceneParser::with_model(MixerModel::X32);
        assert!(matches!(parser_x32.model, MixerModel::X32));
    }

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

    #[test]
    fn test_xair_scene_paths_and_limits() {
        let mut parser = SceneParser::with_model(MixerModel::XR18);

        // /lr/config/name & /lr/mix/fader & /lr/dyn/thr
        let m_lr_name = parser.parse_scene_line("/lr/config/name \"Main LR\"");
        assert_eq!(m_lr_name.len(), 1);
        assert_eq!(m_lr_name[0].path, "/lr/config/name");
        assert_eq!(
            m_lr_name[0].args,
            vec![OscArg::String("Main LR".to_string())]
        );

        let m_lr_fader = parser.parse_scene_line("/lr/mix/fader -oo");
        assert_eq!(m_lr_fader.len(), 1);
        assert_eq!(m_lr_fader[0].args, vec![OscArg::Float(0.0)]);

        // /rtn/aux & /rtn/1 & /fxsend/1 & /headamp/01
        let m_rtn_aux = parser.parse_scene_line("/rtn/aux/mix/on ON");
        assert_eq!(m_rtn_aux.len(), 1);
        assert_eq!(m_rtn_aux[0].args, vec![OscArg::Int(1)]);

        let m_rtn1 = parser.parse_scene_line("/rtn/1/eq/2/type PEQ");
        assert_eq!(m_rtn1.len(), 1);
        assert_eq!(m_rtn1[0].args, vec![OscArg::Int(2)]);

        let m_fxsend = parser.parse_scene_line("/fxsend/1/config/name \"Hall\"");
        assert_eq!(m_fxsend.len(), 1);
        assert_eq!(
            m_fxsend[0].args,
            vec![OscArg::String("Hall".to_string())]
        );

        let m_ha = parser.parse_scene_line("/headamp/01/phantom ON");
        assert_eq!(m_ha.len(), 1);
        assert_eq!(m_ha[0].args, vec![OscArg::Int(1)]);

        // /rtn/aux/config/rtnsrc & /routing/{main,aux,p16,usb}
        let m_rtnsrc = parser.parse_scene_line("/rtn/aux/config/rtnsrc U0304");
        assert_eq!(m_rtnsrc.len(), 1);
        assert_eq!(m_rtnsrc[0].args, vec![OscArg::Int(1)]);

        let m_rout_main = parser.parse_scene_line("/routing/main/01/src MON");
        assert_eq!(m_rout_main.len(), 1);
        assert_eq!(m_rout_main[0].args, vec![OscArg::Int(1)]);

        let m_rout_aux_pos = parser.parse_scene_line("/routing/aux/01/pos PRE");
        assert_eq!(m_rout_aux_pos.len(), 1);
        assert_eq!(m_rout_aux_pos[0].args, vec![OscArg::Int(8)]);

        let m_rout_usb = parser.parse_scene_line("/routing/usb/01/src Ch01");
        assert_eq!(m_rout_usb.len(), 1);
        assert_eq!(m_rout_usb[0].args, vec![OscArg::Int(0)]);

        // Model-aware link/mute limits for XR18 (16 ch = 8 pairs, 6 bus = 3 pairs, 4 mute groups, 4 FX slots)
        assert_eq!(parser.parse_scene_line("/config/chlink ON").len(), 8);
        assert_eq!(parser.parse_scene_line("/config/buslink ON").len(), 3);
        assert_eq!(parser.parse_scene_line("/config/mute ON").len(), 4);
        assert!(parser.parse_scene_line("/fx/5/type GEQ").is_empty());
    }
}
