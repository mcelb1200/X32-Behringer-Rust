use crate::state::SharedState;
use anyhow::Result;
use osc_lib::{OscArg, OscMessage};
use std::sync::Arc;
use tokio::net::UdpSocket;

lazy_static::lazy_static! {
    static ref VOLUME_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/volume", i));
    static ref PAN_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/pan", i));
    static ref MUTE_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/mute", i));
    static ref SOLO_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/solo", i));
    static ref NAME_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/name", i));
    static ref SELECT_PATHS: [String; 257] = core::array::from_fn(|i| format!("/track/{}/select", i));
}

fn get_volume_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if (0..=256).contains(&track_id) {
        std::borrow::Cow::Borrowed(&VOLUME_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/volume", track_id))
    }
}

fn get_pan_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if (0..=256).contains(&track_id) {
        std::borrow::Cow::Borrowed(&PAN_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/pan", track_id))
    }
}

fn get_mute_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if (0..=256).contains(&track_id) {
        std::borrow::Cow::Borrowed(&MUTE_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/mute", track_id))
    }
}

fn get_solo_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if (0..=256).contains(&track_id) {
        std::borrow::Cow::Borrowed(&SOLO_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/solo", track_id))
    }
}

fn get_name_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if (0..=256).contains(&track_id) {
        std::borrow::Cow::Borrowed(&NAME_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/name", track_id))
    }
}

fn get_select_path(track_id: i32) -> std::borrow::Cow<'static, str> {
    if (0..=256).contains(&track_id) {
        std::borrow::Cow::Borrowed(&SELECT_PATHS[track_id as usize])
    } else {
        std::borrow::Cow::Owned(format!("/track/{}/select", track_id))
    }
}

async fn send_osc(socket: &Arc<UdpSocket>, addr: &str, path: &str, arg: OscArg) -> Result<()> {
    let bytes = OscMessage::serialize_to_bytes(path, std::iter::once(&arg))?;
    socket.send_to(&bytes, addr).await?;
    Ok(())
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
    // Master bus mappings (X32ReaperW.c)
    if msg.path == "/main/st/mix/fader" {
        if let Some(OscArg::Float(val)) = msg.args.first() {
            send_osc(socket, addr, "/master/volume", OscArg::Float(*val)).await?;
        }
        return Ok(());
    }
    if msg.path == "/main/st/mix/pan" {
        if let Some(OscArg::Float(val)) = msg.args.first() {
            send_osc(socket, addr, "/master/pan", OscArg::Float(*val)).await?;
        }
        return Ok(());
    }

    // Solo switch: /-stat/solosw/<01..80> -> /track/<id>/solo
    if let Some(rest) = msg.path.strip_prefix("/-stat/solosw/") {
        if let (Ok(sw_idx), Some(OscArg::Int(val))) = (rest.parse::<i32>(), msg.args.first()) {
            if let Some(track_id) = map_solosw_to_track(sw_idx, state).await {
                let path = get_solo_path(track_id);
                let solo_val = if *val != 0 { 1.0 } else { 0.0 };
                send_osc(socket, addr, &path, OscArg::Float(solo_val)).await?;
            }
        }
        return Ok(());
    }

    // Selected channel index: /-stat/selidx -> /track/<id>/select
    if msg.path == "/-stat/selidx" {
        if let Some(OscArg::Int(sel_idx)) = msg.args.first() {
            // X32 selidx is 0-based (0..31 = ch 1..32, 32..39 = aux 1..8, 40..47 = fxrtn 1..8, 48..63 = bus 1..16)
            if let Some(track_id) = map_selidx_to_track(*sel_idx, state).await {
                let path = get_select_path(track_id);
                send_osc(socket, addr, &path, OscArg::Float(1.0)).await?;
            }
        }
        return Ok(());
    }

    let mut it = msg.path.split('/');
    let _ = it.next(); // leading empty segment
    let section = it.next().unwrap_or("");
    let idx_str = it.next().unwrap_or("");
    let idx: i32 = match idx_str.parse() {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };

    let track_id = match section {
        "ch" => map_ch_to_track(idx, state).await,
        "auxin" => map_range_to_track(idx, 1, 8, state, |m| (m.aux_min, m.aux_max)).await,
        "fxrtn" => map_range_to_track(idx, 1, 8, state, |m| (m.fxr_min, m.fxr_max)).await,
        "bus" => map_range_to_track(idx, 1, 16, state, |m| (m.bus_min, m.bus_max)).await,
        "dca" => map_range_to_track(idx, 1, 8, state, |m| (m.dca_min, m.dca_max)).await,
        _ => None,
    };

    let Some(track_id) = track_id else {
        return Ok(());
    };

    let cmd = it.next().unwrap_or("");
    let sub = it.next().unwrap_or("");
    let sub2 = it.next().unwrap_or("");

    // DCA uses /dca/<n>/fader and /dca/<n>/on directly
    if section == "dca" {
        match cmd {
            "fader" => {
                if let Some(OscArg::Float(val)) = msg.args.first() {
                    let path = get_volume_path(track_id);
                    send_osc(socket, addr, &path, OscArg::Float(*val)).await?;
                }
            }
            "on" => {
                if let Some(OscArg::Int(val)) = msg.args.first() {
                    let mute_val = if *val == 1 { 0.0 } else { 1.0 };
                    let path = get_mute_path(track_id);
                    send_osc(socket, addr, &path, OscArg::Float(mute_val)).await?;
                }
            }
            "config" if sub == "name" => {
                if let Some(OscArg::String(name)) = msg.args.first() {
                    let path = get_name_path(track_id);
                    send_osc(socket, addr, &path, OscArg::String(name.clone())).await?;
                }
            }
            _ => {}
        }
        return Ok(());
    }

    match cmd {
        "mix" => match sub {
            "fader" => {
                if let Some(OscArg::Float(val)) = msg.args.first() {
                    let path = get_volume_path(track_id);
                    send_osc(socket, addr, &path, OscArg::Float(*val)).await?;
                }
            }
            "pan" => {
                if let Some(OscArg::Float(val)) = msg.args.first() {
                    let path = get_pan_path(track_id);
                    send_osc(socket, addr, &path, OscArg::Float(*val)).await?;
                }
            }
            "on" => {
                if let Some(OscArg::Int(val)) = msg.args.first() {
                    // X32: 1=on, 0=off/mute. Reaper: 1.0=mute, 0.0=unmute
                    let mute_val = if *val == 1 { 0.0 } else { 1.0 };
                    let path = get_mute_path(track_id);
                    send_osc(socket, addr, &path, OscArg::Float(mute_val)).await?;
                }
            }
            send_num_str if sub2 == "level" => {
                // Track sends: /{ch,auxin,fxrtn}/XX/mix/YY/level -> /track/N/send/S/volume
                if let (Ok(send_num), Some(OscArg::Float(val))) =
                    (send_num_str.parse::<i32>(), msg.args.first())
                {
                    let offset = {
                        let s = state.lock().unwrap_or_else(|e| e.into_inner());
                        s.config.map.track_send_offset
                    };
                    let reaper_send = send_num - offset;
                    if reaper_send >= 1 {
                        let path = format!("/track/{}/send/{}/volume", track_id, reaper_send);
                        send_osc(socket, addr, &path, OscArg::Float(*val)).await?;
                    }
                }
            }
            _ => {}
        },
        "config" if sub == "name" => {
            if let Some(OscArg::String(name)) = msg.args.first() {
                let path = get_name_path(track_id);
                send_osc(socket, addr, &path, OscArg::String(name.clone())).await?;
            }
        }
        "eq" if sub == "on" => {
            // X32 eq/on (1=active, 0=bypass) <-> Reaper fx/1/bypass (0.0=active, 1.0=bypass)
            if let Some(OscArg::Int(val)) = msg.args.first() {
                let bypass = if *val != 0 { 0.0 } else { 1.0 };
                let path = format!("/track/{}/fx/1/bypass", track_id);
                send_osc(socket, addr, &path, OscArg::Float(bypass)).await?;
            }
        }
        "dyn" if sub == "on" => {
            // X32 dyn/on (1=active, 0=bypass) <-> Reaper fx/2/bypass (0.0=active, 1.0=bypass)
            if let Some(OscArg::Int(val)) = msg.args.first() {
                let bypass = if *val != 0 { 0.0 } else { 1.0 };
                let path = format!("/track/{}/fx/2/bypass", track_id);
                send_osc(socket, addr, &path, OscArg::Float(bypass)).await?;
            }
        }
        _ => {}
    }

    Ok(())
}

async fn map_ch_to_track(ch: i32, state: &SharedState) -> Option<i32> {
    let s = state.lock().unwrap_or_else(|e| e.into_inner());
    if !(1..=32).contains(&ch) {
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

async fn map_range_to_track<F>(
    idx: i32,
    min_idx: i32,
    max_idx: i32,
    state: &SharedState,
    range_fn: F,
) -> Option<i32>
where
    F: FnOnce(&crate::config::TrackMapConfig) -> (i32, i32),
{
    if idx < min_idx || idx > max_idx {
        return None;
    }
    let s = state.lock().unwrap_or_else(|e| e.into_inner());
    let (r_min, r_max) = range_fn(&s.config.map);
    if r_min <= 0 || r_max < r_min {
        return None;
    }
    let track_id = r_min + idx - 1;
    if track_id <= r_max {
        Some(track_id)
    } else {
        None
    }
}

async fn map_solosw_to_track(sw_idx: i32, state: &SharedState) -> Option<i32> {
    match sw_idx {
        1..=32 => map_ch_to_track(sw_idx, state).await,
        33..=40 => map_range_to_track(sw_idx - 32, 1, 8, state, |m| (m.aux_min, m.aux_max)).await,
        41..=48 => map_range_to_track(sw_idx - 40, 1, 8, state, |m| (m.fxr_min, m.fxr_max)).await,
        49..=64 => map_range_to_track(sw_idx - 48, 1, 16, state, |m| (m.bus_min, m.bus_max)).await,
        73..=80 => map_range_to_track(sw_idx - 72, 1, 8, state, |m| (m.dca_min, m.dca_max)).await,
        _ => None,
    }
}

async fn map_selidx_to_track(sel_idx: i32, state: &SharedState) -> Option<i32> {
    match sel_idx {
        0..=31 => map_ch_to_track(sel_idx + 1, state).await,
        32..=39 => map_range_to_track(sel_idx - 31, 1, 8, state, |m| (m.aux_min, m.aux_max)).await,
        40..=47 => map_range_to_track(sel_idx - 39, 1, 8, state, |m| (m.fxr_min, m.fxr_max)).await,
        48..=63 => map_range_to_track(sel_idx - 47, 1, 16, state, |m| (m.bus_min, m.bus_max)).await,
        _ => None,
    }
}
