use crate::state::SharedState;
use anyhow::Result;
use osc_lib::{OscArg, OscMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub async fn handle_x32_message(
    buf: &[u8],
    len: usize,
    state: &SharedState,
    reaper_socket: &Arc<UdpSocket>,
    reaper_addr: &str,
) -> Result<()> {
    if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
        process_x32_message(msg, state, reaper_socket, reaper_addr).await?;
    }
    Ok(())
}

async fn process_x32_message(
    msg: OscMessage,
    state: &SharedState,
    socket: &Arc<UdpSocket>,
    addr: &str,
) -> Result<()> {
    // Example: /ch/01/mix/fader -> /track/1/volume

    // Simplification for prototype
    if msg.path.starts_with("/ch/") {
        let parts: Vec<&str> = msg.path.split('/').collect();
        if parts.len() < 4 {
            return Ok(());
        }

        let ch_num: i32 = parts[2].parse().unwrap_or(0);
        let cmd = parts[3]; // "mix", "config"

        if cmd == "mix" {
            let sub = parts.get(4).unwrap_or(&"");
            match *sub {
                "fader" => {
                    if let Some(OscArg::Float(val)) = msg.args.first() {
                        if let Some(track_id) = map_ch_to_track(ch_num, state).await {
                            let r_msg = OscMessage::new(
                                format!("/track/{}/volume", track_id),
                                vec![OscArg::Float(*val)],
                            );
                            socket.send_to(&r_msg.to_bytes()?, addr).await?;
                        }
                    }
                }
                "pan" => {
                    if let Some(OscArg::Float(val)) = msg.args.first() {
                        if let Some(track_id) = map_ch_to_track(ch_num, state).await {
                            let r_msg = OscMessage::new(
                                format!("/track/{}/pan", track_id),
                                vec![OscArg::Float(*val)],
                            );
                            socket.send_to(&r_msg.to_bytes()?, addr).await?;
                        }
                    }
                }
                "on" => {
                    if let Some(OscArg::Int(val)) = msg.args.first() {
                        if let Some(track_id) = map_ch_to_track(ch_num, state).await {
                            // X32: 1=on, 0=off/mute. Reaper: 1.0=mute, 0.0=unmute
                            let mute_val = if *val == 1 { 0.0 } else { 1.0 };
                            let r_msg = OscMessage::new(
                                format!("/track/{}/mute", track_id),
                                vec![OscArg::Float(mute_val)],
                            );
                            socket.send_to(&r_msg.to_bytes()?, addr).await?;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

async fn map_ch_to_track(ch: i32, state: &SharedState) -> Option<i32> {
    let s = state.lock().unwrap();
    // Logic: cnum = cnum * 10 + ...
    // cnum1 = cnum + Xtrk_min - 1
    // if Xchbank_on, Set actual channel number to match Channel Bank
    // cnum = Xchbkof * bkchsz + cnum

    if ch < 1 || ch > 32 {
        return None;
    }

    let mut track_id = ch;
    if s.config.flags.channel_bank_on {
        track_id = s.bank_offset * s.config.bank.bank_size + ch;
    }

    track_id = track_id + s.config.map.track_min - 1;

    if s.config.map.track_max > 0 && track_id <= s.config.map.track_max {
        return Some(track_id);
    }

    None
}
