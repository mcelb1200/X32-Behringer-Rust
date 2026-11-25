use crate::state::SharedState;
use anyhow::Result;
use osc_lib::{OscArg, OscMessage};
use std::sync::Arc;
use tokio::net::UdpSocket;

pub async fn handle_reaper_message(
    buf: &[u8],
    len: usize,
    state: &SharedState,
    x32_socket: &Arc<UdpSocket>,
    x32_addr: &str,
) -> Result<()> {
    // Check for bundle
    if len >= 8 && &buf[0..8] == b"#bundle\0" {
        let mut idx = 16; // Skip #bundle\0 and 64-bit timetag
        while idx < len {
            if 4 > len.saturating_sub(idx) {
                break;
            }
            let size_val = i32::from_be_bytes([buf[idx], buf[idx + 1], buf[idx + 2], buf[idx + 3]]);
            if size_val < 0 {
                break;
            }
            let size = size_val as usize;
            idx += 4;

            if size > len.saturating_sub(idx) {
                break;
            }
            let element_buf = match buf.get(idx..idx + size) {
                Some(b) => b,
                None => break,
            };
            idx += size;

            if let Ok(msg) = OscMessage::from_bytes(element_buf) {
                process_single_message(msg, state, x32_socket, x32_addr).await?;
            }
        }
    } else if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
        process_single_message(msg, state, x32_socket, x32_addr).await?;
    }
    Ok(())
}

async fn send_x32(socket: &Arc<UdpSocket>, addr: &str, path: String, arg: OscArg) -> Result<()> {
    let x_msg = OscMessage::new(path, vec![arg]);
    socket.send_to(&x_msg.to_bytes()?, addr).await?;
    Ok(())
}

async fn process_single_message(
    msg: OscMessage,
    state: &SharedState,
    socket: &Arc<UdpSocket>,
    addr: &str,
) -> Result<()> {
    // Master bus mappings (X32ReaperW.c)
    if msg.path == "/master/volume" {
        if let Some(OscArg::Float(val)) = msg.args.first() {
            send_x32(
                socket,
                addr,
                "/main/st/mix/fader".to_string(),
                OscArg::Float(*val),
            )
            .await?;
        }
        return Ok(());
    }
    if msg.path == "/master/pan" {
        if let Some(OscArg::Float(val)) = msg.args.first() {
            send_x32(
                socket,
                addr,
                "/main/st/mix/pan".to_string(),
                OscArg::Float(*val),
            )
            .await?;
        }
        return Ok(());
    }

    if msg.path.starts_with("/track/") {
        let tnum_str = extract_nth_segment(&msg.path, 2).unwrap_or("");
        if tnum_str.is_empty() {
            return Ok(());
        }

        let tnum: i32 = tnum_str.parse().unwrap_or(-1);
        if tnum < 0 {
            return Ok(());
        }

        let cmd = extract_nth_segment(&msg.path, 3).unwrap_or("");

        match cmd {
            "volume" => {
                if let Some(OscArg::Float(val)) = msg.args.first() {
                    if let Some(addr_str) = map_track_to_x32(tnum, "mix/fader", "fader", state).await
                    {
                        send_x32(socket, addr, addr_str, OscArg::Float(*val)).await?;
                    }
                }
            }
            "pan" => {
                if let Some(OscArg::Float(val)) = msg.args.first() {
                    if let Some(addr_str) = map_track_to_x32(tnum, "mix/pan", "", state).await {
                        send_x32(socket, addr, addr_str, OscArg::Float(*val)).await?;
                    }
                }
            }
            "mute" => {
                if let Some(OscArg::Float(val)) = msg.args.first() {
                    if let Some(addr_str) = map_track_to_x32(tnum, "mix/on", "on", state).await {
                        // Reaper sends 1.0 for mute, X32 uses 0 for mute (on=0)
                        let on_val = if *val > 0.5 { 0 } else { 1 };
                        send_x32(socket, addr, addr_str, OscArg::Int(on_val)).await?;
                    }
                }
            }
            "name" => {
                if let Some(OscArg::String(name)) = msg.args.first() {
                    if let Some(addr_str) =
                        map_track_to_x32(tnum, "config/name", "config/name", state).await
                    {
                        send_x32(socket, addr, addr_str, OscArg::String(name.clone())).await?;
                    }
                }
            }
            "solo" => {
                if let Some(OscArg::Float(val)) = msg.args.first() {
                    if let Some(sw_idx) = map_track_to_solosw(tnum, state).await {
                        let solo_on = if *val > 0.5 { 1 } else { 0 };
                        send_x32(
                            socket,
                            addr,
                            format!("/-stat/solosw/{:02}", sw_idx),
                            OscArg::Int(solo_on),
                        )
                        .await?;
                    }
                }
            }
            "select" => {
                if let Some(OscArg::Float(val)) = msg.args.first() {
                    if *val > 0.5 {
                        if let Some(sel_idx) = map_track_to_selidx(tnum, state).await {
                            send_x32(
                                socket,
                                addr,
                                "/-stat/selidx".to_string(),
                                OscArg::Int(sel_idx),
                            )
                            .await?;
                        }
                    }
                }
            }
            "send" => {
                // /track/<tnum>/send/<snum>/volume -> /{ch,auxin,fxrtn}/XX/mix/YY/level
                let snum_str = extract_nth_segment(&msg.path, 4).unwrap_or("");
                let sub = extract_nth_segment(&msg.path, 5).unwrap_or("");
                if sub == "volume" {
                    if let (Ok(snum), Some(OscArg::Float(val))) =
                        (snum_str.parse::<i32>(), msg.args.first())
                    {
                        let offset = {
                            let s = state.lock().unwrap_or_else(|e| e.into_inner());
                            s.config.map.track_send_offset
                        };
                        let x32_bus = snum + offset;
                        if (1..=16).contains(&x32_bus) {
                            let suffix = format!("mix/{:02}/level", x32_bus);
                            if let Some(addr_str) = map_track_to_x32(tnum, &suffix, "", state).await
                            {
                                send_x32(socket, addr, addr_str, OscArg::Float(*val)).await?;
                            }
                        }
                    }
                }
            }
            "fx" => {
                // /track/<tnum>/fx/<fxnum>/bypass -> /{ch,auxin,fxrtn,bus}/XX/{eq,dyn}/on
                let fxnum = extract_nth_segment(&msg.path, 4).unwrap_or("");
                let sub = extract_nth_segment(&msg.path, 5).unwrap_or("");
                if sub == "bypass" {
                    if let Some(OscArg::Float(val)) = msg.args.first() {
                        let on_val = if *val > 0.5 { 0 } else { 1 };
                        let suffix = match fxnum {
                            "1" => Some("eq/on"),
                            "2" => Some("dyn/on"),
                            _ => None,
                        };
                        if let Some(sfx) = suffix {
                            if let Some(addr_str) = map_track_to_x32(tnum, sfx, "", state).await {
                                send_x32(socket, addr, addr_str, OscArg::Int(on_val)).await?;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn map_track_to_x32(
    tnum: i32,
    suffix: &str,
    dca_suffix: &str,
    state: &SharedState,
) -> Option<String> {
    let s = state.lock().unwrap_or_else(|e| e.into_inner());
    let map = &s.config.map;

    if map.track_min > 0 && tnum >= map.track_min && tnum <= map.track_max {
        if s.config.flags.channel_bank_on {
            let offset = s.bank_offset * s.config.bank.bank_size;
            let raw_t = tnum - map.track_min;
            if raw_t >= offset && raw_t < offset + s.config.bank.bank_size {
                let t = raw_t - offset + 1;
                return Some(format!("/ch/{:02}/{}", t, suffix));
            }
        } else {
            let t = tnum - map.track_min + 1;
            return Some(format!("/ch/{:02}/{}", t, suffix));
        }
    }
    if map.aux_min > 0 && tnum >= map.aux_min && tnum <= map.aux_max {
        let t = tnum - map.aux_min + 1;
        return Some(format!("/auxin/{:02}/{}", t, suffix));
    }
    if map.fxr_min > 0 && tnum >= map.fxr_min && tnum <= map.fxr_max {
        let t = tnum - map.fxr_min + 1;
        return Some(format!("/fxrtn/{:02}/{}", t, suffix));
    }
    if map.bus_min > 0 && tnum >= map.bus_min && tnum <= map.bus_max {
        let t = tnum - map.bus_min + 1;
        return Some(format!("/bus/{:02}/{}", t, suffix));
    }
    if !dca_suffix.is_empty() && map.dca_min > 0 && tnum >= map.dca_min && tnum <= map.dca_max {
        let t = tnum - map.dca_min + 1;
        return Some(format!("/dca/{}/{}", t, dca_suffix));
    }

    None
}

async fn map_track_to_solosw(tnum: i32, state: &SharedState) -> Option<i32> {
    let s = state.lock().unwrap_or_else(|e| e.into_inner());
    let map = &s.config.map;
    if map.track_min > 0 && tnum >= map.track_min && tnum <= map.track_max {
        return Some(tnum - map.track_min + 1);
    }
    if map.aux_min > 0 && tnum >= map.aux_min && tnum <= map.aux_max {
        return Some(32 + (tnum - map.aux_min + 1));
    }
    if map.fxr_min > 0 && tnum >= map.fxr_min && tnum <= map.fxr_max {
        return Some(40 + (tnum - map.fxr_min + 1));
    }
    if map.bus_min > 0 && tnum >= map.bus_min && tnum <= map.bus_max {
        return Some(48 + (tnum - map.bus_min + 1));
    }
    if map.dca_min > 0 && tnum >= map.dca_min && tnum <= map.dca_max {
        return Some(72 + (tnum - map.dca_min + 1));
    }
    None
}

async fn map_track_to_selidx(tnum: i32, state: &SharedState) -> Option<i32> {
    let s = state.lock().unwrap_or_else(|e| e.into_inner());
    let map = &s.config.map;
    if map.track_min > 0 && tnum >= map.track_min && tnum <= map.track_max {
        return Some(tnum - map.track_min);
    }
    if map.aux_min > 0 && tnum >= map.aux_min && tnum <= map.aux_max {
        return Some(32 + (tnum - map.aux_min));
    }
    if map.fxr_min > 0 && tnum >= map.fxr_min && tnum <= map.fxr_max {
        return Some(40 + (tnum - map.fxr_min));
    }
    if map.bus_min > 0 && tnum >= map.bus_min && tnum <= map.bus_max {
        return Some(48 + (tnum - map.bus_min));
    }
    None
}

#[inline(always)]
fn extract_nth_segment(path: &str, n: usize) -> Option<&str> {
    let iter = path.as_bytes().iter().enumerate();
    let mut slashes = 0;
    let mut start_idx = 0;

    for (i, &b) in iter {
        if b == b'/' {
            if slashes == n {
                return Some(&path[start_idx..i]);
            }
            slashes += 1;
            if slashes == n {
                start_idx = i + 1;
            }
        }
    }

    if slashes == n && start_idx < path.len() {
        return Some(&path[start_idx..]);
    }
    None
}
