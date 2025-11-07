//! # Reaper Handler Module
//!
//! Handles OSC messages received from the Reaper DAW.

use anyhow::anyhow;
use osc_lib::{OscArg, OscMessage};

use crate::config::Config;
use crate::state::AppState;

/// Handles a raw OSC message from Reaper.
pub fn handle_reaper_message(bytes: &[u8], _config: &Config, _app_state: &mut AppState) -> anyhow::Result<Vec<OscMessage>> {
    let mut messages_to_x32 = Vec::new();
    let mut messages_from_reaper = Vec::new();

    if bytes.starts_with(b"#bundle\0") {
        let mut cursor = 16;
        while cursor < bytes.len() {
            if cursor + 4 > bytes.len() {
                return Err(anyhow!("Malformed bundle: not enough bytes for size"));
            }
            let mut size_bytes = [0u8; 4];
            size_bytes.copy_from_slice(&bytes[cursor..cursor + 4]);
            let size = u32::from_be_bytes(size_bytes) as usize;
            cursor += 4;

            if cursor + size > bytes.len() {
                return Err(anyhow!("Malformed bundle: message size exceeds buffer length"));
            }

            let msg_bytes = &bytes[cursor..cursor + size];
            let msg = OscMessage::from_bytes(msg_bytes)?;
            messages_from_reaper.push(msg);
            cursor += size;
        }
    } else {
        let msg = OscMessage::from_bytes(bytes)?;
        messages_from_reaper.push(msg);
    }

    for msg in messages_from_reaper {
        messages_to_x32.extend(process_message(msg, _config, _app_state));
    }

    Ok(messages_to_x32)
}

/// Processes a single OSC message from Reaper.
fn process_message(msg: OscMessage, config: &Config, app_state: &mut AppState) -> Vec<OscMessage> {
    let mut messages = Vec::new();
    let path_parts: Vec<&str> = msg.path.split('/').collect();

    if path_parts.len() < 2 {
        return messages;
    }

    if path_parts[1] == "track" {
        if path_parts.len() < 3 {
            return messages;
        }
        let tnum: u8 = path_parts[2].parse().ok().unwrap_or(0);

        if path_parts.len() < 4 {
            return messages;
        }

        match path_parts[3] {
            "volume" => {
                if let Some(OscArg::Float(level)) = msg.args.get(0) {
                    if let Some((x32_path, ch_num)) = get_x32_path_from_track_num(tnum, "mix/fader", config, app_state) {
                        let track_index = (app_state.channel_bank_offset as usize * config.bank_size as usize) + ch_num as usize -1;
                        app_state.track_states[track_index].fader = *level;
                        let x32_level = (*level * 1023.5).round() / 1023.0;
                        messages.push(OscMessage::new(x32_path, vec![OscArg::Float(x32_level)]));
                    }
                }
            },
            "pan" => {
                if let Some(OscArg::Float(level)) = msg.args.get(0) {
                    if let Some((x32_path, ch_num)) = get_x32_path_from_track_num(tnum, "mix/pan", config, app_state) {
                        let track_index = (app_state.channel_bank_offset as usize * config.bank_size as usize) + ch_num as usize -1;
                        app_state.track_states[track_index].pan = *level;
                        messages.push(OscMessage::new(x32_path, vec![OscArg::Float(*level)]));
                    }
                }
            },
            "mute" => {
                if let Some(OscArg::Float(level)) = msg.args.get(0) {
                    if let Some((x32_path, ch_num)) = get_x32_path_from_track_num(tnum, "mix/on", config, app_state) {
                        let track_index = (app_state.channel_bank_offset as usize * config.bank_size as usize) + ch_num as usize -1;
                        let muted = *level > 0.5;
                        app_state.track_states[track_index].mute = muted;
                        messages.push(OscMessage::new(x32_path, vec![OscArg::Int(if muted { 0 } else { 1 })]));
                    }
                }
            },
            "name" => {
                if let Some(OscArg::String(name)) = msg.args.get(0) {
                    if let Some((x32_path, ch_num)) = get_x32_path_from_track_num(tnum, "config/name", config, app_state) {
                        let track_index = (app_state.channel_bank_offset as usize * config.bank_size as usize) + ch_num as usize -1;
                        app_state.track_states[track_index].scribble = name.to_string();
                        messages.push(OscMessage::new(x32_path, vec![OscArg::String(name.to_string())]));
                    }
                }
            },
            "fx" => {
                if path_parts.len() > 5 && path_parts[4] == "fxparam" {
                    let param_num: u8 = path_parts[5].parse().ok().unwrap_or(0);
                    if let Some(OscArg::Float(level)) = msg.args.get(0) {
                        if let Some((x32_path_prefix, _)) = get_x32_path_from_track_num(tnum, "eq", config, app_state) {
                            let (band, param) = match param_num {
                                1 => (1, "f"), 2 => (1, "g"), 3 => (1, "q"),
                                4 => (2, "f"), 5 => (2, "g"), 6 => (2, "q"),
                                7 => (3, "f"), 8 => (3, "g"), 9 => (3, "q"),
                                10 => (4, "f"), 11 => (4, "g"), 12 => (4, "q"),
                                _ => return messages,
                            };
                            let x32_path = format!("{}/{}/{}", x32_path_prefix, band, param);
                            messages.push(OscMessage::new(x32_path, vec![OscArg::Float(*level)]));
                        }
                    }
                }
            },
            "send" => {
                if path_parts.len() > 5 && path_parts[5] == "volume" {
                    let send_num: u8 = path_parts[4].parse().ok().unwrap_or(0);
                    if let Some(OscArg::Float(level)) = msg.args.get(0) {
                        if let Some((x32_path, _)) = get_x32_path_from_track_num(tnum, &format!("mix/{:02}/level", send_num), config, app_state) {
                            messages.push(OscMessage::new(x32_path, vec![OscArg::Float(*level)]));
                        }
                    }
                }
            },
            "solo" => {
                if let Some(OscArg::Float(level)) = msg.args.get(0) {
                    if let Some((_, ch_num)) = get_x32_path_from_track_num(tnum, "", config, app_state) {
                        let soloed = *level > 0.5;
                        let x32_path = format!("/-stat/solosw/{}", ch_num);
                        messages.push(OscMessage::new(x32_path, vec![OscArg::Int(if soloed { 1 } else { 0 })]));
                    }
                }
            }
            _ => {
                println!("Unhandled reaper message: {}", msg.path);
            }
        }
    }

    messages
}

/// Gets the X32 OSC path for a given Reaper track number.
fn get_x32_path_from_track_num(tnum: u8, suffix: &str, config: &Config, app_state: &AppState) -> Option<(String, u8)> {
    if tnum >= config.channel_map.min && tnum <= config.channel_map.max {
        let ch_num = tnum - config.channel_map.min + 1;
        let bank_offset = app_state.channel_bank_offset * config.bank_size;
        if ch_num > bank_offset && ch_num <= bank_offset + config.bank_size {
            let ch_num_in_bank = ch_num - bank_offset;
            return Some((format!("/ch/{:02}/{}", ch_num_in_bank, suffix), ch_num));
        }
    } else if tnum >= config.aux_map.min && tnum <= config.aux_map.max {
        let aux_num = tnum - config.aux_map.min + 1;
        return Some((format!("/auxin/{:02}/{}", aux_num, suffix), aux_num));
    } else if tnum >= config.fx_return_map.min && tnum <= config.fx_return_map.max {
        let fxr_num = tnum - config.fx_return_map.min + 1;
        return Some((format!("/fxrtn/{:02}/{}", fxr_num, suffix), fxr_num));
    } else if tnum >= config.bus_map.min && tnum <= config.bus_map.max {
        let bus_num = tnum - config.bus_map.min + 1;
        return Some((format!("/bus/{:02}/{}", bus_num, suffix), bus_num));
    } else if tnum >= config.dca_map.min && tnum <= config.dca_map.max {
        let dca_num = tnum - config.dca_map.min + 1;
        return Some((format!("/dca/{}/{}", dca_num, suffix), dca_num));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use osc_lib::{OscArg, OscMessage};

    #[test]
    fn test_reaper_fader_message() {
        let config = Config::default();
        let mut app_state = AppState::new(&config);
        let msg = OscMessage::new("/track/1/volume".to_string(), vec![OscArg::Float(0.5)]);
        let bytes = msg.to_bytes().unwrap();

        let result = handle_reaper_message(&bytes, &config, &mut app_state).unwrap();
        assert_eq!(result.len(), 1);
        let x32_msg = &result[0];
        assert_eq!(x32_msg.path, "/ch/01/mix/fader");
        if let Some(OscArg::Float(level)) = x32_msg.args.get(0) {
            assert!((level - 0.5).abs() < 0.001);
        } else {
            panic!("Expected float argument");
        }
    }

    #[test]
    fn test_reaper_pan_message() {
        let config = Config::default();
        let mut app_state = AppState::new(&config);
        let msg = OscMessage::new("/track/1/pan".to_string(), vec![OscArg::Float(0.25)]);
        let bytes = msg.to_bytes().unwrap();

        let result = handle_reaper_message(&bytes, &config, &mut app_state).unwrap();
        assert_eq!(result.len(), 1);
        let x32_msg = &result[0];
        assert_eq!(x32_msg.path, "/ch/01/mix/pan");
        if let Some(OscArg::Float(level)) = x32_msg.args.get(0) {
            assert_eq!(*level, 0.25);
        } else {
            panic!("Expected float argument");
        }
    }

    #[test]
    fn test_reaper_mute_message() {
        let config = Config::default();
        let mut app_state = AppState::new(&config);
        let msg = OscMessage::new("/track/1/mute".to_string(), vec![OscArg::Float(1.0)]);
        let bytes = msg.to_bytes().unwrap();

        let result = handle_reaper_message(&bytes, &config, &mut app_state).unwrap();
        assert_eq!(result.len(), 1);
        let x32_msg = &result[0];
        assert_eq!(x32_msg.path, "/ch/01/mix/on");
        if let Some(OscArg::Int(level)) = x32_msg.args.get(0) {
            assert_eq!(*level, 0);
        } else {
            panic!("Expected int argument");
        }
    }

    #[test]
    fn test_reaper_name_message() {
        let config = Config::default();
        let mut app_state = AppState::new(&config);
        let msg = OscMessage::new("/track/1/name".to_string(), vec![OscArg::String("Test".to_string())]);
        let bytes = msg.to_bytes().unwrap();

        let result = handle_reaper_message(&bytes, &config, &mut app_state).unwrap();
        assert_eq!(result.len(), 1);
        let x32_msg = &result[0];
        assert_eq!(x32_msg.path, "/ch/01/config/name");
        if let Some(OscArg::String(name)) = x32_msg.args.get(0) {
            assert_eq!(name, "Test");
        } else {
            panic!("Expected string argument");
        }
    }

    #[test]
    fn test_reaper_eq_message() {
        let config = Config::default();
        let mut app_state = AppState::new(&config);
        let msg = OscMessage::new("/track/1/fx/fxparam/1/value".to_string(), vec![OscArg::Float(0.5)]);
        let bytes = msg.to_bytes().unwrap();

        let result = handle_reaper_message(&bytes, &config, &mut app_state).unwrap();
        assert_eq!(result.len(), 1);
        let x32_msg = &result[0];
        assert_eq!(x32_msg.path, "/ch/01/eq/1/f");
        if let Some(OscArg::Float(level)) = x32_msg.args.get(0) {
            assert_eq!(*level, 0.5);
        } else {
            panic!("Expected float argument");
        }
    }

    #[test]
    fn test_reaper_send_message() {
        let config = Config::default();
        let mut app_state = AppState::new(&config);
        let msg = OscMessage::new("/track/1/send/1/volume".to_string(), vec![OscArg::Float(0.5)]);
        let bytes = msg.to_bytes().unwrap();

        let result = handle_reaper_message(&bytes, &config, &mut app_state).unwrap();
        assert_eq!(result.len(), 1);
        let x32_msg = &result[0];
        assert_eq!(x32_msg.path, "/ch/01/mix/01/level");
        if let Some(OscArg::Float(level)) = x32_msg.args.get(0) {
            assert_eq!(*level, 0.5);
        } else {
            panic!("Expected float argument");
        }
    }

    #[test]
    fn test_reaper_solo_message() {
        let config = Config::default();
        let mut app_state = AppState::new(&config);
        let msg = OscMessage::new("/track/1/solo".to_string(), vec![OscArg::Float(1.0)]);
        let bytes = msg.to_bytes().unwrap();

        let result = handle_reaper_message(&bytes, &config, &mut app_state).unwrap();
        assert_eq!(result.len(), 1);
        let x32_msg = &result[0];
        assert_eq!(x32_msg.path, "/-stat/solosw/1");
        if let Some(OscArg::Int(level)) = x32_msg.args.get(0) {
            assert_eq!(*level, 1);
        } else {
            panic!("Expected int argument");
        }
    }
}
