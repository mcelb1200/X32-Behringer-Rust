use crate::state::SharedState;
use anyhow::Result;
use osc_lib::{OscArg, OscMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

lazy_static::lazy_static! {
    static ref VOLUME_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/volume", i));
    static ref PAN_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/pan", i));
    static ref MUTE_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/mute", i));
}

fn get_volume_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if track_id >= 0 && track_id <= 256 {
        std::borrow::Cow::Borrowed(&VOLUME_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/volume", track_id))
    }
}

fn get_pan_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if track_id >= 0 && track_id <= 256 {
        std::borrow::Cow::Borrowed(&PAN_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/pan", track_id))
    }
}

fn get_mute_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if track_id >= 0 && track_id <= 256 {
        std::borrow::Cow::Borrowed(&MUTE_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/mute", track_id))
    }
}

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
        let mut it = msg.path.split('/');
        // Format: /ch/<num>/<cmd>/<sub>
        let _ = it.next(); // empty before first /
        let _ = it.next(); // "ch"
        let ch_num_str = it.next().unwrap_or("");
        if ch_num_str.is_empty() {
            return Ok(());
        }

        let ch_num: i32 = ch_num_str.parse().unwrap_or(0);
        let cmd = it.next().unwrap_or("");

        if cmd == "mix" {
            let sub = it.next().unwrap_or("");
            match *sub {
                "fader" => {
                    if let Some(OscArg::Float(val)) = msg.args.first() {
                        if let Some(track_id) = map_ch_to_track(ch_num, state).await {
                            let path = get_volume_path(track_id);
                            let bytes = OscMessage::serialize_to_bytes(
                                &path,
                                std::iter::once(&OscArg::Float(*val)),
                            )?;
                            socket.send_to(&bytes, addr).await?;
                        }
                    }
                }
                "pan" => {
                    if let Some(OscArg::Float(val)) = msg.args.first() {
                        if let Some(track_id) = map_ch_to_track(ch_num, state).await {
                            let path = get_pan_path(track_id);
                            let bytes = OscMessage::serialize_to_bytes(
                                &path,
                                std::iter::once(&OscArg::Float(*val)),
                            )?;
                            socket.send_to(&bytes, addr).await?;
                        }
                    }
                }
                "on" => {
                    if let Some(OscArg::Int(val)) = msg.args.first() {
                        if let Some(track_id) = map_ch_to_track(ch_num, state).await {
                            // X32: 1=on, 0=off/mute. Reaper: 1.0=mute, 0.0=unmute
                            let mute_val = if *val == 1 { 0.0 } else { 1.0 };
                            let path = get_mute_path(track_id);
                            let bytes = OscMessage::serialize_to_bytes(
                                &path,
                                std::iter::once(&OscArg::Float(mute_val)),
                            )?;
                            socket.send_to(&bytes, addr).await?;
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
    let s = state.lock().unwrap_or_else(|e| e.into_inner());
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
