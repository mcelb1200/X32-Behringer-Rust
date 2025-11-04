
use anyhow::{Result, anyhow};
use std::net::UdpSocket;
use crate::{Config, Track};
use osc_lib::{OscMessage, OscArg};

pub fn handle_x32_message(msg: OscMessage, socket: &UdpSocket, config: &Config, tracks: &mut [Track], bank_offset: usize) -> Result<()> {
    let mut path_parts = msg.path.split('/');
    path_parts.next(); // Skip the first empty part

    if let Some("ch") = path_parts.next() {
        if let Some(ch_num_str) = path_parts.next() {
            if let Ok(ch_num) = ch_num_str.parse::<usize>() {
                if ch_num > 0 && ch_num <= config.bank_size {
                    let track_num = ch_num + bank_offset;
                    if track_num > 0 && track_num <= config.track_max as usize {
                        let track_index = track_num - 1;
                        if let Some("mix") = path_parts.next() {
                             match path_parts.next() {
                                Some("fader") => {
                                    if let Some(OscArg::Float(val)) = msg.args.get(0) {
                                        tracks[track_index].fader = *val;
                                        let path = format!("/track/{}/volume", track_num);
                                        let reply = OscMessage::new(path, vec![OscArg::Float(*val)]);
                                        socket.send(&reply.to_bytes().map_err(|e: String| anyhow!(e))?)?;
                                    }
                                }
                                Some("pan") => {
                                    if let Some(OscArg::Float(val)) = msg.args.get(0) {
                                        tracks[track_index].pan = *val;
                                        let path = format!("/track/{}/pan", track_num);
                                        let reply = OscMessage::new(path, vec![OscArg::Float(*val)]);
                                        socket.send(&reply.to_bytes().map_err(|e: String| anyhow!(e))?)?;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
