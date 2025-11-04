
use anyhow::{Result, anyhow};
use std::net::UdpSocket;
use crate::{Config, Track};
use osc_lib::{OscMessage, OscArg};

pub fn handle_reaper_message(msg: OscMessage, socket: &UdpSocket, config: &Config, tracks: &mut [Track], bank_offset: usize) -> Result<()> {
    let mut path_parts = msg.path.split('/');
    path_parts.next(); // Skip the first empty part

    if let Some("track") = path_parts.next() {
        if let Some(track_num_str) = path_parts.next() {
            if let Ok(track_num) = track_num_str.parse::<usize>() {
                if track_num > 0 && track_num <= config.track_max as usize {
                    let track_index = track_num - 1;
                    match path_parts.next() {
                        Some("volume") => {
                            if let Some(OscArg::Float(val)) = msg.args.get(0) {
                                tracks[track_index].fader = *val;
                                let x32_track = track_num - bank_offset;
                                if x32_track > 0 && x32_track <= config.bank_size {
                                    let path = format!("/ch/{:02}/mix/fader", x32_track);
                                    let reply = OscMessage::new(path, vec![OscArg::Float(*val)]);
                                    socket.send(&reply.to_bytes().map_err(|e: String| anyhow!(e))?)?;
                                }
                            }
                        }
                        Some("pan") => {
                            if let Some(OscArg::Float(val)) = msg.args.get(0) {
                                tracks[track_index].pan = *val;
                                let x32_track = track_num - bank_offset;
                                if x32_track > 0 && x32_track <= config.bank_size {
                                    let path = format!("/ch/{:02}/mix/pan", x32_track);
                                    let reply = OscMessage::new(path, vec![OscArg::Float(*val)]);
                                    socket.send(&reply.to_bytes().map_err(|e: String| anyhow!(e))?)?;
                                }
                            }
                        }
                        // Add more handlers for other Reaper messages here
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
