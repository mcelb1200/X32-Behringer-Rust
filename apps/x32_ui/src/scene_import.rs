//! Runtime parser for Behringer X32 `.scn` scene files.
//!
//! Enables offline Virtual Soundcheck `.rpp` generation and routing inspection from
//! any saved X32 scene without requiring a live console connection.

use crate::mixer::MixerSnapshotState;
use anyhow::{Context, Result};
use std::io::Read as _;
use std::path::Path;
use x32_lib::model::db_to_fader;

const MAX_SCENE_FILE_BYTES: u64 = 4 * 1024 * 1024;

/// Loads and parses an X32 `.scn` scene file (bounded to 4 MiB) into [`MixerSnapshotState`].
pub fn parse_x32_scene_file(path: &Path) -> Result<MixerSnapshotState> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open scene file '{}'", path.display()))?;
    if file.metadata()?.len() > MAX_SCENE_FILE_BYTES {
        anyhow::bail!("Scene file exceeds {} byte safety limit", MAX_SCENE_FILE_BYTES);
    }
    let mut buf = String::new();
    file.take(MAX_SCENE_FILE_BYTES + 1).read_to_string(&mut buf)?;
    if buf.len() as u64 > MAX_SCENE_FILE_BYTES {
        anyhow::bail!("Scene file exceeds {} byte safety limit", MAX_SCENE_FILE_BYTES);
    }
    parse_x32_scene_str(&buf)
}

/// Parses raw X32 `.scn` text into a [`MixerSnapshotState`].
pub fn parse_x32_scene_str(content: &str) -> Result<MixerSnapshotState> {
    let mut state = MixerSnapshotState::default();
    state.model = "X32".to_string();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || !line.starts_with('/') {
            continue;
        }
        let tokens = tokenize_scn_line(line);
        if tokens.is_empty() {
            continue;
        }
        let path = tokens[0].as_str();
        let args = &tokens[1..];

        if path == "/config/chlink" {
            for (pair_idx, tok) in args.iter().enumerate() {
                let linked = tok.eq_ignore_ascii_case("ON") || tok == "1";
                let base = pair_idx * 2;
                if let Some(c) = state.channels.get_mut(base) {
                    c.linked_stereo = linked;
                }
                if let Some(c) = state.channels.get_mut(base + 1) {
                    c.linked_stereo = linked;
                }
            }
            continue;
        }
        if path == "/config/auxlink" {
            for (pair_idx, tok) in args.iter().enumerate() {
                let linked = tok.eq_ignore_ascii_case("ON") || tok == "1";
                let base = pair_idx * 2;
                if let Some(c) = state.auxins.get_mut(base) {
                    c.linked_stereo = linked;
                }
                if let Some(c) = state.auxins.get_mut(base + 1) {
                    c.linked_stereo = linked;
                }
            }
            continue;
        }
        if path == "/config/buslink" {
            for (pair_idx, tok) in args.iter().enumerate() {
                let linked = tok.eq_ignore_ascii_case("ON") || tok == "1";
                let base = pair_idx * 2;
                if let Some(c) = state.buses.get_mut(base) {
                    c.linked_stereo = linked;
                }
                if let Some(c) = state.buses.get_mut(base + 1) {
                    c.linked_stereo = linked;
                }
            }
            continue;
        }
        if path == "/config/mtxlink" {
            for (pair_idx, tok) in args.iter().enumerate() {
                let linked = tok.eq_ignore_ascii_case("ON") || tok == "1";
                let base = pair_idx * 2;
                if let Some(c) = state.matrices.get_mut(base) {
                    c.linked_stereo = linked;
                }
                if let Some(c) = state.matrices.get_mut(base + 1) {
                    c.linked_stereo = linked;
                }
            }
            continue;
        }

        if path == "/config/routing" {
            if let Some(v) = args.first() {
                state.routing.routswitch = i32::from(v.eq_ignore_ascii_case("PLAY") || v == "1");
            }
            continue;
        }
        if path == "/config/routing/IN" {
            for i in 0..4 {
                if let Some(tok) = args.get(i) {
                    state.routing.in_blocks[i] = parse_in_block(tok);
                }
            }
            if let Some(tok) = args.get(4) {
                state.routing.in_aux = parse_aux_block(tok);
            }
            continue;
        }
        if path == "/config/routing/PLAY" {
            for i in 0..4 {
                if let Some(tok) = args.get(i) {
                    state.routing.play_blocks[i] = parse_in_block(tok);
                }
            }
            if let Some(tok) = args.get(4) {
                state.routing.play_aux = parse_aux_block(tok);
            }
            continue;
        }
        if path == "/config/routing/CARD" {
            for i in 0..4 {
                if let Some(tok) = args.get(i) {
                    state.routing.card_blocks[i] = parse_card_block(tok);
                }
            }
            continue;
        }
        if path == "/config/routing/AES50A" {
            for i in 0..6 {
                if let Some(tok) = args.get(i) {
                    state.routing.aes50a_blocks[i] = parse_card_block(tok);
                }
            }
            continue;
        }
        if path == "/config/routing/AES50B" {
            for i in 0..6 {
                if let Some(tok) = args.get(i) {
                    state.routing.aes50b_blocks[i] = parse_card_block(tok);
                }
            }
            continue;
        }
        if path == "/config/routing/OUT" {
            for i in 0..4 {
                if let Some(tok) = args.get(i) {
                    state.routing.out_blocks[i] = parse_out_block(tok);
                }
            }
            continue;
        }

        if let Some(rest) = path.strip_prefix("/ch/") {
            if let Some((num_s, sub)) = rest.split_once('/') {
                if let Ok(idx) = num_s.parse::<usize>() {
                    if let Some(ch) = state.channels.get_mut(idx.saturating_sub(1)) {
                        match sub {
                            "config" => {
                                if let Some(name) = args.first() {
                                    ch.name = name.clone();
                                }
                                if let Some(icon) = args.get(1).and_then(|s| s.parse::<i32>().ok()) {
                                    ch.icon = icon;
                                }
                                if let Some(col) = args.get(2) {
                                    ch.color = parse_color_name(col);
                                }
                                if let Some(src) = args.get(3).and_then(|s| s.parse::<i32>().ok()) {
                                    ch.source = src;
                                }
                            }
                            "mix" => {
                                if let Some(on_tok) = args.first() {
                                    ch.on = on_tok.eq_ignore_ascii_case("ON") || on_tok == "1";
                                }
                                if let Some(db_tok) = args.get(1) {
                                    ch.fader = parse_scn_db_to_fader(db_tok);
                                }
                                if let Some(pan_tok) = args.get(3) {
                                    ch.pan = parse_scn_pan(pan_tok);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            continue;
        }

        if let Some(rest) = path.strip_prefix("/auxin/") {
            if let Some((num_s, sub)) = rest.split_once('/') {
                if let Ok(idx) = num_s.parse::<usize>() {
                    if let Some(aux) = state.auxins.get_mut(idx.saturating_sub(1)) {
                        match sub {
                            "config" => {
                                if let Some(name) = args.first() {
                                    aux.name = name.clone();
                                }
                                if let Some(col) = args.get(2) {
                                    aux.color = parse_color_name(col);
                                }
                            }
                            "mix" => {
                                if let Some(on_tok) = args.first() {
                                    aux.on = on_tok.eq_ignore_ascii_case("ON") || on_tok == "1";
                                }
                                if let Some(db_tok) = args.get(1) {
                                    aux.fader = parse_scn_db_to_fader(db_tok);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            continue;
        }

        if let Some(rest) = path.strip_prefix("/bus/") {
            if let Some((num_s, sub)) = rest.split_once('/') {
                if let Ok(idx) = num_s.parse::<usize>() {
                    if let Some(bus) = state.buses.get_mut(idx.saturating_sub(1)) {
                        match sub {
                            "config" => {
                                if let Some(name) = args.first() {
                                    bus.name = name.clone();
                                }
                                if let Some(col) = args.get(2) {
                                    bus.color = parse_color_name(col);
                                }
                            }
                            "mix" => {
                                if let Some(on_tok) = args.first() {
                                    bus.on = on_tok.eq_ignore_ascii_case("ON") || on_tok == "1";
                                }
                                if let Some(db_tok) = args.get(1) {
                                    bus.fader = parse_scn_db_to_fader(db_tok);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            continue;
        }

        if let Some(rest) = path.strip_prefix("/mtx/") {
            if let Some((num_s, sub)) = rest.split_once('/') {
                if let Ok(idx) = num_s.parse::<usize>() {
                    if let Some(mtx) = state.matrices.get_mut(idx.saturating_sub(1)) {
                        match sub {
                            "config" => {
                                if let Some(name) = args.first() {
                                    mtx.name = name.clone();
                                }
                                if let Some(col) = args.get(2) {
                                    mtx.color = parse_color_name(col);
                                }
                            }
                            "mix" => {
                                if let Some(on_tok) = args.first() {
                                    mtx.on = on_tok.eq_ignore_ascii_case("ON") || on_tok == "1";
                                }
                                if let Some(db_tok) = args.get(1) {
                                    mtx.fader = parse_scn_db_to_fader(db_tok);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            continue;
        }

        if let Some(rest) = path.strip_prefix("/dca/") {
            if let Some((num_s, sub)) = rest.split_once('/') {
                if let Ok(idx) = num_s.parse::<usize>() {
                    if let Some(dca) = state.dcas.get_mut(idx.saturating_sub(1)) {
                        match sub {
                            "config" => {
                                if let Some(name) = args.first() {
                                    dca.name = name.clone();
                                }
                                if let Some(col) = args.get(2) {
                                    dca.color = parse_color_name(col);
                                }
                            }
                            "fader" => {
                                if let Some(db_tok) = args.first() {
                                    dca.fader = parse_scn_db_to_fader(db_tok);
                                }
                            }
                            "on" => {
                                if let Some(on_tok) = args.first() {
                                    dca.on = on_tok.eq_ignore_ascii_case("ON") || on_tok == "1";
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(state)
}

fn tokenize_scn_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            if in_quotes {
                tokens.push(std::mem::take(&mut cur));
                in_quotes = false;
            } else {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
                in_quotes = true;
            }
        } else if ch.is_whitespace() && !in_quotes {
            if !cur.is_empty() {
                tokens.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(ch);
        }
    }
    if in_quotes || !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

fn parse_color_name(tok: &str) -> i32 {
    if let Ok(n) = tok.parse::<i32>() {
        return n.clamp(0, 15);
    }
    let upper = tok.trim().to_ascii_uppercase();
    let inv = if upper.ends_with('I') && upper.len() >= 3 {
        8
    } else {
        0
    };
    let base_str = upper.trim_end_matches('I');
    let base = match base_str {
        "OFF" => 0,
        "RD" => 1,
        "GN" => 2,
        "YE" => 3,
        "BL" => 4,
        "MG" => 5,
        "CY" => 6,
        "WH" => 7,
        _ => 7,
    };
    base + inv
}

fn parse_in_block(tok: &str) -> i32 {
    match tok.trim().to_ascii_uppercase().as_str() {
        "AN1-8" => 0,
        "AN9-16" => 1,
        "AN17-24" => 2,
        "AN25-32" => 3,
        "A1-8" => 4,
        "A9-16" => 5,
        "A17-24" => 6,
        "A25-32" => 7,
        "A33-40" => 8,
        "A41-48" => 9,
        "B1-8" => 10,
        "B9-16" => 11,
        "B17-24" => 12,
        "B25-32" => 13,
        "B33-40" => 14,
        "B41-48" => 15,
        "CARD1-8" => 16,
        "CARD9-16" => 17,
        "CARD17-24" => 18,
        "CARD25-32" => 19,
        other => other.parse().unwrap_or(0),
    }
}

fn parse_aux_block(tok: &str) -> i32 {
    match tok.trim().to_ascii_uppercase().as_str() {
        "AUX1-4" => 0,
        "AN1-2" => 1,
        "AN1-4" => 2,
        "AN1-6" => 3,
        "A1-2" => 4,
        "A1-4" => 5,
        "A1-6" => 6,
        "B1-2" => 7,
        "B1-4" => 8,
        "B1-6" => 9,
        "CARD1-2" => 10,
        "CARD1-4" => 11,
        "CARD1-6" => 12,
        other => other.parse().unwrap_or(0),
    }
}

fn parse_card_block(tok: &str) -> i32 {
    match tok.trim().to_ascii_uppercase().as_str() {
        "OUT1-8" => 20,
        "OUT9-16" => 21,
        "P161-8" => 22,
        "P169-16" => 23,
        "AUX1-6/MON" => 24,
        "AUXIN1-6/TB" => 25,
        "UOUT1-8" => 26,
        "UOUT9-16" => 27,
        "UOUT17-24" => 28,
        "UOUT25-32" => 29,
        "UOUT33-40" => 30,
        "UOUT41-48" => 31,
        _ => parse_in_block(tok),
    }
}

fn parse_out_block(tok: &str) -> i32 {
    match tok.trim().to_ascii_uppercase().as_str() {
        "AN1-4" | "AN5-8" => 0,
        "AN9-12" | "AN13-16" => 1,
        "A1-4" | "A5-8" => 2,
        "B1-4" | "B5-8" => 3,
        "CARD1-4" | "CARD5-8" => 4,
        "OUT1-4" | "OUT5-8" => 5,
        "OUT9-12" | "OUT13-16" => 6,
        other => other.parse().unwrap_or(0),
    }
}

fn parse_scn_db_to_fader(tok: &str) -> f32 {
    let t = tok.trim();
    if t.eq_ignore_ascii_case("-oo") {
        return 0.0;
    }
    if let Ok(db) = t.parse::<f32>() {
        return db_to_fader(db);
    }
    0.0
}

fn parse_scn_pan(tok: &str) -> f32 {
    let t = tok.trim();
    if let Ok(p) = t.parse::<f32>() {
        return ((p + 100.0) / 200.0).clamp(0.0, 1.0);
    }
    0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_synthetic_scene_extracts_sources_links_and_uout() {
        let scn = "\
#4.0# \"synthetic\" \"\" %000000000 1
/config/chlink OFF ON OFF OFF OFF OFF OFF OFF OFF OFF OFF OFF OFF OFF OFF OFF
/config/auxlink ON ON OFF OFF
/config/routing REC
/config/routing/IN A1-8 A9-16 AN17-24 AN25-32 CARD1-2
/config/routing/AES50A OUT1-8 OUT9-16 CARD17-24 OUT9-16 P161-8 P169-16
/config/routing/CARD UOUT1-8 CARD9-16 CARD17-24 CARD25-32
/config/routing/PLAY AN1-8 AN1-8 AN1-8 AN1-8 AUX1-4
/ch/01/config \"Kick\" 1 RD 15
/ch/01/mix ON +0.0 OFF +0 OFF -oo
/ch/02/config \"Snare\" 2 RD 15
/ch/02/mix OFF -6.0 OFF -25 OFF -oo
/ch/03/config \"Synth L\" 3 BL 9
/ch/03/mix ON -3.0 OFF -100 OFF -oo
/ch/04/config \"Synth R\" 4 BL 10
/ch/04/mix ON -3.0 OFF +100 OFF -oo
/ch/05/config \"\" 1 WH 5
/auxin/01/config \"Media L\" 33 YE 33
/mtx/01/config \"Zone A\" 72 MG
";
        let st = parse_x32_scene_str(scn).unwrap();
        assert_eq!(st.channels[0].name, "Kick");
        assert_eq!(st.channels[0].source, 15);
        assert!(st.channels[0].on);
        assert!((st.channels[0].fader - 0.75).abs() < 1e-3);

        assert_eq!(st.channels[1].name, "Snare");
        assert_eq!(st.channels[1].source, 15);
        assert!(!st.channels[1].on);

        assert!(st.channels[2].linked_stereo);
        assert!(st.channels[3].linked_stereo);
        assert_eq!(st.channels[2].source, 9);
        assert_eq!(st.channels[3].source, 10);

        assert_eq!(st.channels[4].name, "");
        assert!(st.auxins[0].linked_stereo);
        assert_eq!(st.auxins[0].name, "Media L");
        assert_eq!(st.matrices[0].name, "Zone A");

        assert_eq!(st.routing.in_blocks, [4, 5, 2, 3]);
        assert_eq!(st.routing.in_aux, 10); // CARD1-2
        assert_eq!(st.routing.card_blocks[0], 26); // UOUT1-8
        assert_eq!(st.routing.aes50a_blocks[2], 18); // CARD17-24
        assert_eq!(st.routing.aes50a_blocks[4], 22); // P161-8
    }
}
