use osc_lib::{OscArg, OscMessage};

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
        Self { fx_types: [0; 8] }
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
    let mut parts = [""; 8];
    let mut num_parts = 0;
    for part in path.trim_start_matches('/').split('/') {
        if num_parts < 8 {
            parts[num_parts] = part;
            num_parts += 1;
        } else {
            break;
        }
    }
    if num_parts == 0 {
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
            let exact_path = if path.starts_with('/') {
                path.to_string()
            } else {
                let mut s = String::with_capacity(path.len() + 1);
                s.push('/');
                s.push_str(path);
                s
            };

            // Try to figure out the type from the structure of `arg_str`
            // If it starts and ends with quotes, it's a string.
            if arg_str.starts_with('"') && arg_str.ends_with('"') {
                if let Some(arg) = parse_str(arg_str) {
                    messages.push(OscMessage::new(exact_path, vec![arg]));
                }
            }
            // If it's a known string enum (ON, OFF, PRE, POST, etc)
            else if arg_str == "ON" || arg_str == "OFF" || arg_str == "PRE" || arg_str == "POST" {
                if let Some(arg) = parse_onoff(arg_str, "OFF") {
                    messages.push(OscMessage::new(exact_path, vec![arg]));
                }
            }
            // If it starts with % it's bits
            else if arg_str.starts_with('%') {
                if let Some(arg) = parse_bits(arg_str) {
                    messages.push(OscMessage::new(exact_path, vec![arg]));
                }
            }
            // If it's -oo or looks like a float or level
            else if arg_str == "-oo"
                || arg_str.contains('.')
                || arg_str.contains('k')
                || arg_str.contains('K')
            {
                // Frequency heuristics
                if path.ends_with("/f") || path.contains("freq") || arg_str.contains('k') {
                    if let Some(arg) = parse_frequency(arg_str, 200.0) {
                        messages.push(OscMessage::new(exact_path, vec![arg]));
                    }
                }
                // Level/fader heuristics
                else if path.ends_with("/fader")
                    || path.ends_with("/level")
                    || path.ends_with("/mlevel")
                    || path.ends_with("/trim")
                {
                    if let Some(arg) = parse_level(arg_str, 1023.0) {
                        messages.push(OscMessage::new(exact_path, vec![arg]));
                    }
                }
                // Pan heuristics
                else if path.ends_with("/pan") {
                    if let Some(arg) = parse_flin(arg_str, -100.0, 200.0, 2.0) {
                        messages.push(OscMessage::new(exact_path, vec![arg]));
                    }
                }
                // General float fallback
                else {
                    if let Some(arg) = parse_float_raw(arg_str) {
                        messages.push(OscMessage::new(exact_path, vec![OscArg::Float(arg)]));
                    }
                }
            }
            // If it's just an integer
            else if let Ok(val) = arg_str.parse::<i32>() {
                messages.push(OscMessage::new(exact_path, vec![OscArg::Int(val)]));
            }
        }
    }

    messages
}

// --- Helper parsing functions ---

pub(crate) fn parse_str(val: &str) -> Option<OscArg> {
    let mut v = val;
    if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
        v = &v[1..v.len() - 1];
    }
    Some(OscArg::String(v.to_string()))
}

#[allow(dead_code)]
pub(crate) fn parse_int(val: &str) -> Option<OscArg> {
    val.parse::<i32>().ok().map(OscArg::Int)
}

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

pub(crate) fn parse_bits(val: &str) -> Option<OscArg> {
    if !val.starts_with('%') {
        return None;
    }
    let bits_str = &val[1..];
    let mut result = 0;
    for (i, c) in bits_str.chars().rev().enumerate() {
        if c == '1' {
            result |= 1 << i;
        }
    }
    Some(OscArg::Int(result))
}

#[allow(dead_code)]
pub(crate) fn parse_percent(val: &str) -> Option<OscArg> {
    val.parse::<f32>().ok().map(|v| OscArg::Float(v / 100.0))
}

pub(crate) fn parse_float_raw(val: &str) -> Option<f32> {
    if let Ok(v) = val.parse::<f32>() {
        return Some(v);
    }
    if let Some((int_part, dec_part)) = val.split_once('k') {
        let i = int_part.parse::<f32>().unwrap_or(0.0);
        let mut f = i * 1000.0;
        if !dec_part.is_empty() {
            let dec = dec_part.parse::<f32>().unwrap_or(0.0);
            let p = 10.0_f32.powi(3 - dec_part.len() as i32);
            f += dec * p;
        }
        return Some(f);
    }
    None
}

pub(crate) fn parse_flin(val: &str, xmin: f32, lmaxmin: f32, xstep: f32) -> Option<OscArg> {
    let mut fval = parse_float_raw(val)?;
    fval = (fval - xmin) / lmaxmin;
    let step = lmaxmin / xstep;
    fval = (fval * step).round() / step;
    Some(OscArg::Float(fval.clamp(0.0, 1.0)))
}

#[allow(dead_code)]
pub(crate) fn parse_flog(val: &str, xmin: f32, lmaxmin: f32, nsteps: f32) -> Option<OscArg> {
    let mut fval = parse_float_raw(val)?;
    fval = (fval / xmin).ln() / lmaxmin;
    fval = (fval * nsteps).round() / nsteps;
    Some(OscArg::Float(fval.clamp(0.0, 1.0)))
}

pub(crate) fn parse_frequency(val: &str, nsteps: f32) -> Option<OscArg> {
    let mut fval = parse_float_raw(val)?;
    fval = (fval / 20.0).ln() / 6.907_755_4;
    fval = (fval * nsteps).round() / nsteps;
    Some(OscArg::Float(fval.clamp(0.0, 1.0)))
}

pub(crate) fn parse_level(val: &str, nsteps: f32) -> Option<OscArg> {
    if val == "-oo" {
        return Some(OscArg::Float(0.0));
    }
    let mut fval = val.parse::<f32>().ok()?;
    if fval < -60.0 {
        fval = fval * 0.002_083_333_4 + 0.1875;
        fval = (fval * nsteps).round() / nsteps;
        if fval < 0.0 {
            fval = 0.0;
        }
    } else if fval < -30.0 {
        fval = 0.00625 * fval + 0.4375;
        fval = (fval * nsteps).round() / nsteps;
    } else if fval < -10.0 {
        fval = 0.0125 * fval + 0.625;
        fval = (fval * nsteps).round() / nsteps;
    } else {
        fval = fval * 0.025 + 0.75;
        if fval > 1.0 {
            fval = 1.0;
        }
    }
    Some(OscArg::Float(fval))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scene_line_string() {
        let msgs = parse_scene_line("/ch/01/config/name \"Lead Vox\"");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/config/name");
        assert_eq!(msgs[0].args, vec![OscArg::String("Lead Vox".to_string())]);
    }

    #[test]
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
    fn test_parse_scene_line_onoff() {
        let msgs = parse_scene_line("/ch/01/mix/on ON");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/mix/on");
        assert_eq!(msgs[0].args, vec![OscArg::Int(1)]);
    }

    #[test]
    fn test_parse_scene_line_chlink() {
        let msgs = parse_scene_line("/config/chlink ON");
        assert_eq!(msgs.len(), 16);
        assert_eq!(msgs[0].path, "/config/chlink/1-2");
        assert_eq!(msgs[0].args, vec![OscArg::Int(1)]);
        assert_eq!(msgs[15].path, "/config/chlink/31-32");
    }

    #[test]
    fn test_parse_scene_line_frequency() {
        let msgs = parse_scene_line("/ch/01/eq/1/f 1k2");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/eq/1/f");
        assert!(matches!(msgs[0].args[0], OscArg::Float(_)));
    }
}
/// Parses space-separated string arguments into proper OscArgs for a given FX slot and type.
pub(crate) fn parse_fx_par(slot: &str, fx_type: i32, args_str: &str) -> Option<OscMessage> {
    let args: Vec<&str> = args_str.split_whitespace().collect();
    let mut osc_args = Vec::new();

    // Helper functions for inline scaling
    let parse_ratio = |val: &str, a: f32| -> OscArg {
        if let Some(mut f) = parse_float_raw(val) {
            f /= a;
            OscArg::Float(f.clamp(0.0, 1.0))
        } else {
            OscArg::Float(0.0)
        }
    };

    let parse_afine = |val: &str, a: f32, b: f32| -> OscArg {
        if let Some(mut f) = parse_float_raw(val) {
            f = (f - a) / b;
            OscArg::Float(f.clamp(0.0, 1.0))
        } else {
            OscArg::Float(0.0)
        }
    };

    let parse_log = |val: &str, a: f32, b: f32| -> OscArg {
        if let Some(mut f) = parse_float_raw(val) {
            f = (f / a).ln() / b;
            OscArg::Float(f.clamp(0.0, 1.0))
        } else {
            OscArg::Float(0.0)
        }
    };

    #[allow(dead_code)]
    let _parse_toggle = |val: &str, off_val: &str| -> OscArg {
        if val == off_val {
            OscArg::Int(0)
        } else {
            OscArg::Int(1)
        }
    };

    let parse_list_idx = |val: &str, list: &[&str]| -> OscArg {
        if let Some(idx) = list.iter().position(|&s| s == val) {
            OscArg::Int(idx as i32)
        } else {
            OscArg::Int(0)
        }
    };

    let is_slot_1_4 = if let Ok(s) = slot.parse::<usize>() {
        s <= 4
    } else {
        true
    };

    if is_slot_1_4 {
        match fx_type {
            0 | 5 => {
                // HALL, PLAT
                if args.len() >= 12 {
                    osc_args.push(parse_ratio(args[0], 200.0)); // pre delay
                    if fx_type == 0 {
                        osc_args.push(parse_log(args[1], 0.1, 3.912_023)); // decay
                    } else {
                        osc_args.push(parse_log(args[1], 0.5, 2.995_732_3)); // decay
                    }
                    osc_args.push(parse_afine(args[2], 2.0, 98.0)); // size
                    osc_args.push(parse_log(args[3], 1000.0, 2.995_732_3)); // damping
                    osc_args.push(parse_afine(args[4], 1.0, 29.0)); // diffuse
                    osc_args.push(parse_afine(args[5], -12.0, 24.0)); // level
                    osc_args.push(parse_log(args[6], 10.0, 3.912_023)); // lo cut
                    osc_args.push(parse_log(args[7], 200.0, 4.605_170_2)); // hi cut
                    osc_args.push(parse_log(args[8], 0.5, 1.386_294_4)); // bass multi
                    if fx_type == 0 {
                        osc_args.push(parse_ratio(args[9], 50.0)); // spread
                    } else {
                        osc_args.push(parse_log(args[9], 10.0, 3.912_023)); // xover
                    }
                    if fx_type == 0 {
                        osc_args.push(parse_ratio(args[10], 250.0)); // shape
                    } else {
                        osc_args.push(parse_ratio(args[10], 50.0)); // mod
                    }
                    osc_args.push(parse_ratio(args[11], 100.0)); // modspeed
                }
            }
            1 => {
                // AMBI
                if args.len() >= 10 {
                    osc_args.push(parse_ratio(args[0], 200.0)); // pre delay
                    osc_args.push(parse_log(args[1], 0.1, 3.912_023)); // decay
                    osc_args.push(parse_afine(args[2], 2.0, 98.0)); // size
                    osc_args.push(parse_log(args[3], 1000.0, 2.995_732_3)); // damping
                    osc_args.push(parse_afine(args[4], 1.0, 29.0)); // diffuse
                    osc_args.push(parse_afine(args[5], -12.0, 24.0)); // level
                    osc_args.push(parse_log(args[6], 10.0, 3.912_023)); // lo cut
                    osc_args.push(parse_log(args[7], 200.0, 4.605_170_2)); // hi cut
                    osc_args.push(parse_ratio(args[8], 50.0)); // mod
                    osc_args.push(parse_ratio(args[9], 100.0)); // modspeed
                }
            }
            2..=4 => {
                // RPLT, ROOM, CHAM
                if args.len() >= 16 {
                    osc_args.push(parse_ratio(args[0], 200.0)); // pre delay
                    osc_args.push(parse_log(args[1], 0.1, 3.912_023)); // decay
                    if fx_type == 2 {
                        osc_args.push(parse_afine(args[2], 4.0, 35.0)); // size
                    } else {
                        osc_args.push(parse_afine(args[2], 4.0, 72.0)); // size
                    }
                    osc_args.push(parse_log(args[3], 1000.0, 2.995_732_3)); // damping
                    osc_args.push(parse_afine(args[4], 1.0, 29.0)); // diffuse
                    osc_args.push(parse_afine(args[5], -12.0, 24.0)); // level
                    osc_args.push(parse_log(args[6], 10.0, 3.912_023)); // lo cut
                    osc_args.push(parse_log(args[7], 200.0, 4.605_170_2)); // hi cut
                    osc_args.push(parse_log(args[8], 0.25, 2.772_588_7)); // bass multi
                    osc_args.push(parse_ratio(args[9], 50.0)); // spread
                    if fx_type == 2 {
                        osc_args.push(parse_ratio(args[10], 100.0)); // attack
                    } else {
                        osc_args.push(parse_ratio(args[10], 250.0)); // shape
                    }
                    osc_args.push(parse_ratio(args[11], 100.0)); // spin
                    if fx_type == 4 {
                        osc_args.push(parse_ratio(args[12], 500.0)); // refl. L
                        osc_args.push(parse_ratio(args[13], 500.0)); // refl. R
                        osc_args.push(parse_ratio(args[14], 100.0)); // refl. gain L
                        osc_args.push(parse_ratio(args[15], 100.0)); // refl. gain R
                    } else {
                        osc_args.push(parse_ratio(args[12], 1200.0)); // echo L
                        osc_args.push(parse_ratio(args[13], 1200.0)); // echo R
                        osc_args.push(parse_afine(args[14], -100.0, 200.0)); // echo feed L
                        osc_args.push(parse_afine(args[15], -100.0, 200.0)); // echo feed R
                    }
                }
            }
            10 => {
                // DLY
                if args.len() >= 12 {
                    osc_args.push(parse_ratio(args[0], 100.0)); // mix
                    osc_args.push(parse_afine(args[1], 1.0, 2999.0)); // time
                    osc_args.push(parse_list_idx(args[2], &["ST", "X", "M"])); // mode
                    osc_args.push(parse_list_idx(
                        args[3],
                        &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                    )); // factor L
                    osc_args.push(parse_list_idx(
                        args[4],
                        &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                    )); // factor R
                    osc_args.push(parse_ratio(args[5], 100.0)); // offset L
                    osc_args.push(parse_ratio(args[6], 100.0)); // offset R
                    osc_args.push(parse_log(args[7], 10.0, 3.912_023)); // lo cut
                    osc_args.push(parse_log(args[8], 200.0, 4.605_170_2)); // hi cut
                    osc_args.push(parse_log(args[9], 10.0, 3.912_023)); // feed lo cut
                    osc_args.push(parse_ratio(args[10], 100.0)); // feed left
                    osc_args.push(parse_ratio(args[11], 100.0)); // feed right
                    // feed hi cut omitted if len is 12, wait, DLY has 13 params in C if you count feed hi cut
                    if args.len() >= 13 {
                        osc_args.push(parse_log(args[12], 200.0, 4.605_170_2)); // feed hi cut
                    }
                }
            }
            _ => {
                // For safety, we do not fallback to unscaled floats/ints for unmapped effects,
                // as sending unscaled values (e.g., 500.0 instead of 0.5) to the X32 can peg
                // parameters to their maximum limits and cause dangerous volume spikes or feedback.
                // Additional effects from `fxparse1.c` and `fxparse.c` must be explicitly ported.
                return None;
            }
        }
    } else {
        // Slot 5-8 FX types (GEQ, TEQ, etc.)
        // For safety, do not fallback to unscaled floats.
        return None;
    }

    Some(OscMessage::new(format!("/fx/{}/par", slot), osc_args))
}
