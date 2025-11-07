//! # X32 Handler Module
//!
//! Handles OSC messages received from the X32/M32 console.

use osc_lib::{OscArg, OscMessage};

use crate::config::Config;
use crate::state::AppState;

/// Handles a raw OSC message from the X32.
pub fn handle_x32_message(bytes: &[u8], config: &Config, app_state: &mut AppState) -> anyhow::Result<Vec<OscMessage>> {
    let msg = OscMessage::from_bytes(bytes)?;
    let mut messages = Vec::new();

    let path_parts: Vec<&str> = msg.path.split('/').collect();

    if path_parts.len() < 3 {
        return Ok(messages);
    }

    let prefix = path_parts[1];

    if prefix == "ch" || prefix == "auxin" || prefix == "fxrtn" || prefix == "bus" {
        let num: u8 = path_parts[2].parse().ok().unwrap_or(0);

        if path_parts.len() < 4 {
            return Ok(messages);
        }

        if path_parts[3] == "mix" {
            if path_parts.len() < 5 {
                return Ok(messages);
            }
            if let Some(tnum) = get_track_num_from_x32_path(prefix, num, config, app_state) {
                match path_parts[4] {
                    "fader" => {
                        if let Some(OscArg::Float(level)) = msg.args.get(0) {
                            let reaper_path = format!("/track/{}/volume", tnum);
                            messages.push(OscMessage::new(reaper_path, vec![OscArg::Float(*level)]));
                        }
                    },
                    "pan" => {
                        if let Some(OscArg::Float(level)) = msg.args.get(0) {
                            let reaper_path = format!("/track/{}/pan", tnum);
                            messages.push(OscMessage::new(reaper_path, vec![OscArg::Float(*level)]));
                        }
                    },
                    "on" => {
                        if let Some(OscArg::Int(level)) = msg.args.get(0) {
                            let muted = *level == 0;
                            let reaper_path = format!("/track/{}/mute", tnum);
                            messages.push(OscMessage::new(reaper_path, vec![OscArg::Float(if muted { 1.0 } else { 0.0 })]));
                        }
                    },
                    _ => {
                        if path_parts.len() > 5 && path_parts[5] == "level" {
                            let send_num: u8 = path_parts[4].parse().ok().unwrap_or(0);
                            if let Some(OscArg::Float(level)) = msg.args.get(0) {
                                let reaper_path = format!("/track/{}/send/{}/volume", tnum, send_num);
                                messages.push(OscMessage::new(reaper_path, vec![OscArg::Float(*level)]));
                            }
                        }
                    }
                }
            }
        } else if path_parts[3] == "eq" {
            if path_parts.len() < 6 {
                return Ok(messages);
            }
            if let Some(tnum) = get_track_num_from_x32_path(prefix, num, config, app_state) {
                let band: u8 = path_parts[4].parse().ok().unwrap_or(0);
                let param = path_parts[5];
                if let Some(OscArg::Float(level)) = msg.args.get(0) {
                    let param_num = match (band, param) {
                        (1, "f") => 1, (1, "g") => 2, (1, "q") => 3,
                        (2, "f") => 4, (2, "g") => 5, (2, "q") => 6,
                        (3, "f") => 7, (3, "g") => 8, (3, "q") => 9,
                        (4, "f") => 10, (4, "g") => 11, (4, "q") => 12,
                        _ => 0,
                    };
                    if param_num > 0 {
                        let reaper_path = format!("/track/{}/fx/fxparam/{}/value", tnum, param_num);
                        messages.push(OscMessage::new(reaper_path, vec![OscArg::Float(*level)]));
                    }
                }
            }
        }
    } else if prefix == "dca" {
        let num: u8 = path_parts[2].parse().ok().unwrap_or(0);

        if path_parts.len() < 4 {
            return Ok(messages);
        }

        if path_parts[3] == "fader" {
            if let Some(OscArg::Float(level)) = msg.args.get(0) {
                if let Some(tnum) = get_track_num_from_x32_path(prefix, num, config, app_state) {
                    let reaper_path = format!("/track/{}/volume", tnum);
                    messages.push(OscMessage::new(reaper_path, vec![OscArg::Float(*level)]));
                }

                let dca_index = (num - 1) as usize;
                if dca_index < config.reaper_dca_map.len() {
                    let dca_map = &config.reaper_dca_map[dca_index];
                    for i in dca_map.min..=dca_map.max {
                        let reaper_path = format!("/track/{}/volume", i);
                        messages.push(OscMessage::new(reaper_path, vec![OscArg::Float(*level)]));
                    }
                }
            }
        }
    } else if prefix == "-stat" {
        if path_parts.len() < 3 {
            return Ok(messages);
        }
        if path_parts[2] == "selidx" {
            if let Some(OscArg::Int(idx)) = msg.args.get(0) {
                app_state.x32_selected_channel = *idx as u8;
            }
        } else if path_parts[2] == "solosw" {
            if path_parts.len() < 4 {
                return Ok(messages);
            }
            let num: u8 = path_parts[3].parse().ok().unwrap_or(0);
            if let Some(OscArg::Int(val)) = msg.args.get(0) {
                if let Some(tnum) = get_track_num_from_x32_path("ch", num, config, app_state) {
                    let soloed = *val == 1;
                    let reaper_path = format!("/track/{}/solo", tnum);
                    messages.push(OscMessage::new(reaper_path, vec![OscArg::Float(if soloed { 1.0 } else { 0.0 })]));
                }
            }
        } else if path_parts[2] == "userpar" {
            if path_parts.len() < 4 {
                return Ok(messages);
            }
            let num: u8 = path_parts[3].parse().ok().unwrap_or(0);
            if let Some(OscArg::Int(val)) = msg.args.get(0) {
                if *val == 0 { // Button up
                    if config.transport_on {
                        match num {
                            17 => messages.push(OscMessage::new("/action/40042".to_string(), vec![])), // REW
                            18 => messages.push(OscMessage::new("/play".to_string(), vec![])), // PLAY
                            19 => messages.push(OscMessage::new("/pause".to_string(), vec![])), // PAUSE
                            20 => messages.push(OscMessage::new("/action/40043".to_string(), vec![])), // FF
                            23 => messages.push(OscMessage::new("/stop".to_string(), vec![])), // STOP
                            24 => messages.push(OscMessage::new("/record".to_string(), vec![])), // REC
                            _ => {}
                        }
                    }
                    if config.channel_bank_on {
                        let num_tracks = (config.channel_map.max - config.channel_map.min + 1) as u8;
                        let max_bank_offset = (num_tracks - 1) / config.bank_size;

                        if num == config.bank_up_button + 12 {
                            if app_state.channel_bank_offset < max_bank_offset {
                                app_state.channel_bank_offset += 1;
                                messages.extend(update_x32_channel_bank(config, app_state));
                            }
                        } else if num == config.bank_down_button + 12 {
                            if app_state.channel_bank_offset > 0 {
                                app_state.channel_bank_offset -= 1;
                                messages.extend(update_x32_channel_bank(config, app_state));
                            }
                        }
                    }
                    if config.marker_button_on {
                        if num == config.marker_button + 12 {
                            messages.push(OscMessage::new("/action/40157".to_string(), vec![])); // Set marker
                        }
                    }
                } else if *val > 64 { // Encoder right
                    match num {
                        33 => messages.push(OscMessage::new("/action/40841".to_string(), vec![])), // Next Beat
                        34 => messages.push(OscMessage::new("/action/40839".to_string(), vec![])), // Next Measure
                        35 => messages.push(OscMessage::new("/action/40173".to_string(), vec![])), // Next Marker
                        36 => messages.push(OscMessage::new("/action/40319".to_string(), vec![])), // Next Item
                        _ => {}
                    }
                } else if *val < 64 { // Encoder left
                    match num {
                        33 => messages.push(OscMessage::new("/action/40842".to_string(), vec![])), // Previous Beat
                        34 => messages.push(OscMessage::new("/action/40840".to_string(), vec![])), // Previous Measure
                        35 => messages.push(OscMessage::new("/action/40172".to_string(), vec![])), // Previous Marker
                        36 => messages.push(OscMessage::new("/action/40318".to_string(), vec![])), // Previous Item
                        _ => {}
                    }
                }
            }
        }
    } else {
        println!("Unhandled x32 message: {}", msg.path);
    }

    Ok(messages)
}

/// Gets the Reaper track number for a given X32 OSC path.
fn get_track_num_from_x32_path(prefix: &str, num: u8, config: &Config, app_state: &AppState) -> Option<u8> {
    match prefix {
        "ch" => {
            let ch_num = num;
            if ch_num <= config.bank_size {
                return Some(ch_num + (app_state.channel_bank_offset * config.bank_size) + config.channel_map.min - 1);
            }
        },
        "auxin" => {
            let aux_num = num;
            return Some(aux_num + config.aux_map.min - 1);
        },
        "fxrtn" => {
            let fxr_num = num;
            return Some(fxr_num + config.fx_return_map.min - 1);
        },
        "bus" => {
            let bus_num = num;
            return Some(bus_num + config.bus_map.min - 1);
        },
        "dca" => {
            let dca_num = num;
            return Some(dca_num + config.dca_map.min - 1);
        }
        _ => {}
    }

    None
}

fn update_x32_channel_bank(config: &Config, app_state: &AppState) -> Vec<OscMessage> {
    let mut messages = Vec::new();
    let bank_offset = app_state.channel_bank_offset as usize;
    let bank_size = config.bank_size as usize;

    for i in 0..bank_size {
        let track_index = bank_offset * bank_size + i;
        if track_index < app_state.track_states.len() {
            let track_state = &app_state.track_states[track_index];
            let ch_num = i + 1;

            // Fader
            let path = format!("/ch/{:02}/mix/fader", ch_num);
            let x32_level = (track_state.fader * 1023.5).round() / 1023.0;
            messages.push(OscMessage::new(path, vec![OscArg::Float(x32_level)]));

            // Pan
            let path = format!("/ch/{:02}/mix/pan", ch_num);
            messages.push(OscMessage::new(path, vec![OscArg::Float(track_state.pan)]));

            // Mute
            let path = format!("/ch/{:02}/mix/on", ch_num);
            messages.push(OscMessage::new(path, vec![OscArg::Int(if track_state.mute { 0 } else { 1 })]));

            // Scribble
            let path = format!("/ch/{:02}/config/name", ch_num);
            messages.push(OscMessage::new(path, vec![OscArg::String(track_state.scribble.clone())]));

            // Color
            let path = format!("/ch/{:02}/config/color", ch_num);
            messages.push(OscMessage::new(path, vec![OscArg::Int(track_state.color as i32)]));

            // Icon
            let path = format!("/ch/{:02}/config/icon", ch_num);
            messages.push(OscMessage::new(path, vec![OscArg::Int(track_state.icon as i32)]));
        }
    }
    messages
}

#[cfg(test)]
mod tests;
