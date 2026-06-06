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

pub(crate) fn parse_ratio(s: &str, m: f32) -> OscArg {
    let f = s.parse::<f32>().unwrap_or(0.0);
    OscArg::Float((f / m).clamp(0.0, 1.0))
}

pub(crate) fn parse_afine(s: &str, min: f32, steps: f32) -> OscArg {
    let f = s.parse::<f32>().unwrap_or(0.0);
    OscArg::Float(((f - min) / steps).clamp(0.0, 1.0))
}

pub(crate) fn parse_log(s: &str, base: f32, factor: f32) -> OscArg {
    let mut f = s.parse::<f32>().unwrap_or(base);
    if f < base {
        f = base;
    }
    let v = (f / base).ln() / factor;
    OscArg::Float(v.clamp(0.0, 1.0))
}

pub(crate) fn parse_list_idx(s: &str, list: &[&str]) -> OscArg {
    if let Some(pos) = list.iter().position(|&x| x == s) {
        OscArg::Int(pos as i32)
    } else {
        OscArg::Int(0)
    }
}

#[allow(dead_code)]
pub(crate) fn parse_sub(s: &str, limit: i32) -> OscArg {
    let mut num: i32 = s.parse().unwrap_or(0);
    if num > limit {
        num = limit;
    }
    OscArg::Int(num)
}

#[allow(dead_code)]
pub(crate) fn parse_pan(s: &str, max_val: f32) -> OscArg {
    let num: f32 = s.parse().unwrap_or(0.0);
    OscArg::Float((num / max_val).clamp(0.0, 1.0))
}

#[allow(
    clippy::excessive_precision,
    clippy::manual_range_patterns,
    clippy::approx_constant,
    clippy::needless_range_loop
)]
pub(crate) fn parse_fx_par(slot: &str, fx_type: i32, args_str: &str) -> Option<OscMessage> {
    let args: Vec<&str> = args_str.split_whitespace().collect();
    let mut osc_args = Vec::with_capacity(args.len());
    let path = format!("/fx/{}/par", slot);

    match fx_type {
        0 | 5 => {
            if args.len() >= 12 {
                osc_args.push(parse_ratio(args[0], 200.0)); // pre delay 0-200
                if fx_type == 0 {
                    osc_args.push(parse_log(args[1], 0.2, 3.218895825));
                } // decay [.2, 5] 3.218895825 = log(5/0.2)
                if fx_type == 5 {
                    osc_args.push(parse_log(args[1], 0.5, 2.995732274));
                } // decay [.5, 10] 2.995732274 = log(10/0.5)
                osc_args.push(parse_afine(args[2], 2.0, 98.0)); // size [2, 100]
                osc_args.push(parse_log(args[3], 1000.0, 2.995732274)); // damping HZ [1000, 20000] 2.995732274 = log(20000/1000)
                osc_args.push(parse_afine(args[4], 1.0, 29.0)); // Diffuse [1, 30]
                osc_args.push(parse_afine(args[5], -12.0, 24.0)); // level [-12,+12]
                osc_args.push(parse_log(args[6], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[7], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(200/20000)
                osc_args.push(parse_log(args[8], 0.5, 1.386294361)); // bass multi [.5, 2] 1.386294361 = log(2/.5)
                if fx_type == 0 {
                    osc_args.push(parse_ratio(args[9], 50.0));
                } // spread [0, 50]
                if fx_type == 5 {
                    osc_args.push(parse_log(args[9], 10.0, 3.912023005));
                } // xover [10, 500] 3.912023005 = log(500/10)
                if fx_type == 0 {
                    osc_args.push(parse_ratio(args[10], 250.0));
                } // shape [0, 250]
                if fx_type == 5 {
                    osc_args.push(parse_ratio(args[10], 50.0));
                } // mod [0, 50]
                osc_args.push(parse_ratio(args[11], 100.0)); // modspeed [0, 100]
            }
        }
        1 | 2 | 3 => {
            if args.len() >= 10 {
                osc_args.push(parse_ratio(args[0], 200.0)); // pre delay 0-200
                osc_args.push(parse_log(args[1], 0.2, 3.597312261)); // decay [.2, 7.3] [ 3.597312261 = (log(7.3) - log(0.2))
                osc_args.push(parse_afine(args[2], 2.0, 98.0)); // size [2, 100]
                osc_args.push(parse_log(args[3], 1000.0, 2.995732274)); // damping HZ [1000, 20000] 2.995732274 = log(20000/1000)
                osc_args.push(parse_afine(args[4], 1.0, 29.0)); // Diffuse [1, 30]
                osc_args.push(parse_afine(args[5], -12.0, 24.0)); // level [-12,+12]
                osc_args.push(parse_log(args[6], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[7], 200.0, 4.605170186)); // hi cut HZ [200, 20k] [ 4.605170186 = log(200/20000)]
                osc_args.push(parse_ratio(args[8], 100.0)); // modulate [0, 100]
                osc_args.push(parse_ratio(args[9], 100.0)); // tail gain [0, 100]
            }
        }
        4 => {
            if args.len() >= 14 {
                osc_args.push(parse_ratio(args[0], 200.0)); // pre delay 0-200
                osc_args.push(parse_log(args[1], 0.3, 4.571268634)); // decay [.3, 29]
                osc_args.push(parse_log(args[2], 1000.0, 2.995732274)); // damping HZ [1000, 20000] 2.995732274 = log(20000/1000)
                osc_args.push(parse_ratio(args[3], 100.0)); // Diffuse [0, 100]
                osc_args.push(parse_afine(args[4], -12.0, 24.0)); // level [-12,+12]
                osc_args.push(parse_log(args[5], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[6], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(200/20000)
                osc_args.push(parse_log(args[7], 0.25, 2.772588722)); // bass multi [.25, 4]
                osc_args.push(parse_ratio(args[8], 50.0)); // spread [0, 50]
                osc_args.push(parse_ratio(args[9], 100.0)); // spin [0, 100]
                if fx_type == 4 {
                    osc_args.push(parse_ratio(args[10], 500.0));
                } // refl. L [0,500]
                if fx_type == 4 {
                    osc_args.push(parse_ratio(args[11], 500.0));
                } // refl. R [0,500]
                if fx_type == 4 {
                    osc_args.push(parse_ratio(args[12], 100.0));
                } // refl. gain L [0,100]
                if fx_type == 4 {
                    osc_args.push(parse_ratio(args[13], 100.0));
                } // refl. gain R [0,100]
            }
        }
        6 => {
            if args.len() >= 8 {
                osc_args.push(parse_ratio(args[0], 120.0)); // pre delay 0-120
                osc_args.push(parse_log(args[1], 0.3, 2.708050201)); // decay [.3, 4.5]
                osc_args.push(parse_ratio(args[2], 100.0)); // modulate [0, 10]
                osc_args.push(parse_afine(args[3], -12.0, 24.0)); // level [-12,+12]
                osc_args.push(parse_log(args[4], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[5], 10000.0, 0.693147181)); // hi cut HZ [10k, 20k] 4.605170186 = log(20000/10000)
                osc_args.push(parse_log(args[6], 0.5, 1.386294361)); // lo multiply [0.5, 2]
                osc_args.push(parse_log(args[7], 0.25, 1.386294361)); // hi multiply [0.25, 1]
            }
        }
        7 | 8 => {
            if args.len() >= 12 {
                osc_args.push(parse_ratio(args[0], 200.0)); // revb delay 0-200
                osc_args.push(parse_log(args[1], 0.1, 5.298317367)); // decay [.1, 20]
                osc_args.push(parse_ratio(args[2], 100.0)); // size [0, 10]
                osc_args.push(parse_afine(args[3], 1.0, 29.0)); // density [1, 30]
                osc_args.push(parse_ratio(args[4], 100.0)); // ER level [0, 190]
                osc_args.push(parse_afine(args[5], -12.0, 24.0)); // level [-12,+12]
                osc_args.push(parse_log(args[6], 0.1, 4.605170186)); // lo multiply [0.1, 10]
                osc_args.push(parse_log(args[7], 0.1, 4.605170186)); // hi multiply [0.1, 10]
                osc_args.push(parse_log(args[8], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[9], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(20000/200)
                osc_args.push(parse_ratio(args[10], 200.0)); // ER left [0, 10]
                osc_args.push(parse_ratio(args[11], 200.0)); // ER right [0, 10]
            }
        }
        9 => {
            if args.len() >= 10 {
                osc_args.push(parse_ratio(args[0], 200.0)); // pre-delay 0-200
                osc_args.push(parse_afine(args[1], 140.0, 860.0)); // decay [140, 1000]
                if fx_type == 8 {
                    osc_args.push(parse_ratio(args[2], 30.0));
                } // attack [0, 30]
                if fx_type == 9 {
                    osc_args.push(parse_ratio(args[2], 50.0));
                } //rize [0, 50]
                if fx_type == 8 {
                    osc_args.push(parse_afine(args[3], 1.0, 49.0));
                } // density [1, 50]
                if fx_type == 9 {
                    osc_args.push(parse_afine(args[3], 1.0, 29.0));
                } //diffuse [1, 30]
                osc_args.push(parse_ratio(args[4], 100.0)); // spread [0, 100]
                osc_args.push(parse_afine(args[5], -12.0, 24.0)); // level [-12,+12]
                osc_args.push(parse_log(args[6], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[7], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(20000/200)
                osc_args.push(parse_afine(args[8], -30.0, 30.0)); // hi shv gain [-30, 0]
                if fx_type == 8 {
                    osc_args.push(parse_afine(args[9], 1.0, 29.0));
                } // diffuse [1, 30]
            }
        }
        10 => {
            if args.len() >= 12 {
                osc_args.push(parse_ratio(args[0], 100.0)); // mix [0, 100]
                osc_args.push(parse_afine(args[1], 1.0, 2999.0)); // time [1, 3000]
                osc_args.push(parse_list_idx(args[2], &["ST", "X", "M"])); // mode [ST, X, M]
                osc_args.push(parse_list_idx(
                    args[3],
                    &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                )); // factor L [1/4, 3/8, 1/2, 2/3, 1, 4/3, 3/2, 2, 3]
                osc_args.push(parse_list_idx(
                    args[4],
                    &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                )); // factor R [1/4, 3/8, 1/2, 2/3, 1, 4/3, 3/2, 2, 3]
                osc_args.push(parse_afine(args[5], -100.0, 200.0)); // offset L/R [-100, +100]
                osc_args.push(parse_log(args[6], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[7], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(20000/200)
                osc_args.push(parse_log(args[8], 10.0, 3.912023005)); // feed lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_ratio(args[9], 100.0)); // feed left [0, 100]
                osc_args.push(parse_ratio(args[10], 100.0)); // feed right [0, 100]
                osc_args.push(parse_log(args[11], 200.0, 4.605170186)); // feed hi cut HZ [200, 20k] 4.605170186 = log(20000/200)
            }
        }
        11 => {
            if args.len() >= 12 {
                osc_args.push(parse_afine(args[0], 1.0, 2999.0)); // dry [1, 3000]
                osc_args.push(parse_ratio(args[1], 100.0)); // gain base [0, 100]
                osc_args.push(parse_afine(args[2], -100.0, 200.0)); // pan base [-100, 100]
                osc_args.push(parse_ratio(args[3], 100.0)); // feedback [0, 100]
                osc_args.push(parse_log(args[4], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[5], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(20000/200)
                osc_args.push(parse_list_idx(
                    args[6],
                    &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                )); // factorA [1/4...3]
                osc_args.push(parse_ratio(args[7], 100.0)); // gainA [0, 100]
                osc_args.push(parse_afine(args[8], -100.0, 200.0)); // panA [-100, 100]
                osc_args.push(parse_list_idx(
                    args[9],
                    &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                )); // factorB [1/4...3]
                osc_args.push(parse_ratio(args[10], 100.0)); // gainB [0, 100]
                osc_args.push(parse_afine(args[11], -100.0, 200.0)); // panB [-100, 100]
            }
        }
        12 => {
            if args.len() >= 12 {
                osc_args.push(parse_afine(args[0], 1.0, 2999.0)); // time [1, 3000]
                osc_args.push(parse_ratio(args[1], 100.0)); // gain base [0, 100]
                osc_args.push(parse_ratio(args[2], 100.0)); // feedback [0, 100]
                osc_args.push(parse_log(args[3], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[4], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(20000/200)
                osc_args.push(parse_ratio(args[5], 6.0)); // spread [0, 6]
                osc_args.push(parse_list_idx(
                    args[6],
                    &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                )); // factorA [1/4...3]
                osc_args.push(parse_ratio(args[7], 100.0)); // gainA [0, 100]
                osc_args.push(parse_list_idx(
                    args[8],
                    &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                )); // factorB [1/4...3]
                osc_args.push(parse_ratio(args[9], 100.0)); // gainB [0, 100]
                osc_args.push(parse_list_idx(
                    args[10],
                    &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                )); // factorC [1/4...3]
                osc_args.push(parse_ratio(args[11], 100.0)); // gainC [0, 100]
            }
        }
        13 => {
            if args.len() >= 11 {
                osc_args.push(parse_log(args[0], 0.05, 4.605170186)); // speed HZ [0.05, 5]
                osc_args.push(parse_ratio(args[1], 100.0)); // depth L [0, 100]
                osc_args.push(parse_ratio(args[2], 100.0)); // depth R [0, 100]
                osc_args.push(parse_log(args[3], 0.5, 4.605170186)); // delay L [0.5, 50]
                osc_args.push(parse_log(args[4], 0.5, 4.605170186)); // delay R [0.5, 50]
                osc_args.push(parse_ratio(args[5], 100.0)); // mix [0, 100]
                osc_args.push(parse_log(args[6], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[7], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(20000/200)
                osc_args.push(parse_ratio(args[8], 180.0)); // phase [0, 180]
                osc_args.push(parse_ratio(args[9], 100.0)); // wave [0, 100]
                osc_args.push(parse_ratio(args[10], 100.0)); // spread [0, 100]
            }
        }
        14 | 15 | 16 | 17 => {
            if args.len() >= 12 {
                osc_args.push(parse_log(args[0], 0.05, 4.605170186)); // speed HZ [0.05, 5]
                osc_args.push(parse_ratio(args[1], 100.0)); // depth L [0, 100]
                osc_args.push(parse_ratio(args[2], 100.0)); // depth R [0, 100]
                osc_args.push(parse_log(args[3], 0.5, 3.688879454)); // delay L [0.5, 20]
                osc_args.push(parse_log(args[4], 0.5, 3.688879454)); // delay R [0.5, 20]
                osc_args.push(parse_ratio(args[5], 100.0)); // mix [0, 100]
                osc_args.push(parse_log(args[6], 10.0, 3.912023005)); // lo cut HZ [10, 500] 3.912023005 = log(500/10)
                osc_args.push(parse_log(args[7], 200.0, 4.605170186)); // hi cut HZ [200, 20k] 4.605170186 = log(20000/200)
                osc_args.push(parse_ratio(args[8], 180.0)); // phase [0, 180]
                osc_args.push(parse_log(args[9], 10.0, 3.912023005)); // feed lo cut HZ [10, 500]
                osc_args.push(parse_log(args[10], 200.0, 4.605170186)); // feed hi cut HZ [200, 20k]
                osc_args.push(parse_afine(args[11], -90.0, 180.0)); // feed [-90, 90]]
            }
        }
        18 | 19 | 20 | 21 | 24 => {
            if args.len() >= 6 {
                osc_args.push(parse_log(args[0], 0.1, 3.688879454)); // lo speed HZ [0.1, 4]
                osc_args.push(parse_log(args[1], 2.0, 1.609437912)); // hi speed HZ [2, 10]
                osc_args.push(parse_ratio(args[2], 100.0)); // accelerate [0, 100]
                osc_args.push(parse_ratio(args[3], 100.0)); // distance [0, 100]
                osc_args.push(parse_afine(args[4], -100.0, 200.0)); // balance [-100, 100]
                osc_args.push(parse_ratio(args[5], 100.0)); // mix [0, 100]
            }
        }
        22 | 25 => {
            if args.len() >= 22 {
                osc_args.push(parse_afine(args[0], 1.0, 2999.0)); // time [1, 3000]
                osc_args.push(parse_list_idx(args[1], &["1", "2", "3", "4"])); // pattern
                osc_args.push(parse_log(args[2], 1000.0, 2.995732274)); // feed hi cut [1000, 20000]
                osc_args.push(parse_ratio(args[3], 100.0)); // feedback [0, 100]
                osc_args.push(parse_ratio(args[4], 100.0)); // cross feed [0, 100]
                osc_args.push(parse_afine(args[5], -100.0, 200.0)); // balance [-100, 100]
                osc_args.push(parse_ratio(args[6], 200.0)); // pre delay [0, 200]
                osc_args.push(parse_log(args[7], 0.1, 3.912023005)); // decay [0.1, 5]
                osc_args.push(parse_afine(args[8], 2.0, 98.0)); // size [2, 100]
                osc_args.push(parse_log(args[9], 1000.0, 2.995732274)); // damping [1000, 20000]
                osc_args.push(parse_log(args[10], 10.0, 3.912023005)); // lo cut [10, 500]
                osc_args.push(parse_log(args[11], 0.05, 4.382026635)); // speed [0.05, 4]
                osc_args.push(parse_ratio(args[12], 100.0)); // depth [0, 100]
                osc_args.push(parse_log(args[13], 0.5, 4.605170186)); // delay [0.5, 50]
                osc_args.push(parse_ratio(args[14], 180.0)); // phase [0, 180]
                osc_args.push(parse_ratio(args[15], 100.0)); // wave [0, 100]
                osc_args.push(parse_log(args[16], 0.05, 4.382026635)); // speed [0.05, 4]
                osc_args.push(parse_ratio(args[17], 100.0)); // depth [0, 100]
                osc_args.push(parse_log(args[18], 0.5, 3.688879454)); // delay [0.5, 20]
                osc_args.push(parse_ratio(args[19], 180.0)); // phase [0, 180]
                osc_args.push(parse_afine(args[20], -90.0, 180.0)); // feed [-90, 90]
                osc_args.push(parse_ratio(args[21], 100.0)); // mix [0, 100]
            }
        }
        23 => {
            if args.len() >= 12 {
                osc_args.push(parse_log(args[0], 0.05, 4.382026635)); // speed [0.05, 4]
                osc_args.push(parse_ratio(args[1], 100.0)); // depth [0, 100]
                if fx_type == 22 {
                    osc_args.push(parse_log(args[2], 0.5, 4.605170186));
                } // delay [0.5, 50]
                if fx_type == 23 {
                    osc_args.push(parse_log(args[2], 0.5, 3.688879454));
                } // delay [0.5, 20]
                osc_args.push(parse_ratio(args[3], 180.0)); // phase [0, 180]
                if fx_type == 22 {
                    osc_args.push(parse_ratio(args[4], 100.0));
                } // wave [0, 100]
                if fx_type == 23 {
                    osc_args.push(parse_afine(args[4], -90.0, 180.0));
                } // feed [-90, 90]
                osc_args.push(parse_afine(args[5], -100.0, 200.0)); // balance [-100, 100]
                osc_args.push(parse_ratio(args[6], 200.0)); // pre delay [0, 200]
                osc_args.push(parse_log(args[7], 0.1, 3.912023005)); // decay [0.1, 5]
                osc_args.push(parse_afine(args[8], 2.0, 98.0)); // size [2, 100]
                osc_args.push(parse_log(args[9], 1000.0, 2.995732274)); // damping [1000, 20000]
                osc_args.push(parse_log(args[10], 10.0, 3.912023005)); // lo cut [10, 500]
                osc_args.push(parse_ratio(args[11], 100.0)); // mix [0, 100]
            }
        }

        28 | 30 => {
            // GEQ (28), TEQ (30) - 32 parameters
            if args.len() >= 32 {
                for i in 0..32 {
                    osc_args.push(parse_afine(args[i], -15.0, 30.0)); // [-15, +15]
                }
            }
        }
        27 | 29 => {
            // GEQ2 (27), TEQ2 (29) - 64 parameters
            if args.len() >= 64 {
                for i in 0..64 {
                    osc_args.push(parse_afine(args[i], -15.0, 30.0)); // [-15, +15]
                }
            }
        }
        26 => {
            if args.len() >= 12 {
                osc_args.push(parse_afine(args[0], 1.0, 2999.0)); // time [1, 3000]
                osc_args.push(parse_list_idx(
                    args[1],
                    &["1/4", "3/8", "1/2", "2/3", "1", "4/3", "3/2", "2", "3"],
                )); // delay
                osc_args.push(parse_ratio(args[2], 100.0)); // feed [0, 100]
                osc_args.push(parse_log(args[3], 10.0, 3.912023005)); // lo cut HZ [10, 500]
                osc_args.push(parse_log(args[4], 200.0, 4.605170186)); // hi cut HZ [200, 20k]
                osc_args.push(parse_ratio(args[5], 100.0)); // depth R [0, 100]
                osc_args.push(parse_log(args[6], 0.05, 5.298317367)); // rate [0.05, 10]
                osc_args.push(parse_list_idx(args[7], &["AMB", "CLUB", "HALL"])); // type
                osc_args.push(parse_afine(args[8], 1.0, 9.0)); // decay [1, 10]
                osc_args.push(parse_log(args[9], 1000.0, 2.995732274)); // damping [1000, 20000]
                osc_args.push(parse_afine(args[10], -100.0, 200.0)); // balance [-100, 100]
                osc_args.push(parse_ratio(args[11], 100.0)); // mix [0, 100]
            }
        }
        40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57
        | 58 => {
            if args.len() >= 20 {
                osc_args.push(parse_list_idx(args[0], &["1", "2", "3", "4"])); // band solo
                osc_args.push(parse_ratio(args[1], 100.0)); // mix [0, 100]
                osc_args.push(parse_ratio(args[2], 19.0)); // attack [0, 19]
                osc_args.push(parse_log(args[3], 20.0, 5.010635294)); // release [20, 3000]
                osc_args.push(parse_ratio(args[4], 10.0)); // sbc speed [0, 10]
                osc_args.push(parse_afine(args[5], -50.0, 100.0)); // xover [-50, 50]
                osc_args.push(parse_list_idx(args[6], &["1", "2", "3", "4"])); // ratio
                osc_args.push(parse_afine(args[7], -40.0, 40.0)); // threshold [-40, 0]
                osc_args.push(parse_afine(args[8], -10.0, 20.0)); // gain [-10, 10]
                osc_args.push(parse_afine(args[9], -10.0, 20.0)); // band threshold Low
                osc_args.push(parse_afine(args[10], -10.0, 20.0)); // band gain Low
                osc_args.push(parse_afine(args[11], -10.0, 20.0)); // band threshold LoMid
                osc_args.push(parse_afine(args[12], -10.0, 20.0)); // band gain LoMid
                osc_args.push(parse_afine(args[13], -10.0, 20.0)); // band threshold Mid
                osc_args.push(parse_afine(args[14], -10.0, 20.0)); // band gain Mid
                osc_args.push(parse_afine(args[15], -10.0, 20.0)); // band threshold HiMid
                osc_args.push(parse_afine(args[16], -10.0, 20.0)); // band gain HiMid
                osc_args.push(parse_afine(args[17], -10.0, 20.0)); // band threshold High
                osc_args.push(parse_afine(args[18], -10.0, 20.0)); // band gain High
                osc_args.push(parse_list_idx(args[19], &["1", "2", "3", "4"])); // meter mode
            }
        }
        59 | 60 => {
            if args.len() >= 6 {
                osc_args.push(parse_afine(args[0], -12.0, 24.0)); // semitone [-12, 12]
                osc_args.push(parse_afine(args[1], -50.0, 100.0)); // cent [-50, 50]
                osc_args.push(parse_log(args[2], 1.0, 6.214608098)); // delay [1, 500]
                osc_args.push(parse_log(args[3], 10.0, 3.912023005)); // lo cut [10, 500]
                osc_args.push(parse_log(args[4], 2000.0, 2.302585093)); // hi cut [2k, 20k]
                osc_args.push(parse_ratio(args[5], 100.0)); // mix [0, 100]
            }
        }

        _ => {
            return None;
        }
    }

    if osc_args.is_empty() {
        return None;
    }

    Some(OscMessage {
        path,
        args: osc_args,
    })
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
