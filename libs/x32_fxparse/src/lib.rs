#![allow(
    clippy::excessive_precision,
    clippy::approx_constant,
    clippy::collapsible_if,
    clippy::manual_range_contains
)]
use osc_lib::{OscArg, OscMessage};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixerModel {
    X32,
    XR18,
    XR16,
    XR12,
}

struct ModelLimits {
    channels: usize,
    buses: usize,
    auxins: usize,
    fxrtns: usize,
    matrices: usize,
    dcas: usize,
    fx_slots: usize,
    channel_eq_bands: usize,
    auxin_eq_bands: usize,
    fxrtn_eq_bands: usize,
    bus_eq_bands: usize,
    mtx_eq_bands: usize,
    main_eq_bands: usize,
}

fn parse_float(s: Option<&str>) -> f32 {
    s.and_then(|v| f32::from_str(v).ok()).unwrap_or(0.0)
}

fn log2float(val: f32, a: f32, b: f32) -> f32 {
    if val <= a {
        0.0
    } else {
        ((val / a).ln() / b).clamp(0.0, 1.0)
    }
}

fn afine2float(val: f32, a: f32, b: f32) -> f32 {
    ((val - a) / b).clamp(0.0, 1.0)
}

fn ratio2float(val: f32, a: f32) -> f32 {
    (val / a).clamp(0.0, 1.0)
}

fn parse_string(val: &str) -> Option<OscArg> {
    let mut v = val.trim();
    if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
        v = &v[1..v.len() - 1];
    }
    Some(OscArg::String(v.to_string()))
}

fn parse_enum(val: &str, range: &[&str]) -> Option<OscArg> {
    let val = val.trim();
    if val.starts_with('%') {
        return parse_bits(val);
    }
    if let Ok(idx) = val.parse::<i32>() {
        return Some(OscArg::Int(idx));
    }
    range
        .iter()
        .position(|&x| x == val)
        .map(|i| OscArg::Int(i as i32))
}

fn parse_enum_fx(s: Option<&str>, range: &[&str]) -> i32 {
    let s = match s {
        Some(val) => val,
        None => return 0,
    };
    if let Ok(idx) = s.parse::<i32>() {
        return idx;
    }
    range
        .iter()
        .position(|&x| x == s)
        .map(|i| i as i32)
        .unwrap_or(0)
}

fn parse_level(val: &str) -> Option<OscArg> {
    let val = val.trim();
    if val == "-oo" {
        return Some(OscArg::Float(0.0));
    }
    let mut fval = val.parse::<f32>().ok()?;
    if fval < -60.0 {
        fval = fval * 0.002_083_333_4 + 0.1875;
        fval = (fval * 1023.0).round() / 1023.0;
        if fval < 0.0 {
            fval = 0.0;
        }
    } else if fval < -30.0 {
        fval = 0.00625 * fval + 0.4375;
        fval = (fval * 1023.0).round() / 1023.0;
    } else if fval < -10.0 {
        fval = 0.0125 * fval + 0.625;
        fval = (fval * 1023.0).round() / 1023.0;
    } else {
        fval = fval * 0.025 + 0.75;
        if fval > 1.0 {
            fval = 1.0;
        }
    }
    Some(OscArg::Float(fval))
}

fn parse_frequency(val: &str) -> Option<OscArg> {
    let fval = parse_float_raw(val)?;
    let mut res = (fval / 20.0).ln() / 6.907_755_4;
    res = (res * 200.0).round() / 200.0;
    Some(OscArg::Float(res.clamp(0.0, 1.0)))
}

fn parse_flin(val: &str, min: f32, max: f32) -> Option<OscArg> {
    let fval = parse_float_raw(val)?;
    let res = (fval - min) / (max - min);
    Some(OscArg::Float(res.clamp(0.0, 1.0)))
}

fn parse_logf(val: &str, min: f32, max: f32) -> Option<OscArg> {
    let fval = parse_float_raw(val)?;
    if fval <= min {
        Some(OscArg::Float(0.0))
    } else {
        let res = (fval / min).ln() / (max / min).ln();
        Some(OscArg::Float(res.clamp(0.0, 1.0)))
    }
}

fn parse_bits(val: &str) -> Option<OscArg> {
    let val = val.trim();
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

fn parse_float_raw(val: &str) -> Option<f32> {
    let val = val.trim();
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

/// Parses any general channel-strip, bus, matrix, or configuration parameter.
pub fn parse_parameter(model: MixerModel, path: &str, arg_str: &str) -> Option<OscMessage> {
    if arg_str.is_empty() {
        return None;
    }

    let limits = match model {
        MixerModel::X32 => ModelLimits {
            channels: 32,
            buses: 16,
            auxins: 8,
            fxrtns: 8,
            matrices: 6,
            dcas: 8,
            fx_slots: 8,
            channel_eq_bands: 4,
            auxin_eq_bands: 2,
            fxrtn_eq_bands: 2,
            bus_eq_bands: 6,
            mtx_eq_bands: 6,
            main_eq_bands: 6,
        },
        MixerModel::XR18 => ModelLimits {
            channels: 16,
            buses: 6,
            auxins: 2,
            fxrtns: 4,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            channel_eq_bands: 4,
            auxin_eq_bands: 4,
            fxrtn_eq_bands: 4,
            bus_eq_bands: 6,
            mtx_eq_bands: 0,
            main_eq_bands: 6,
        },
        MixerModel::XR16 => ModelLimits {
            channels: 16,
            buses: 4,
            auxins: 2,
            fxrtns: 4,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            channel_eq_bands: 4,
            auxin_eq_bands: 4,
            fxrtn_eq_bands: 4,
            bus_eq_bands: 6,
            mtx_eq_bands: 0,
            main_eq_bands: 6,
        },
        MixerModel::XR12 => ModelLimits {
            channels: 12,
            buses: 2,
            auxins: 2,
            fxrtns: 4,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            channel_eq_bands: 4,
            auxin_eq_bands: 4,
            fxrtn_eq_bands: 4,
            bus_eq_bands: 6,
            mtx_eq_bands: 0,
            main_eq_bands: 6,
        },
    };

    let mut parts_arr = [""; 8];
    let parts = osc_lib::extract_segments(path.trim_start_matches('/'), '/', &mut parts_arr);

    match model {
        MixerModel::X32 => {
            include!(concat!(env!("OUT_DIR"), "/channel_parameters_x32_gen.rs"));
        }
        MixerModel::XR18 => {
            include!(concat!(env!("OUT_DIR"), "/channel_parameters_xr18_gen.rs"));
        }
        MixerModel::XR16 => {
            include!(concat!(env!("OUT_DIR"), "/channel_parameters_xr16_gen.rs"));
        }
        MixerModel::XR12 => {
            include!(concat!(env!("OUT_DIR"), "/channel_parameters_xr12_gen.rs"));
        }
    }

    None
}

/// Parses FX parameters from a scene file line according to the active FX type.
/// The `ifx` is the effect type index (e.g., HALL = 0).
/// `arg_str` contains the space-separated string of parameter values.
#[allow(unused_assignments)]
pub fn parse_fx_par(ifx: usize, arg_str: &str, path: &str) -> Option<OscMessage> {
    if arg_str.is_empty() {
        return None;
    }

    let mut parts = arg_str.split_whitespace();
    // Peak at the first item to ensure it's not empty, mirroring the previous check
    parts.clone().next()?;

    let mut args = Vec::new();

    include!(concat!(env!("OUT_DIR"), "/fx_parameters_gen.rs"));

    Some(OscMessage::new(path.to_string(), args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hall_reverb() {
        let path = "/fx/1/par";
        let arg_str = "100.0 1.0 51.0 4472.0 15.5 0.0 70.7 2000.0 1.0 25.0 125.0 50.0";

        let msg = parse_fx_par(0, arg_str, path).expect("Should parse HALL parameters");
        assert_eq!(msg.path, path);
        assert_eq!(msg.args.len(), 12);

        for arg in msg.args {
            if let OscArg::Float(f) = arg {
                assert!((f - 0.5).abs() < 0.01, "Expected ~0.5, got {}", f);
            } else {
                panic!("Expected float argument");
            }
        }
    }

    #[test]
    fn test_parse_plate_reverb() {
        let path = "/fx/2/par";
        let arg_str = "100.0 2.236 51.0 4472.0 15.5 0.0 70.7 2000.0 1.0 70.7 25.0 50.0";

        let msg = parse_fx_par(5, arg_str, path).expect("Should parse PLAT parameters");
        assert_eq!(msg.args.len(), 12);

        for arg in msg.args {
            if let OscArg::Float(f) = arg {
                assert!((f - 0.5).abs() < 0.01, "Expected ~0.5, got {}", f);
            }
        }
    }

    #[test]
    fn test_parse_fx_with_empty_str() {
        assert!(parse_fx_par(0, "", "/fx/1/par").is_none());
    }

    #[test]
    fn test_parse_parameter_fader_x32() {
        let msg =
            parse_parameter(MixerModel::X32, "/ch/01/mix/fader", "-10.0").expect("Parse fader");
        assert_eq!(msg.path, "/ch/01/mix/fader");
        if let OscArg::Float(f) = msg.args[0] {
            assert!(f > 0.0 && f <= 1.0);
        } else {
            panic!("Expected float arg");
        }
    }

    #[test]
    fn test_parse_parameter_fader_xr18() {
        let msg_ok = parse_parameter(MixerModel::XR18, "/ch/16/mix/fader", "0.0");
        assert!(msg_ok.is_some());

        let msg_none = parse_parameter(MixerModel::XR18, "/ch/32/mix/fader", "0.0");
        assert!(msg_none.is_none());
    }

    #[test]
    fn test_parse_parameter_auxin_limits() {
        let x32_ok = parse_parameter(MixerModel::X32, "/auxin/08/mix/fader", "0.0");
        assert!(x32_ok.is_some());

        let xr18_none = parse_parameter(MixerModel::XR18, "/auxin/08/mix/fader", "0.0");
        assert!(xr18_none.is_none());
    }

    #[test]
    fn test_parse_parameter_eq_bands_limits() {
        // Aux EQ bands X32 has 2, XR18 has 4
        let x32_band3 = parse_parameter(MixerModel::X32, "/auxin/01/eq/3/f", "1000.0");
        assert!(x32_band3.is_none());

        let xr18_band3 = parse_parameter(MixerModel::XR18, "/auxin/01/eq/3/f", "1000.0");
        assert!(xr18_band3.is_some());
    }

    #[test]
    fn test_parse_parameter_chlink_limits() {
        let x32_link = parse_parameter(MixerModel::X32, "/config/chlink/31-32", "ON");
        assert!(x32_link.is_some());

        let xr18_link = parse_parameter(MixerModel::XR18, "/config/chlink/31-32", "ON");
        assert!(xr18_link.is_none());
    }

    #[test]
    fn test_parse_parameter_xr18_removed_overrides() {
        // Mono mode: valid on X32, invalid on XR18 (removed via overrides)
        assert!(parse_parameter(MixerModel::X32, "/config/mono/mode", "LR+M").is_some());
        assert!(parse_parameter(MixerModel::XR18, "/config/mono/mode", "LR+M").is_none());

        // Main mono fader: valid on X32, invalid on XR18 (removed via overrides)
        assert!(parse_parameter(MixerModel::X32, "/main/m/mix/fader", "-10.0").is_some());
        assert!(parse_parameter(MixerModel::XR18, "/main/m/mix/fader", "-10.0").is_none());
    }
}
