use lazy_static::lazy_static;
use osc_lib::{OscArg, OscMessage};
use regex::Regex;

lazy_static! {
    static ref RE_FLOAT: Regex = Regex::new(r"^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)$").unwrap();
    static ref RE_INT: Regex = Regex::new(r"^[+-]?[0-9]+$").unwrap();
}

/// Parses a shorthand scene line into standard OscMessages.
/// This translates compact .scn and .snp commands into valid OSC messages.
pub fn parse_scene_line(line: &str) -> Option<Vec<OscMessage>> {
    let mut parts = line.splitn(2, ' ');
    let path = parts.next()?;
    let value_str = parts.next().unwrap_or("").trim();

    // Fast-path out if it's already a well-formed OSC message with a type tag string
    if value_str.starts_with(',') {
        return None;
    }

    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in value_str.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current_arg.push(c);
            }
            ' ' if !in_quotes => {
                if !current_arg.is_empty() {
                    args.push(parse_single_value(&current_arg));
                    current_arg.clear();
                }
            }
            _ => current_arg.push(c),
        }
    }

    if !current_arg.is_empty() {
        args.push(parse_single_value(&current_arg));
    }

    Some(vec![OscMessage::new(path.to_string(), args)])
}

fn parse_single_value(val: &str) -> OscArg {
    if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
        return OscArg::String(val[1..val.len() - 1].to_string());
    }

    if val == "ON" {
        return OscArg::Int(1);
    } else if val == "OFF" {
        return OscArg::Int(0);
    }

    if val == "-oo" {
        return OscArg::Float(0.0);
    }

    if RE_INT.is_match(val) {
        if let Ok(i) = val.parse::<i32>() {
            return OscArg::Int(i);
        }
    }

    if RE_FLOAT.is_match(val) {
        if let Ok(f) = val.parse::<f32>() {
            return OscArg::Float(f);
        }
    }

    // If it has a %, it's a bitmask
    if val.contains('%') {
        let mut bitmask = 0;
        // OPTIMIZATION: Use .as_bytes() instead of .chars() for purely binary strings
        // to bypass UTF-8 decoding overhead.
        let bits = val.trim_matches('%').as_bytes();
        for (i, &b) in bits.iter().rev().enumerate() {
            if b == b'1' {
                bitmask |= 1 << i;
            }
        }
        return OscArg::Int(bitmask);
    }

    // Default to string
    OscArg::String(val.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_float() {
        let msgs = parse_scene_line("/ch/01/mix/fader -9.9").unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].path, "/ch/01/mix/fader");
        assert_eq!(msgs[0].args, vec![OscArg::Float(-9.9)]);
    }

    #[test]
    fn test_parse_on_off() {
        let msgs = parse_scene_line("/ch/01/mix/on ON").unwrap();
        assert_eq!(msgs[0].args, vec![OscArg::Int(1)]);
    }

    #[test]
    fn test_parse_string() {
        let msgs = parse_scene_line(r#"/ch/01/config/name "SCENE_SET""#).unwrap();
        assert_eq!(msgs[0].args, vec![OscArg::String("SCENE_SET".to_string())]);
    }

    #[test]
    fn test_parse_minus_oo() {
        let msgs = parse_scene_line("/ch/01/mix/fader -oo").unwrap();
        assert_eq!(msgs[0].args, vec![OscArg::Float(0.0)]);
    }
}
