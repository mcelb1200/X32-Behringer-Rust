use osc_lib::{OscArg, OscMessage};

/// Parses a line from an X32/XAir scene file into one or more OSC messages.
///
/// Scene files contain commands like `/ch/01/config/name "MyName"` or `/config/chlink 1-2`.
/// This function translates those shorthand commands into fully-typed OSC messages that can
/// be sent to the mixer (e.g. `/ch/01/config/name ,s "MyName"`).
///
/// If a line cannot be parsed as a known scene command, it returns an empty vector,
/// allowing the caller to attempt parsing it as a raw OSC message.
pub fn parse_scene_line(line: &str) -> Vec<OscMessage> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return vec![];
    }

    let (path, arg_str) = match line.split_once(|c: char| c.is_whitespace()) {
        Some((p, a)) => (p, a.trim()),
        None => (line, ""),
    };
    if arg_str.is_empty() { return vec![]; }

    let mut messages = Vec::new();
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.is_empty() { return messages; }

    match parts.as_slice() {
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
                messages.push(OscMessage::new("/config/linkcfg/eq".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/linkcfg/dyn".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/linkcfg/fdrmute".to_string(), vec![arg]));
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
            if let Some(arg) = parse_list(arg_str, &["OFF", "LR", "LR+C", "LRPFL", "LRAFL", "AUX56", "AUX78"]) {
                messages.push(OscMessage::new(p, vec![arg.clone()]));
                messages.push(OscMessage::new("/config/solo/level".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/solo/chmode".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/solo/busmode".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/solo/dcamode".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/solo/exclusive".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/solo/followsel".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/solo/followfader".to_string(), vec![arg.clone()]));
                messages.push(OscMessage::new("/config/solo/dimatt".to_string(), vec![arg]));
            }
        }
        // General fallback handler for all specific channel/bus/etc settings
        _ => {
            let exact_path = format!("/{}", parts.join("/"));

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
            else if arg_str == "-oo" || arg_str.contains('.') || arg_str.contains('k') || arg_str.contains('K') {
                // Frequency heuristics
                if path.ends_with("/f") || path.contains("freq") || arg_str.contains('k') {
                    if let Some(arg) = parse_frequency(arg_str, 200.0) {
                        messages.push(OscMessage::new(exact_path, vec![arg]));
                    }
                }
                // Level/fader heuristics
                else if path.ends_with("/fader") || path.ends_with("/level") || path.ends_with("/mlevel") || path.ends_with("/trim") {
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
    list.iter().position(|&s| s == val).map(|i| OscArg::Int(i as i32))
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
        if fval < 0.0 { fval = 0.0; }
    } else if fval < -30.0 {
        fval = 0.00625 * fval + 0.4375;
        fval = (fval * nsteps).round() / nsteps;
    } else if fval < -10.0 {
        fval = 0.0125 * fval + 0.625;
        fval = (fval * nsteps).round() / nsteps;
    } else {
        fval = fval * 0.025 + 0.75;
        if fval > 1.0 { fval = 1.0; }
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
