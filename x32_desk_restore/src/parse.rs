use osc_lib::{OscArg, OscMessage};

// Parsing constants
const R_PROT: &[&str] = &["MC", "HUI", "CC"];
const R_PORT: &[&str] = &["MIDI", "CARD", "RTP"];
const R_IQM: &[&str] = &["none", "iQ8", "iQ10", "iQ12", "iQ15", "iQ15B", "iQ18B"];
const R_IQE: &[&str] = &["none", "Linear", "Live", "Speech", "Playback", "User"];
const R_UF: &[&str] = &["FW", "USB"];
const R_US: &[&str] = &["32/32", "16/16", "32/8", "8/32", "8/8", "2/2"];
const R_CR: &[&str] = &["48K", "44K1"];
const R_CS: &[&str] = &["INT", "AES50A", "AES50B", "CARD"];
const R_SH: &[&str] = &["CUES", "SCENES", "SNIPPETS"];
const R_CL: &[&str] = &["24h", "12h"];
const R_IM: &[&str] = &["NORM", "INV"];
const R_ST: &[&str] = &["STOP", "PPAUSE", "PLAY", "RPAUSE", "REC", "FF", "REW"];
const R_S_SS: &[&str] = &[
    "CHAN", "METER", "ROUTE", "SETUP", "LIB", "FX", "MON", "USB", "SCENE", "ASSIGN", "LOCK",
];
const R_S_CP: &[&str] = &[
    "HOME", "FX1", "FX2", "FX3", "FX4", "FX5", "FX6", "FX7", "FX8",
];
const R_S_EP: &[&str] = &["CHANNEL", "MIXBUS", "AUX/FX", "IN/OUT", "RTA"];
const R_S_RP: &[&str] = &[
    "HOME", "ANAOUT", "AUXOUT", "P16OUT", "CARDOUT", "AES50A", "AES50B", "XLROUT",
];
const R_S_TP: &[&str] = &["GLOB", "CONF", "REMOTE", "NETW", "NAMES", "PREAMPS", "CARD"];
const R_S_LP: &[&str] = &["CHAN", "EFFECT", "ROUTE"];
const R_S_FP: &[&str] = &[
    "HOME", "FX1", "FX2", "FX3", "FX4", "FX5", "FX6", "FX7", "FX8",
];
const R_S_MP: &[&str] = &["MONITOR", "TALKA", "TALKB", "OSC"];
const R_S_UP: &[&str] = &["HOME", "CONFIG"];
const R_S_SP: &[&str] = &["HOME", "SCENES", "BITS", "PARSAFE", "CHNSAFE", "MIDI"];
const R_S_AP: &[&str] = &["HOME", "SETA", "SETB", "SETC"];
const R_S_EL: &[&str] = &[
    "Ch01", "Ch02", "Ch03", "Ch04", "Ch05", "Ch06", "Ch07", "Ch08", "Ch09", "Ch10", "Ch11", "Ch12",
    "Ch13", "Ch14", "Ch15", "Ch16", "Ch17", "Ch18", "Ch19", "Ch20", "Ch21", "Ch22", "Ch23", "Ch24",
    "Ch25", "Ch26", "Ch27", "Ch28", "Ch29", "Ch30", "Ch31", "Ch32", "Aux01", "Aux02", "Aux03",
    "Aux04", "Aux05", "Aux06", "Aux07", "Aux08", "Fx1L", "Fx1R", "Fx2L", "Fx2R", "Fx3L", "Fx3R",
    "Fx4L", "Fx4R", "Bus1", "Bus2", "Bus3", "Bus4", "Bus5", "Bus6", "Bus7", "Bus8", "Bus9", "Bs10",
    "Bs11", "Bs12", "Bs13", "Bs14", "Bs15", "Bs16", "Mtx1", "Mtx2", "Mtx3", "Mtx4", "Mtx5", "Mtx6",
    "LR", "M/C",
];
const R_B_AR: &[&str] = &["BAR", "SPEC"];
const R_P_RE: &[&str] = &["PRE", "POST"];
const R_R_MS: &[&str] = &["RMS", "PEAK"];
const R_R_PH: &[&str] = &["OFF", "1", "2", "3", "4", "5", "6", "7", "8"];
const R_R_TV: &[&str] = &[
    "OFF", "25%", "30%", "35%", "40%", "45%", "50%", "55%", "60%", "65%", "70%", "75%", "80%",
];

/// The main node parser function based on XDS_parse.
pub fn parse_node_line(line: &str) -> Option<OscMessage> {
    if !line.starts_with("/-") {
        return None;
    }

    // Split into command part and argument part
    // e.g. "/-stat/selidx Ch01" -> ["/-stat/selidx", "Ch01"]
    // Some commands like "/-stat/tape/file" have quoted arguments that may contain spaces.
    let (path, mut arg_str) = line.split_once(|c: char| c.is_whitespace())?;
    arg_str = arg_str.trim();

    if arg_str.is_empty() {
        return None;
    }

    match path {
        // --- stat ---
        "/-stat/selidx" => parse_list(path, arg_str, R_S_EL),
        "/-stat/chfaderbank"
        | "/-stat/grpfaderbank"
        | "/-stat/bussendbank"
        | "/-stat/eqband"
        | "/-stat/userbank"
        | "/-stat/rtasource"
        | "/-stat/xcardtype"
        | "/-stat/geqpos"
        | "/-stat/tape/etime"
        | "/-stat/tape/rtime" => parse_int(path, arg_str),
        "/-stat/sendsonfader"
        | "/-stat/solo"
        | "/-stat/keysolo"
        | "/-stat/autosave"
        | "/-stat/lock"
        | "/-stat/usbmounted"
        | "/-stat/remote"
        | "/-stat/rtaeqpre"
        | "/-stat/rtaeqpost"
        | "/-stat/xcardsync"
        | "/-stat/geqonfdr"
        | "/-stat/screen/mutegrp"
        | "/-stat/screen/utils" => parse_onoff(path, arg_str),
        "/-stat/rtamodeeq" | "/-stat/rtamodegeq" => parse_list(path, arg_str, R_B_AR),

        // --- stat screen ---
        "/-stat/screen/screen" => parse_list(path, arg_str, R_S_SS),
        "/-stat/screen/CHAN/page" => parse_list(path, arg_str, R_S_CP),
        "/-stat/screen/METER/page" => parse_list(path, arg_str, R_S_EP),
        "/-stat/screen/ROUTE/page" => parse_list(path, arg_str, R_S_RP),
        "/-stat/screen/SETUP/page" => parse_list(path, arg_str, R_S_TP),
        "/-stat/screen/LIB/page" => parse_list(path, arg_str, R_S_LP),
        "/-stat/screen/FX/page" => parse_list(path, arg_str, R_S_FP),
        "/-stat/screen/MON/page" => parse_list(path, arg_str, R_S_MP),
        "/-stat/screen/USB/page" => parse_list(path, arg_str, R_S_UP),
        "/-stat/screen/SCENE/page" => parse_list(path, arg_str, R_S_SP),
        "/-stat/screen/ASSIGN/page" => parse_list(path, arg_str, R_S_AP),

        // --- stat tape ---
        "/-stat/tape/state" => parse_list(path, arg_str, R_ST),
        "/-stat/tape/file" => parse_str(path, arg_str),

        // --- stat osc ---
        "/-stat/osc" => parse_onoff(path, arg_str),

        // --- stat solosw (0..79 lines handled in loop in C, but here we parse line-by-line) ---
        p if p.starts_with("/-stat/solosw/") => parse_onoff(path, arg_str),

        // --- prefs ---
        "/-prefs/style" => parse_str(path, arg_str),
        "/-prefs/bright" | "/-prefs/ledbright" => parse_flin(path, arg_str, 10.0, 90.0, 5.0),
        "/-prefs/lcdcont" => parse_flin(path, arg_str, 0.0, 100.0, 2.0),
        "/-prefs/lamp" => parse_flin(path, arg_str, 10.0, 90.0, 10.0),
        "/-prefs/lampon"
        | "/-prefs/confirm_general"
        | "/-prefs/confirm_overwrite"
        | "/-prefs/confirm_sceneload"
        | "/-prefs/viewrtn"
        | "/-prefs/selfollowbank"
        | "/-prefs/sceneadvance"
        | "/-prefs/safe_masterlevels"
        | "/-prefs/autosel"
        | "/-prefs/hardmute"
        | "/-prefs/dcsmute" => parse_onoff(path, arg_str),
        "/-prefs/clockrate" => parse_list(path, arg_str, R_CR),
        "/-prefs/clocksource" => parse_list(path, arg_str, R_CS),
        "/-prefs/show_control" => parse_list(path, arg_str, R_SH),
        "/-prefs/clockmode" => parse_list(path, arg_str, R_CL),
        "/-prefs/invertmute" => parse_list(path, arg_str, R_IM),
        "/-prefs/haflags" => parse_bits(path, arg_str),

        // --- prefs card ---
        "/-prefs/card/UFifc" => parse_list(path, arg_str, R_UF),
        "/-prefs/card/UFmode" | "/-prefs/card/USBmode" => parse_list(path, arg_str, R_US),

        // --- prefs rta ---
        "/-prefs/rta/visibility" => parse_list(path, arg_str, R_R_TV),
        "/-prefs/rta/gain" => parse_flin(path, arg_str, 0.0, 60.0, 6.0),
        "/-prefs/rta/autogain" => parse_onoff(path, arg_str),
        "/-prefs/rta/source" => parse_int(path, arg_str),
        "/-prefs/rta/pos" => parse_list(path, arg_str, R_P_RE),
        "/-prefs/rta/mode" => parse_list(path, arg_str, R_B_AR),
        "/-prefs/rta/option" => parse_bits(path, arg_str),
        "/-prefs/rta/det" => parse_list(path, arg_str, R_R_MS),
        "/-prefs/rta/decay" => parse_flog(path, arg_str, 0.25, 15.75, 19.0),
        "/-prefs/rta/peakhold" => parse_list(path, arg_str, R_R_PH),

        // --- prefs ip ---
        "/-prefs/ip/dhcp" => parse_onoff(path, arg_str),
        "/-prefs/ip/addr/0"
        | "/-prefs/ip/addr/1"
        | "/-prefs/ip/addr/2"
        | "/-prefs/ip/addr/3"
        | "/-prefs/ip/mask/0"
        | "/-prefs/ip/mask/1"
        | "/-prefs/ip/mask/2"
        | "/-prefs/ip/mask/3"
        | "/-prefs/ip/gateway/0"
        | "/-prefs/ip/gateway/1"
        | "/-prefs/ip/gateway/2"
        | "/-prefs/ip/gateway/3" => parse_int(path, arg_str),

        // --- prefs remote ---
        "/-prefs/remote/enable" => parse_onoff(path, arg_str),
        "/-prefs/remote/protocol" => parse_list(path, arg_str, R_PROT),
        "/-prefs/remote/port" => parse_list(path, arg_str, R_PORT),
        "/-prefs/remote/ioenable" => parse_bits(path, arg_str),

        // --- prefs iQ ---
        p if p.starts_with("/-prefs/iQ/") => {
            let iq_path = p;
            if iq_path.ends_with("/iQmodel") {
                parse_list(iq_path, arg_str, R_IQM)
            } else if iq_path.ends_with("/iQeqset") {
                parse_list(iq_path, arg_str, R_IQE)
            } else if iq_path.ends_with("/iQsound") {
                parse_int(iq_path, arg_str)
            } else {
                None
            }
        }

        _ => None,
    }
}

fn parse_str(path: &str, val: &str) -> Option<OscMessage> {
    // Strings in node files are typically quoted, e.g. "Some Name"
    let clean_val = val.trim_matches('"');
    Some(OscMessage::new(
        path.to_string(),
        vec![OscArg::String(clean_val.to_string())],
    ))
}

fn parse_int(path: &str, val: &str) -> Option<OscMessage> {
    val.parse::<i32>()
        .ok()
        .map(|i| OscMessage::new(path.to_string(), vec![OscArg::Int(i)]))
}

fn parse_flin(path: &str, val: &str, xmin: f32, lmaxmin: f32, xstep: f32) -> Option<OscMessage> {
    let mut v = val.parse::<f32>().ok()?;
    v = (v - xmin) / lmaxmin;
    let step = lmaxmin / xstep;
    v = (v * step).round() / step;
    if v <= 0.0 {
        v = 0.0;
    }
    if v > 1.0 {
        v = 1.0;
    }
    Some(OscMessage::new(path.to_string(), vec![OscArg::Float(v)]))
}

fn parse_flog(path: &str, val: &str, xmin: f32, lmaxmin: f32, nsteps: f32) -> Option<OscMessage> {
    let mut v = val.parse::<f32>().ok()?;
    v = (v / xmin).ln() / lmaxmin;
    v = (v * nsteps).round() / nsteps;
    if v <= 0.0 {
        v = 0.0;
    }
    if v > 1.0 {
        v = 1.0;
    }
    Some(OscMessage::new(path.to_string(), vec![OscArg::Float(v)]))
}

fn parse_bits(path: &str, val: &str) -> Option<OscMessage> {
    if !val.starts_with('%') {
        return None;
    }
    let mut bits = 0;
    for c in val[1..].chars() {
        bits <<= 1;
        if c == '1' {
            bits |= 1;
        } else if c != '0' {
            return None;
        }
    }
    Some(OscMessage::new(path.to_string(), vec![OscArg::Int(bits)]))
}

fn parse_onoff(path: &str, val: &str) -> Option<OscMessage> {
    let v = if val == "OFF" { 0 } else { 1 };
    Some(OscMessage::new(path.to_string(), vec![OscArg::Int(v)]))
}

fn parse_list(path: &str, val: &str, list: &[&str]) -> Option<OscMessage> {
    let index = list.iter().position(|&s| s == val)?;
    Some(OscMessage::new(
        path.to_string(),
        vec![OscArg::Int(index as i32)],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_node_line_list() {
        let msg = parse_node_line("/-stat/selidx Ch01").unwrap();
        assert_eq!(msg.path, "/-stat/selidx");
        assert_eq!(msg.args, vec![OscArg::Int(0)]);

        let msg2 = parse_node_line("/-stat/screen/METER/page MIXBUS").unwrap();
        assert_eq!(msg2.path, "/-stat/screen/METER/page");
        assert_eq!(msg2.args, vec![OscArg::Int(1)]);
    }

    #[test]
    fn test_parse_node_line_onoff() {
        let msg = parse_node_line("/-stat/sendsonfader ON").unwrap();
        assert_eq!(msg.args, vec![OscArg::Int(1)]);

        let msg2 = parse_node_line("/-stat/sendsonfader OFF").unwrap();
        assert_eq!(msg2.args, vec![OscArg::Int(0)]);
    }

    #[test]
    fn test_parse_node_line_int() {
        let msg = parse_node_line("/-stat/chfaderbank 2").unwrap();
        assert_eq!(msg.args, vec![OscArg::Int(2)]);
    }

    #[test]
    fn test_parse_node_line_flin() {
        // bright -> xmin=10, lmaxmin=90, xstep=5
        // e.g. val=50 -> v=(50-10)/90 = 40/90 = 0.4444
        // step = 90/5 = 18.0
        // v = round(0.4444 * 18.0) / 18.0 = round(8.0) / 18.0 = 8/18 = 0.4444...
        let msg = parse_node_line("/-prefs/bright 50.0").unwrap();
        if let OscArg::Float(f) = msg.args[0] {
            assert!((f - 0.44444445).abs() < 0.0001);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_parse_node_line_bits() {
        let msg = parse_node_line("/-prefs/rta/option %0000000000000101").unwrap();
        assert_eq!(msg.args, vec![OscArg::Int(5)]);
    }

    #[test]
    fn test_parse_node_line_str() {
        let msg = parse_node_line("/-stat/tape/file \"test_file.wav\"").unwrap();
        assert_eq!(msg.args, vec![OscArg::String("test_file.wav".to_string())]);

        let msg2 = parse_node_line("/-stat/tape/file \"test file with spaces.wav\"").unwrap();
        assert_eq!(
            msg2.args,
            vec![OscArg::String("test file with spaces.wav".to_string())]
        );
    }
}
