//! Reaper Virtual Soundcheck (VSC) automation:
//! - X32 routing plan computation (`CARD` Out + `PLAY` Card return) with pre-VSC backup snapshots
//! - One-click `REC (Live) <-> PLAY (VSC)` routswitch control
//! - REAPER `.rpp` project generator and non-destructive `.rpp` template/session adapter
//! - `.X32Reaper.ini` generator compatible with `x32_reaper`
//! - Live REAPER OSC track naming, record-arming, and transport control

use crate::mixer::{record_routing_entries, MixerManager, MixerSnapshotState, RoutingState};
use crate::UiEvent;
use anyhow::{Context, Result};
use osc_lib::{OscArg, OscMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Write as _;
use std::net::SocketAddr;
use std::path::Path;
use tokio::net::UdpSocket;
use x32_lib::model::fader_to_db;
use x32_lib::safety::ConsoleSnapshot;

/// `/config/routing/routswitch` value for live stage inputs (`REC`).
pub const ROUTSWITCH_REC: i32 = 0;
/// `/config/routing/routswitch` value for virtual soundcheck (`PLAY`).
pub const ROUTSWITCH_PLAY: i32 = 1;
/// `/config/routing/PLAY/{1-8,9-16,17-24,25-32}` values pointing to Card 1–32.
pub const PLAY_CARD_BLOCKS: [i32; 4] = [16, 17, 18, 19];
/// `/config/routing/PLAY/AUX` value pointing to `CARD1-6`.
pub const PLAY_AUX_CARD1_6: i32 = 12;

/// Human-readable label for `/config/routing/IN/*` and `/config/routing/PLAY/*` indices.
pub fn routing_in_label(val: i32) -> &'static str {
    match val {
        0 => "AN1-8 (Local)",
        1 => "AN9-16 (Local)",
        2 => "AN17-24 (Local)",
        3 => "AN25-32 (Local)",
        4 => "A1-8 (AES50-A)",
        5 => "A9-16 (AES50-A)",
        6 => "A17-24 (AES50-A)",
        7 => "A25-32 (AES50-A)",
        8 => "A33-40 (AES50-A)",
        9 => "A41-48 (AES50-A)",
        10 => "B1-8 (AES50-B)",
        11 => "B9-16 (AES50-B)",
        12 => "B17-24 (AES50-B)",
        13 => "B25-32 (AES50-B)",
        14 => "B33-40 (AES50-B)",
        15 => "B41-48 (AES50-B)",
        16 => "CARD1-8",
        17 => "CARD9-16",
        18 => "CARD17-24",
        19 => "CARD25-32",
        _ => "Unknown",
    }
}

/// Human-readable label for `/config/routing/IN/AUX` and `/config/routing/PLAY/AUX` indices.
pub fn routing_aux_label(val: i32) -> &'static str {
    match val {
        0 => "AUX1-4",
        1 => "AN1-2",
        2 => "AN1-4",
        3 => "AN1-6",
        4 => "A1-2",
        5 => "A1-4",
        6 => "A1-6",
        7 => "B1-2",
        8 => "B1-4",
        9 => "B1-6",
        10 => "CARD1-2",
        11 => "CARD1-4",
        12 => "CARD1-6",
        _ => "Unknown",
    }
}

/// Human-readable label for `/config/routing/CARD/*` indices.
pub fn routing_card_label(val: i32) -> &'static str {
    match val {
        0..=19 => routing_in_label(val),
        20 => "OUT1-8",
        21 => "OUT9-16",
        22 => "P16 1-8",
        23 => "P16 9-16",
        24 => "AUX1-6/Mon",
        25 => "AuxIN1-6/TB",
        26 => "UOUT1-8",
        27 => "UOUT9-16",
        28 => "UOUT17-24",
        29 => "UOUT25-32",
        30 => "UOUT33-40",
        31 => "UOUT41-48",
        _ => "Unknown",
    }
}

/// Maps an X32 scribble-strip color index (`0..=15`) to `(css_hex, reaper_peakcol_bgr)`.
pub fn x32_color_to_css_and_reaper(color_idx: i32) -> (&'static str, u32) {
    let base = color_idx.rem_euclid(8);
    let (r, g, b, hex): (u32, u32, u32, &'static str) = match base {
        0 => (85, 92, 108, "#555c6c"),    // OFF / neutral slate
        1 => (220, 62, 62, "#dc3e3e"),    // RD
        2 => (62, 196, 98, "#3ec462"),    // GN
        3 => (230, 196, 56, "#e6c438"),   // YE
        4 => (68, 134, 240, "#4486f0"),   // BL
        5 => (210, 76, 214, "#d24cd6"),   // MG
        6 => (52, 206, 218, "#34ceda"),   // CY
        _ => (226, 232, 240, "#e2e8f0"),  // WH
    };
    let peakcol = 0x0100_0000 | (b << 16) | (g << 8) | r;
    (hex, peakcol)
}

/// Single routing diff line between the current X32 routing and the planned VSC routing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingChange {
    /// Target OSC address path.
    pub path: String,
    /// Human-readable section name.
    pub label: String,
    /// Previous integer index.
    pub before: i32,
    /// Planned integer index.
    pub after: i32,
    /// Human-readable previous source label.
    pub before_label: String,
    /// Human-readable planned source label.
    pub after_label: String,
}

/// Planned X32 routing modifications required for Virtual Soundcheck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VscRoutingPlan {
    /// Target `/config/routing/CARD/{1-8,9-16,17-24,25-32}` values.
    pub card_blocks: [i32; 4],
    /// Target `/config/routing/PLAY/{1-8,9-16,17-24,25-32}` values.
    pub play_blocks: [i32; 4],
    /// Optional target `/config/routing/PLAY/AUX` value.
    pub play_aux: Option<i32>,
    /// Ordered diff entries for UI preview.
    pub changes: Vec<RoutingChange>,
    /// Advisory warnings (e.g. preserved `UOUT` USB feeds or active AES50 stagebox protection).
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Computes a safe Virtual Soundcheck routing plan from `current` console routing.
///
/// Stage inputs (`IN/1-32`) are mirrored to `CARD` outputs whenever they point to physical
/// inputs (`< 16`); if an `IN` block currently points to `CARD` (`>= 16`), `CARD` defaults
/// to `AN1-8..25-32` (`0..=3`) so the card does not loop back on itself. `PLAY` blocks are
/// always assigned to `CARD1-8..CARD25-32` (`16..=19`).
pub fn compute_vsc_routing_plan(current: &RoutingState, include_aux_card: bool) -> VscRoutingPlan {
    compute_vsc_routing_plan_with_options(current, include_aux_card, false)
}

/// Computes a Virtual Soundcheck routing plan with optional preservation of existing `UOUT` Card feeds.
pub fn compute_vsc_routing_plan_with_options(
    current: &RoutingState,
    include_aux_card: bool,
    preserve_card_uout: bool,
) -> VscRoutingPlan {
    let block_names = ["1-8", "9-16", "17-24", "25-32"];
    let mut card_blocks = [0i32; 4];
    let mut changes = Vec::new();
    let mut warnings = Vec::new();

    for i in 0..4 {
        let cur_card = current.card_blocks[i];
        let is_uout = (26..=31).contains(&cur_card);
        if preserve_card_uout && is_uout {
            card_blocks[i] = cur_card;
            warnings.push(format!(
                "Preserved Card Out {} ({}) — disable 'Preserve Card UOUT' to record raw stage inputs on channels {}.",
                block_names[i],
                routing_card_label(cur_card),
                block_names[i]
            ));
            continue;
        }
        if is_uout {
            warnings.push(format!(
                "Card Out {} currently uses {} (User Out); applying VSC will switch it to raw stage inputs (backup saved).",
                block_names[i],
                routing_card_label(cur_card)
            ));
        }

        let stage_src = if current.in_blocks[i] >= 0 && current.in_blocks[i] < 16 {
            current.in_blocks[i]
        } else {
            i as i32
        };
        card_blocks[i] = stage_src;
        changes.push(RoutingChange {
            path: format!("/config/routing/CARD/{}", block_names[i]),
            label: format!("Card Out {}", block_names[i]),
            before: cur_card,
            after: stage_src,
            before_label: routing_card_label(cur_card).to_string(),
            after_label: routing_card_label(stage_src).to_string(),
        });
    }

    for i in 0..4 {
        let target = PLAY_CARD_BLOCKS[i];
        changes.push(RoutingChange {
            path: format!("/config/routing/PLAY/{}", block_names[i]),
            label: format!("Play Bank {}", block_names[i]),
            before: current.play_blocks[i],
            after: target,
            before_label: routing_in_label(current.play_blocks[i]).to_string(),
            after_label: routing_in_label(target).to_string(),
        });
    }

    let play_aux = if include_aux_card {
        let target_aux = if (10..=12).contains(&current.in_aux) {
            current.in_aux
        } else {
            PLAY_AUX_CARD1_6
        };
        changes.push(RoutingChange {
            path: "/config/routing/PLAY/AUX".to_string(),
            label: "Play Aux In".to_string(),
            before: current.play_aux,
            after: target_aux,
            before_label: routing_aux_label(current.play_aux).to_string(),
            after_label: routing_aux_label(target_aux).to_string(),
        });
        Some(target_aux)
    } else {
        None
    };

    VscRoutingPlan {
        card_blocks,
        play_blocks: PLAY_CARD_BLOCKS,
        play_aux,
        changes,
        warnings,
    }
}

/// Builds a [`ConsoleSnapshot`] storing the console's pre-VSC routing state.
pub fn build_routing_backup_snapshot(current: &RoutingState) -> ConsoleSnapshot {
    let mut snap = ConsoleSnapshot::new();
    record_routing_entries(&mut snap, current);
    snap
}

/// Saves a backup snapshot (if `backup_path` is provided) and sends all `plan` routing changes to the X32.
pub async fn apply_vsc_routing(
    mixer: &MixerManager,
    plan: &VscRoutingPlan,
    backup_path: Option<&Path>,
) -> Result<usize> {
    let current = mixer.snapshot().await;
    if let Some(path) = backup_path {
        let backup = build_routing_backup_snapshot(&current.routing);
        backup.save_to_file(path)?;
    }

    for change in &plan.changes {
        mixer
            .send_osc(&change.path, vec![OscArg::Int(change.after)])
            .await?;
    }

    Ok(plan.changes.len())
}

/// Toggles the X32 global routing switch between `REC` (Live, `0`) and `PLAY` (Virtual Soundcheck, `1`).
pub async fn set_vsc_mode(
    mixer: &MixerManager,
    play_mode: bool,
    events_tx: Option<&tokio::sync::broadcast::Sender<UiEvent>>,
) -> Result<()> {
    let val = if play_mode {
        ROUTSWITCH_PLAY
    } else {
        ROUTSWITCH_REC
    };
    mixer
        .send_osc("/config/routing/routswitch", vec![OscArg::Int(val)])
        .await?;
    let snap = mixer.snapshot().await;
    if let Some(tx) = events_tx {
        let _ = tx.send(UiEvent::State(snap));
        let _ = tx.send(UiEvent::VscStatus {
            message: if play_mode {
                "X32 switched to PLAY (Virtual Soundcheck)".to_string()
            } else {
                "X32 switched to REC (Live Inputs)".to_string()
            },
            play_mode,
        });
    }
    Ok(())
}

/// Per-channel track specification used when building or adapting a REAPER `.rpp` session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VscTrackSpec {
    /// 1-based X32 channel index (`1..=32`).
    pub channel: u8,
    /// Track display name.
    pub name: String,
    /// X32 scribble-strip color index (`0..=15`).
    pub color: i32,
    /// X32 scribble-strip icon index.
    pub icon: i32,
    /// Whether this channel is stereo-linked with its odd/even pair.
    pub linked_stereo: bool,
    /// Whether to include this channel in the REAPER project.
    pub include: bool,
    /// Physical X32 input source (`1..=32`) from `/ch/XX/config/source`.
    #[serde(default)]
    pub source_input: u8,
    /// 1-based X-USB / X-Live card input channel (`1..=32`).
    pub card_in: u8,
    /// 1-based X-USB / X-Live card output return channel (`1..=32`).
    pub card_out: u8,
    /// Console normalized fader (`0.0..=1.0`).
    #[serde(default = "default_fader")]
    pub fader: f32,
    /// Console unmuted/on flag.
    #[serde(default = "default_true")]
    pub on: bool,
    /// Console normalized pan (`0.0..=1.0`, `0.5` = center).
    #[serde(default = "default_pan")]
    pub pan: f32,
}

fn default_fader() -> f32 {
    0.75
}
fn default_pan() -> f32 {
    0.5
}

/// Options controlling REAPER `.rpp` generation and adaptation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VscSessionOptions {
    /// Project title / session name.
    #[serde(default = "default_project_name")]
    pub project_name: String,
    /// Project sample rate in Hz (`48000` or `44100`).
    #[serde(default = "default_sample_rate")]
    pub sample_rate_hz: u32,
    /// Subdirectory where REAPER records audio files (e.g. `"Audio"`).
    #[serde(default = "default_record_path")]
    pub record_path: String,
    /// Merge stereo-linked odd/even X32 channel pairs into single stereo REAPER tracks.
    #[serde(default = "default_true")]
    pub merge_stereo_links: bool,
    /// Arm generated/adapted tracks for recording (`REC 1 ...`).
    #[serde(default = "default_true")]
    pub arm_tracks_for_recording: bool,
    /// Disable Master Send (`MAINSEND 0 0`) so tracks return directly to hardware X32 Card outs.
    #[serde(default = "default_true")]
    pub disable_master_send: bool,
    /// Include channels that still have generic/empty names.
    #[serde(default = "default_true")]
    pub include_empty_channels: bool,
    /// Skip creating duplicate tracks when multiple X32 channels share the same physical source.
    #[serde(default = "default_true")]
    pub deduplicate_shared_sources: bool,
    /// Initialize REAPER track faders, pans, and mutes to mirror the live console mix (digital twin).
    #[serde(default = "default_true")]
    pub digital_twin_mode: bool,
    /// Append `%<icon> %<color>` tags to track names for `x32_reaper` compatibility.
    #[serde(default = "default_true")]
    pub append_x32_reaper_tags: bool,
    /// Prepend DCA 1–8 tracks at the top of the REAPER project.
    #[serde(default)]
    pub include_dca_tracks: bool,
    /// Append Aux In 1–8 tracks at the bottom of the REAPER project.
    #[serde(default)]
    pub include_aux_tracks: bool,
    /// Append Bus 1–16 tracks at the bottom of the REAPER project.
    #[serde(default)]
    pub include_bus_tracks: bool,
    /// Append Matrix 1–6 tracks at the bottom of the REAPER project.
    #[serde(default)]
    pub include_matrix_tracks: bool,
    /// Per-channel track definitions (1..=32).
    #[serde(default)]
    pub selected_channels: Vec<VscTrackSpec>,
    /// Optional DCA 1–8 names from the console.
    #[serde(default)]
    pub dca_names: Vec<String>,
    /// Optional Aux In 1–8 names from the console.
    #[serde(default)]
    pub aux_names: Vec<String>,
    /// Optional Bus 1–16 names from the console.
    #[serde(default)]
    pub bus_names: Vec<String>,
    /// Optional Matrix 1–6 names from the console.
    #[serde(default)]
    pub matrix_names: Vec<String>,
}

fn default_project_name() -> String {
    "X32_Virtual_Soundcheck".to_string()
}
fn default_sample_rate() -> u32 {
    48_000
}
fn default_record_path() -> String {
    "Audio".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for VscSessionOptions {
    fn default() -> Self {
        Self::from_mixer_state(&MixerSnapshotState::default())
    }
}

impl VscSessionOptions {
    /// Builds default [`VscSessionOptions`] populated from a captured [`MixerSnapshotState`].
    pub fn from_mixer_state(state: &MixerSnapshotState) -> Self {
        let has_custom_names = state.channels.iter().any(|ch| {
            let t = ch.name.trim();
            !t.is_empty()
                && !t.eq_ignore_ascii_case(&format!("Ch {:02}", ch.index))
                && !t.eq_ignore_ascii_case(&format!("Ch {}", ch.index))
        });
        let selected_channels = state
            .channels
            .iter()
            .map(|ch| {
                let src = if (1..=32).contains(&ch.source) {
                    ch.source as u8
                } else {
                    ch.index
                };
                VscTrackSpec {
                    channel: ch.index,
                    name: ch.name.clone(),
                    color: ch.color,
                    icon: ch.icon.max(1),
                    linked_stereo: ch.linked_stereo,
                    include: true,
                    source_input: src,
                    card_in: src,
                    card_out: src,
                    fader: ch.fader,
                    on: ch.on,
                    pan: ch.pan,
                }
            })
            .collect();
        let dca_names = state.dcas.iter().map(|d| d.name.clone()).collect();
        let aux_names = state.auxins.iter().map(|a| a.name.clone()).collect();
        let bus_names = state.buses.iter().map(|b| b.name.clone()).collect();
        let matrix_names = state.matrices.iter().map(|m| m.name.clone()).collect();
        Self {
            project_name: "X32_Virtual_Soundcheck".to_string(),
            sample_rate_hz: if state.sample_rate_hz == 44_100 {
                44_100
            } else {
                48_000
            },
            record_path: "Audio".to_string(),
            merge_stereo_links: true,
            arm_tracks_for_recording: true,
            disable_master_send: true,
            include_empty_channels: !has_custom_names,
            deduplicate_shared_sources: true,
            digital_twin_mode: true,
            append_x32_reaper_tags: true,
            include_dca_tracks: false,
            include_aux_tracks: false,
            include_bus_tracks: false,
            include_matrix_tracks: false,
            selected_channels,
            dca_names,
            aux_names,
            bus_names,
            matrix_names,
        }
    }
}

/// Resolved REAPER track entry produced from [`VscSessionOptions`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolvedReaperTrack {
    /// 1-based REAPER track number.
    pub reaper_track_num: usize,
    /// Display name (including optional `%icon %color` tags).
    pub formatted_name: String,
    /// Raw track name without tags.
    pub raw_name: String,
    /// REAPER `PEAKCOL` BGR color integer.
    pub peakcol: u32,
    /// Whether this is a stereo track.
    pub is_stereo: bool,
    /// Whether this track should be record-armed.
    pub rec_arm: bool,
    /// REAPER `REC` hardware input index (`0..31` mono, `1024..` stereo).
    pub rec_input: u32,
    /// REAPER `HWOUT` hardware output index (`1024..` mono, `0..` stereo).
    pub hwout_index: u32,
    /// `MAINSEND` flag (`0` or `1`).
    pub main_send: u8,
    /// Deterministic GUID string.
    pub guid: String,
    /// Console normalized fader (`0.0..=1.0`).
    pub fader: f32,
    /// Console unmuted/on flag.
    pub on: bool,
    /// Console normalized pan (`0.0..=1.0`).
    pub pan: f32,
}

/// Expands [`VscSessionOptions`] into the ordered list of REAPER channel tracks (handling stereo link merging and shared-source dedup).
pub fn resolve_channel_tracks(options: &VscSessionOptions) -> Vec<ResolvedReaperTrack> {
    let mut specs = options.selected_channels.clone();
    specs.sort_by_key(|s| s.channel);

    let mut tracks = Vec::new();
    let mut claimed_sources: HashSet<u8> = HashSet::new();
    let mut idx = 0usize;
    let arm_flag = options.arm_tracks_for_recording;
    let main_send = if options.disable_master_send { 0 } else { 1 };

    while idx < specs.len() {
        let ch = &specs[idx];
        if !ch.include {
            idx += 1;
            continue;
        }
        let trimmed = ch.name.trim();
        let is_generic = trimmed.is_empty()
            || trimmed.eq_ignore_ascii_case(&format!("Ch {:02}", ch.channel))
            || trimmed.eq_ignore_ascii_case(&format!("Ch {}", ch.channel));
        if !options.include_empty_channels && is_generic {
            idx += 1;
            continue;
        }

        if options.deduplicate_shared_sources && claimed_sources.contains(&ch.card_out) {
            idx += 1;
            continue;
        }

        // Check odd/even stereo link merge
        let can_merge_stereo = options.merge_stereo_links
            && ch.linked_stereo
            && ch.channel % 2 == 1
            && idx + 1 < specs.len()
            && specs[idx + 1].channel == ch.channel + 1
            && specs[idx + 1].linked_stereo
            && specs[idx + 1].include
            && specs[idx + 1].card_out == ch.card_out.saturating_add(1);

        let reaper_num = tracks.len() + 1;
        let (_, peakcol) = x32_color_to_css_and_reaper(ch.color);
        let x32_color_tag = (ch.color.rem_euclid(8)).max(1);
        let x32_icon_tag = ch.icon.max(1);

        if can_merge_stereo {
            let next = &specs[idx + 1];
            claimed_sources.insert(ch.card_out);
            claimed_sources.insert(next.card_out);
            let base_name = if trimmed.is_empty() {
                format!("Ch {}-{}", ch.channel, next.channel)
            } else {
                format!("{} (ST)", sanitize_rpp_name(trimmed))
            };
            let formatted_name = if options.append_x32_reaper_tags {
                format!("{} %{} %{}", base_name, x32_icon_tag, x32_color_tag)
            } else {
                base_name.clone()
            };
            let in_zero = ch.card_in.saturating_sub(1) as u32;
            let out_zero = ch.card_out.saturating_sub(1) as u32;
            tracks.push(ResolvedReaperTrack {
                reaper_track_num: reaper_num,
                formatted_name,
                raw_name: base_name,
                peakcol,
                is_stereo: true,
                rec_arm: arm_flag,
                rec_input: 1024 + in_zero,
                hwout_index: out_zero,
                main_send,
                guid: deterministic_guid("CHST", ch.channel as u32),
                fader: ch.fader,
                on: ch.on,
                pan: ch.pan,
            });
            idx += 2;
        } else {
            claimed_sources.insert(ch.card_out);
            let base_name = if trimmed.is_empty() {
                format!("Ch {:02}", ch.channel)
            } else {
                sanitize_rpp_name(trimmed)
            };
            let formatted_name = if options.append_x32_reaper_tags {
                format!("{} %{} %{}", base_name, x32_icon_tag, x32_color_tag)
            } else {
                base_name.clone()
            };
            let in_zero = ch.card_in.saturating_sub(1) as u32;
            let out_zero = ch.card_out.saturating_sub(1) as u32;
            tracks.push(ResolvedReaperTrack {
                reaper_track_num: reaper_num,
                formatted_name,
                raw_name: base_name,
                peakcol,
                is_stereo: false,
                rec_arm: arm_flag,
                rec_input: in_zero,
                hwout_index: 1024 + out_zero,
                main_send,
                guid: deterministic_guid("CHMN", ch.channel as u32),
                fader: ch.fader,
                on: ch.on,
                pan: ch.pan,
            });
            idx += 1;
        }
    }

    tracks
}

/// Generates a complete REAPER `.rpp` project configured for X32 Virtual Soundcheck.
pub fn generate_rpp(options: &VscSessionOptions) -> String {
    let mut out = String::with_capacity(32 * 1024);
    let sr = if options.sample_rate_hz == 44_100 {
        44_100
    } else {
        48_000
    };
    let rec_path = sanitize_rpp_name(&options.record_path);
    let channel_tracks = resolve_channel_tracks(options);

    let _ = writeln!(out, "<REAPER_PROJECT 0.1 \"6.80/x64\" 1700000000");
    let _ = writeln!(out, "  RIPPLE 0");
    let _ = writeln!(out, "  GROUPOVERRIDE 0 0 0");
    let _ = writeln!(out, "  AUTOXFADE 1");
    let _ = writeln!(out, "  ENVATTACH 1");
    let _ = writeln!(out, "  MIXERUIFLAGS 11 305");
    let _ = writeln!(out, "  PEAKGAIN 1");
    let _ = writeln!(out, "  PANLAW 1");
    let _ = writeln!(out, "  USE_REC_CFG 0");
    let _ = writeln!(out, "  RECMODE 1");
    let _ = writeln!(out, "  LOOP 0");
    let _ = writeln!(out, "  RECORD_PATH \"{}\" \"\"", rec_path);
    let _ = writeln!(out, "  SAMPLERATE {} 0 0", sr);
    let _ = writeln!(out, "  <METRONOME 6 2");
    let _ = writeln!(out, "    VOL 0.25 0.125");
    let _ = writeln!(out, "    FREQ 800 1600 1");
    let _ = writeln!(out, "    BEATLEN 4");
    let _ = writeln!(out, "  >");
    let _ = writeln!(out, "  TEMPO 120 4 4");
    let _ = writeln!(out, "  MASTER_NCH 2 2");
    let _ = writeln!(out, "  MASTER_VOLUME 1 0 -1 -1 1");
    let _ = writeln!(out, "  MASTER_FX 1");
    let _ = writeln!(out, "  <PROJBAY");
    let _ = writeln!(out, "  >");

    if options.include_dca_tracks {
        for dca_idx in 1..=8usize {
            let dca_name = options
                .dca_names
                .get(dca_idx - 1)
                .map(|s| sanitize_rpp_name(s.trim()))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("DCA {}", dca_idx));
            let name_with_tags = if options.append_x32_reaper_tags {
                format!("{} %1 %7", dca_name)
            } else {
                dca_name
            };
            write_rpp_track(
                &mut out,
                &deterministic_guid("DCAT", dca_idx as u32),
                &name_with_tags,
                0x01E2_E8F0,
                false,
                0,
                None,
                1,
                1.0,
                0.0,
                false,
            );
        }
    }

    for trk in &channel_tracks {
        let (vol_amp, pan_rpp, muted) = if options.digital_twin_mode {
            let db = fader_to_db(trk.fader);
            let amp = if trk.fader <= 0.0005 {
                0.0
            } else {
                10.0_f64.powf(db as f64 / 20.0).clamp(0.0, 3.16228)
            };
            let pan_val = ((trk.pan as f64) * 2.0 - 1.0).clamp(-1.0, 1.0);
            (amp, pan_val, !trk.on)
        } else {
            (1.0, 0.0, false)
        };
        write_rpp_track(
            &mut out,
            &trk.guid,
            &trk.formatted_name,
            trk.peakcol,
            trk.rec_arm,
            trk.rec_input,
            Some(trk.hwout_index),
            trk.main_send,
            vol_amp,
            pan_rpp,
            muted,
        );
    }

    if options.include_aux_tracks {
        for aux_idx in 1..=8usize {
            let aux_name = options
                .aux_names
                .get(aux_idx - 1)
                .map(|s| sanitize_rpp_name(s.trim()))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Aux {:02}", aux_idx));
            let name_with_tags = if options.append_x32_reaper_tags {
                format!("{} %1 %3", aux_name)
            } else {
                aux_name
            };
            write_rpp_track(
                &mut out,
                &deterministic_guid("AUXT", aux_idx as u32),
                &name_with_tags,
                0x0138_C4E6,
                false,
                0,
                None,
                1,
                1.0,
                0.0,
                false,
            );
        }
    }

    if options.include_bus_tracks {
        for bus_idx in 1..=16usize {
            let bus_name = options
                .bus_names
                .get(bus_idx - 1)
                .map(|s| sanitize_rpp_name(s.trim()))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Bus {:02}", bus_idx));
            let name_with_tags = if options.append_x32_reaper_tags {
                format!("{} %1 %6", bus_name)
            } else {
                bus_name
            };
            write_rpp_track(
                &mut out,
                &deterministic_guid("BUST", bus_idx as u32),
                &name_with_tags,
                0x01DA_CE34,
                false,
                0,
                None,
                1,
                1.0,
                0.0,
                false,
            );
        }
    }

    if options.include_matrix_tracks {
        for mtx_idx in 1..=6usize {
            let mtx_name = options
                .matrix_names
                .get(mtx_idx - 1)
                .map(|s| sanitize_rpp_name(s.trim()))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Mtx {:02}", mtx_idx));
            let name_with_tags = if options.append_x32_reaper_tags {
                format!("{} %1 %5", mtx_name)
            } else {
                mtx_name
            };
            write_rpp_track(
                &mut out,
                &deterministic_guid("MTXT", mtx_idx as u32),
                &name_with_tags,
                0x01D6_4CD2,
                false,
                0,
                None,
                1,
                1.0,
                0.0,
                false,
            );
        }
    }

    let _ = writeln!(out, ">");
    out
}

#[allow(clippy::too_many_arguments)]
fn write_rpp_track(
    out: &mut String,
    guid: &str,
    name: &str,
    peakcol: u32,
    rec_arm: bool,
    rec_input: u32,
    hwout: Option<u32>,
    main_send: u8,
    vol_amp: f64,
    pan_rpp: f64,
    muted: bool,
) {
    let arm = i32::from(rec_arm);
    let mute_flag = i32::from(muted);
    let _ = writeln!(out, "  <TRACK {}", guid);
    let _ = writeln!(out, "    NAME \"{}\"", name);
    let _ = writeln!(out, "    PEAKCOL {}", peakcol);
    let _ = writeln!(out, "    BEAT -1");
    let _ = writeln!(out, "    AUTOMODE 0");
    let _ = writeln!(out, "    VOLPAN {:.6} {:.4} -1 -1 1", vol_amp, pan_rpp);
    let _ = writeln!(out, "    MUTESOLO {} 0 0", mute_flag);
    let _ = writeln!(out, "    IPHASE 0");
    let _ = writeln!(out, "    ISBUS 0 0");
    let _ = writeln!(out, "    BUSCOMP 0 0 0 0 0");
    let _ = writeln!(out, "    SHOWINMIX 1 0.6667 1 1 0.5 0 0 0");
    let _ = writeln!(out, "    FREEMODE 0");
    let _ = writeln!(out, "    SEL 0");
    let _ = writeln!(out, "    REC {} {} 0 0 0 0 0", arm, rec_input);
    let _ = writeln!(out, "    VU 2");
    let _ = writeln!(out, "    TRACKHEIGHT 0 0 0 0 -1 0");
    let _ = writeln!(out, "    INQ 0 0 0 0.5 100 0 0 100");
    let _ = writeln!(out, "    NCHAN 2");
    let _ = writeln!(out, "    FX 1");
    let _ = writeln!(out, "    TRACKID {}", guid);
    let _ = writeln!(out, "    PERF 0");
    let _ = writeln!(out, "    MIDIOUT -1");
    let _ = writeln!(out, "    MAINSEND {} 0", main_send);
    if let Some(hw) = hwout {
        let _ = writeln!(out, "    HWOUT {} 0 1 0 0 0 0 -1:U -1", hw);
    }
    let _ = writeln!(out, "  >");
}

/// Adapts an existing REAPER `.rpp` project/template in-place for X32 Virtual Soundcheck.
///
/// Updates project-level `SAMPLERATE` and `RECORD_PATH`, skips DCA header tracks (names
/// starting with `"DCA "`), and updates sequential channel tracks' `NAME`, `PEAKCOL`, `REC`,
/// `MAINSEND`, and `HWOUT` while preserving all `<ITEM>` audio media, `<FXCHAIN>` plugins,
/// automation envelopes, markers, and `<EXTENSIONS>`.
pub fn adapt_rpp(existing_rpp: &str, options: &VscSessionOptions) -> Result<String> {
    if !existing_rpp.contains("<REAPER_PROJECT") {
        anyhow::bail!("Input does not appear to be a valid REAPER .rpp file");
    }

    let resolved = resolve_channel_tracks(options);
    let sr = if options.sample_rate_hz == 44_100 {
        44_100
    } else {
        48_000
    };
    let rec_path = sanitize_rpp_name(&options.record_path);
    let line_ending = if existing_rpp.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    let raw_lines: Vec<&str> = existing_rpp.lines().collect();
    let mut output_lines: Vec<String> = Vec::with_capacity(raw_lines.len() + 64);
    let mut idx = 0usize;
    let mut track_cursor = 0usize;

    while idx < raw_lines.len() {
        let line = raw_lines[idx];
        let trimmed = line.trim_start();

        // Project-level SAMPLERATE & RECORD_PATH (only outside track/sub-blocks)
        if trimmed.starts_with("SAMPLERATE ") {
            let indent = &line[..line.len() - trimmed.len()];
            output_lines.push(format!("{}SAMPLERATE {} 0 0", indent, sr));
            idx += 1;
            continue;
        }
        if trimmed.starts_with("RECORD_PATH ") && !options.record_path.trim().is_empty() {
            let indent = &line[..line.len() - trimmed.len()];
            output_lines.push(format!("{}RECORD_PATH \"{}\" \"\"", indent, rec_path));
            idx += 1;
            continue;
        }

        if trimmed.starts_with("<TRACK") {
            // Collect the full <TRACK ...> block by tracking nested <...> depth.
            let mut block: Vec<String> = vec![line.to_string()];
            let mut depth: i32 = 1;
            idx += 1;
            while idx < raw_lines.len() && depth > 0 {
                let l = raw_lines[idx];
                let t = l.trim();
                if t.starts_with('<') {
                    depth += 1;
                } else if t == ">" {
                    depth -= 1;
                }
                block.push(l.to_string());
                idx += 1;
            }

            let is_dca_track = block.iter().any(|l| {
                let t = l.trim();
                if let Some(rest) = t.strip_prefix("NAME ") {
                    let clean = rest.trim().trim_matches('"');
                    clean.to_ascii_uppercase().starts_with("DCA ")
                } else {
                    false
                }
            });

            if !is_dca_track && track_cursor < resolved.len() {
                let rewritten = rewrite_track_block(&block, &resolved[track_cursor]);
                output_lines.extend(rewritten);
                track_cursor += 1;
            } else {
                output_lines.extend(block);
            }
            continue;
        }

        output_lines.push(line.to_string());
        idx += 1;
    }

    let mut joined = output_lines.join(line_ending);
    if existing_rpp.ends_with('\n') {
        joined.push_str(line_ending);
    }
    Ok(joined)
}

fn rewrite_track_block(block: &[String], trk: &ResolvedReaperTrack) -> Vec<String> {
    let mut out = Vec::with_capacity(block.len() + 2);
    let mut depth = 0i32;
    let mut saw_hwout = false;
    let mut saw_name = false;
    let mut saw_peakcol = false;
    let mut saw_rec = false;
    let mut saw_mainsend = false;
    let hwout_line = format!("    HWOUT {} 0 1 0 0 0 0 -1:U -1", trk.hwout_index);

    for (i, line) in block.iter().enumerate() {
        let t = line.trim();
        let is_closing_root = i + 1 == block.len() && t == ">";

        if t.starts_with('<') {
            depth += 1;
            out.push(line.clone());
            continue;
        }
        if t == ">" {
            depth -= 1;
            if is_closing_root {
                if !saw_hwout {
                    out.push(hwout_line.clone());
                }
            }
            out.push(line.clone());
            continue;
        }

        // Only rewrite top-level attributes inside <TRACK ...> (depth == 1)
        if depth == 1 {
            let indent = &line[..line.len() - line.trim_start().len()];
            if t.starts_with("NAME ") {
                saw_name = true;
                out.push(format!("{}NAME \"{}\"", indent, trk.formatted_name));
                continue;
            }
            if t.starts_with("PEAKCOL ") {
                saw_peakcol = true;
                out.push(format!("{}PEAKCOL {}", indent, trk.peakcol));
                continue;
            }
            if t.starts_with("REC ") {
                saw_rec = true;
                out.push(format!(
                    "{}REC {} {} 0 0 0 0 0",
                    indent,
                    i32::from(trk.rec_arm),
                    trk.rec_input
                ));
                continue;
            }
            if t.starts_with("MAINSEND ") {
                saw_mainsend = true;
                out.push(format!("{}MAINSEND {} 0", indent, trk.main_send));
                continue;
            }
            if t.starts_with("HWOUT ") {
                if !saw_hwout {
                    saw_hwout = true;
                    out.push(format!(
                        "{}HWOUT {} 0 1 0 0 0 0 -1:U -1",
                        indent, trk.hwout_index
                    ));
                }
                continue;
            }
        }

        out.push(line.clone());
    }

    // In case a minimal template lacked NAME/PEAKCOL/REC/MAINSEND, insert right after <TRACK ...>
    let mut extras = Vec::new();
    if !saw_name {
        extras.push(format!("    NAME \"{}\"", trk.formatted_name));
    }
    if !saw_peakcol {
        extras.push(format!("    PEAKCOL {}", trk.peakcol));
    }
    if !saw_rec {
        extras.push(format!(
            "    REC {} {} 0 0 0 0 0",
            i32::from(trk.rec_arm),
            trk.rec_input
        ));
    }
    if !saw_mainsend {
        extras.push(format!("    MAINSEND {} 0", trk.main_send));
    }
    if !extras.is_empty() && !out.is_empty() {
        out.splice(1..1, extras);
    }

    out
}

/// Options for generating a `.X32Reaper.ini` configuration file consumed by `x32_reaper`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaperIniOptions {
    /// IP address of the X32 console (host only, without `:10023`).
    pub x32_ip: String,
    /// IP address of the machine running REAPER.
    pub reaper_ip: String,
    /// UDP port REAPER listens on for incoming OSC (default `8000`).
    #[serde(default = "default_reaper_send_port")]
    pub reaper_send_port: u16,
    /// UDP port `x32_reaper` listens on for REAPER outgoing OSC (default `9000`).
    #[serde(default = "default_reaper_recv_port")]
    pub reaper_recv_port: u16,
    /// Enable transport control on X32 User Assign Bank C.
    #[serde(default = "default_true")]
    pub transport_on: bool,
    /// Enable channel bank switching.
    #[serde(default = "default_true")]
    pub ch_bank_on: bool,
    /// Enable marker button sync.
    #[serde(default = "default_true")]
    pub marker_btn_on: bool,
    /// Enable ReaEQ/ReaComp FX sync.
    #[serde(default = "default_true")]
    pub eq_ctrl_on: bool,
    /// Enable Master fader/mute/pan sync.
    #[serde(default = "default_true")]
    pub master_on: bool,
    /// First REAPER track mapped to X32 channels (1-based).
    #[serde(default = "default_one")]
    pub trk_min: i32,
    /// Last REAPER track mapped to X32 channels (1-based).
    #[serde(default = "default_32")]
    pub trk_max: i32,
    /// First REAPER track mapped to X32 auxes.
    #[serde(default)]
    pub aux_min: i32,
    /// Last REAPER track mapped to X32 auxes.
    #[serde(default)]
    pub aux_max: i32,
    /// First REAPER track mapped to FX returns.
    #[serde(default)]
    pub fxr_min: i32,
    /// Last REAPER track mapped to FX returns.
    #[serde(default)]
    pub fxr_max: i32,
    /// First REAPER track mapped to buses.
    #[serde(default)]
    pub bus_min: i32,
    /// Last REAPER track mapped to buses.
    #[serde(default)]
    pub bus_max: i32,
    /// First REAPER track mapped to DCAs.
    #[serde(default)]
    pub dca_min: i32,
    /// Last REAPER track mapped to DCAs.
    #[serde(default)]
    pub dca_max: i32,
    /// Number of channels per fader bank (default `32` or `16`).
    #[serde(default = "default_32")]
    pub bank_size: i32,
}

fn default_reaper_send_port() -> u16 {
    8000
}
fn default_reaper_recv_port() -> u16 {
    9000
}
fn default_one() -> i32 {
    1
}
fn default_32() -> i32 {
    32
}

impl Default for ReaperIniOptions {
    fn default() -> Self {
        Self {
            x32_ip: "192.168.1.64".to_string(),
            reaper_ip: "127.0.0.1".to_string(),
            reaper_send_port: 8000,
            reaper_recv_port: 9000,
            transport_on: true,
            ch_bank_on: true,
            marker_btn_on: true,
            eq_ctrl_on: true,
            master_on: true,
            trk_min: 1,
            trk_max: 32,
            aux_min: 0,
            aux_max: 0,
            fxr_min: 0,
            fxr_max: 0,
            bus_min: 0,
            bus_max: 0,
            dca_min: 0,
            dca_max: 0,
            bank_size: 32,
        }
    }
}

/// Generates a `.X32Reaper.ini` file matching the exact positional schema of `x32_reaper::config::Config::load`.
pub fn generate_x32_reaper_ini(opts: &ReaperIniOptions) -> String {
    let x32_host = opts
        .x32_ip
        .split(':')
        .next()
        .unwrap_or("127.0.0.1")
        .trim();
    let x32_host = if x32_host.is_empty() {
        "127.0.0.1"
    } else {
        x32_host
    };
    let reaper_host = opts.reaper_ip.trim();
    let reaper_host = if reaper_host.is_empty() {
        "127.0.0.1"
    } else {
        reaper_host
    };

    // xx_send_mask: 0x0001..0x1FFF enables pan/fader/name/mute/select/send/solo/fx/master
    let xx_mask = 0x1FFF;
    let xr_mask = 0x03FF;
    let mut out = String::with_capacity(512);
    let _ = writeln!(out, "800 600 0 10 2 {} {}", xx_mask, xr_mask);
    let _ = writeln!(out, "{}", x32_host);
    let _ = writeln!(out, "{}", reaper_host);
    let _ = writeln!(out, "{}", opts.reaper_send_port);
    let _ = writeln!(out, "{}", opts.reaper_recv_port);
    let _ = writeln!(
        out,
        "{} {} {} 7 {} {}",
        i32::from(opts.transport_on),
        i32::from(opts.ch_bank_on),
        i32::from(opts.marker_btn_on),
        i32::from(opts.eq_ctrl_on),
        i32::from(opts.master_on)
    );
    let _ = writeln!(
        out,
        "{} {} {} {} {} {} {} {} {} {} 0",
        opts.trk_min,
        opts.trk_max,
        opts.aux_min,
        opts.aux_max,
        opts.fxr_min,
        opts.fxr_max,
        opts.bus_min,
        opts.bus_max,
        opts.dca_min,
        opts.dca_max
    );
    for _ in 0..8 {
        let _ = writeln!(out, "0 0");
    }
    let _ = writeln!(out, "11 12 10 0 {}", opts.bank_size.max(1));
    out
}

/// Pushes track names and record-arm states directly to a live REAPER instance over OSC.
pub async fn push_reaper_osc(
    reaper_ip: &str,
    reaper_port: u16,
    tracks: &[ResolvedReaperTrack],
    arm_all: Option<bool>,
) -> Result<usize> {
    let addr: SocketAddr = format!("{}:{}", reaper_ip.trim(), reaper_port)
        .parse()
        .context("Invalid REAPER IP or port")?;
    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    let mut sent = 0usize;

    for trk in tracks {
        let name_msg = OscMessage::new(
            format!("/track/{}/name", trk.reaper_track_num),
            vec![OscArg::String(trk.raw_name.clone())],
        );
        sock.send_to(&name_msg.to_bytes()?, addr).await?;
        sent += 1;

        let arm = arm_all.unwrap_or(trk.rec_arm);
        let arm_msg = OscMessage::new(
            format!("/track/{}/recarm", trk.reaper_track_num),
            vec![OscArg::Float(if arm { 1.0 } else { 0.0 })],
        );
        sock.send_to(&arm_msg.to_bytes()?, addr).await?;
        sent += 1;

        let vol_msg = OscMessage::new(
            format!("/track/{}/volume", trk.reaper_track_num),
            vec![OscArg::Float(trk.fader.clamp(0.0, 1.0))],
        );
        sock.send_to(&vol_msg.to_bytes()?, addr).await?;
        sent += 1;

        let mute_msg = OscMessage::new(
            format!("/track/{}/mute", trk.reaper_track_num),
            vec![OscArg::Float(if trk.on { 0.0 } else { 1.0 })],
        );
        sock.send_to(&mute_msg.to_bytes()?, addr).await?;
        sent += 1;

        let pan_msg = OscMessage::new(
            format!("/track/{}/pan", trk.reaper_track_num),
            vec![OscArg::Float(trk.pan.clamp(0.0, 1.0))],
        );
        sock.send_to(&pan_msg.to_bytes()?, addr).await?;
        sent += 1;
    }

    let refresh = OscMessage::new("/action/41743".to_string(), vec![]);
    sock.send_to(&refresh.to_bytes()?, addr).await?;
    sent += 1;
    Ok(sent)
}

/// Sends a transport or control-surface refresh action to REAPER over OSC.
///
/// Supported actions: `"play"`, `"stop"`, `"record"`, `"pause"`, `"refresh"`, `"arm_all"`, `"disarm_all"`.
pub async fn send_reaper_transport(
    reaper_ip: &str,
    reaper_port: u16,
    action: &str,
) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", reaper_ip.trim(), reaper_port)
        .parse()
        .context("Invalid REAPER IP or port")?;
    let sock = UdpSocket::bind("0.0.0.0:0").await?;

    let msg = match action {
        "play" => OscMessage::new("/play".to_string(), vec![OscArg::Float(1.0)]),
        "stop" => OscMessage::new("/stop".to_string(), vec![OscArg::Float(1.0)]),
        "record" => OscMessage::new("/record".to_string(), vec![OscArg::Float(1.0)]),
        "pause" => OscMessage::new("/pause".to_string(), vec![OscArg::Float(1.0)]),
        "refresh" => OscMessage::new("/action/41743".to_string(), vec![]),
        "arm_all" => OscMessage::new("/action/40490".to_string(), vec![]),
        "disarm_all" => OscMessage::new("/action/40491".to_string(), vec![]),
        other => anyhow::bail!("Unknown REAPER action: {}", other),
    };
    sock.send_to(&msg.to_bytes()?, addr).await?;
    Ok(())
}

fn sanitize_rpp_name(raw: &str) -> String {
    raw.replace('"', "'")
        .replace(['\r', '\n'], " ")
        .trim()
        .to_string()
}

fn deterministic_guid(tag: &str, idx: u32) -> String {
    let tag_bytes = tag.as_bytes();
    let a = *tag_bytes.first().unwrap_or(&b'X') as u32;
    let b = *tag_bytes.get(1).unwrap_or(&b'3') as u32;
    let c = *tag_bytes.get(2).unwrap_or(&b'2') as u32;
    let d = *tag_bytes.get(3).unwrap_or(&b'V') as u32;
    format!(
        "{{{:02X}{:02X}{:02X}{:02X}-4000-8000-8000-{:012X}}}",
        a,
        b,
        c,
        d,
        idx as u64 + 1
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_vsc_routing_plan_mirrors_stage_inputs_and_avoids_card_loop() {
        let mut current = RoutingState::default();
        current.in_blocks = [4, 5, 16, 3]; // A1-8, A9-16, CARD1-8 (loop!), AN25-32
        current.card_blocks = [0, 0, 0, 0];
        current.play_blocks = [0, 1, 2, 3];
        current.play_aux = 0;

        let plan = compute_vsc_routing_plan(&current, true);
        // Physical stage inputs mirrored to CARD; block 2 (which pointed to CARD1-8) falls back to AN17-24 (2).
        assert_eq!(plan.card_blocks, [4, 5, 2, 3]);
        assert_eq!(plan.play_blocks, [16, 17, 18, 19]);
        assert_eq!(plan.play_aux, Some(12));
        assert_eq!(plan.changes.len(), 9);
    }

    #[test]
    fn test_generate_rpp_mono_and_stereo_linked() {
        let mut opts = VscSessionOptions::default();
        opts.sample_rate_hz = 44_100;
        opts.selected_channels[0].name = "Kick".to_string();
        opts.selected_channels[0].color = 1; // Red
        opts.selected_channels[2].name = "Keys L".to_string();
        opts.selected_channels[2].linked_stereo = true;
        opts.selected_channels[3].name = "Keys R".to_string();
        opts.selected_channels[3].linked_stereo = true;

        // Stereo merge ON: 30 mono + 1 stereo = 31 tracks
        opts.merge_stereo_links = true;
        let rpp_merged = generate_rpp(&opts);
        assert!(rpp_merged.contains("SAMPLERATE 44100 0 0"));
        assert!(rpp_merged.contains("NAME \"Kick %1 %1\""));
        assert!(rpp_merged.contains("NAME \"Keys L (ST) %3 %3\""));
        assert!(rpp_merged.contains("REC 1 1026 0 0 0 0 0")); // Ch 3/4 stereo input = 1024 + 2
        assert!(rpp_merged.contains("HWOUT 2 0 1 0 0 0 0 -1:U -1")); // Stereo out pair 3/4 = 2
        assert!(rpp_merged.contains("MAINSEND 0 0"));
        let track_count_merged = rpp_merged.matches("<TRACK ").count();
        assert_eq!(track_count_merged, 31);

        // Stereo merge OFF: 32 mono tracks
        opts.merge_stereo_links = false;
        let rpp_mono = generate_rpp(&opts);
        assert_eq!(rpp_mono.matches("<TRACK ").count(), 32);
        assert!(rpp_mono.contains("HWOUT 1024 0 1 0 0 0 0 -1:U -1"));
        assert!(rpp_mono.contains("HWOUT 1025 0 1 0 0 0 0 -1:U -1"));
    }

    #[test]
    fn test_adapt_reference_templates() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let tpl1 = std::fs::read_to_string(repo_root.join("docs/X32Template.rpp")).unwrap();
        let tpl2 = std::fs::read_to_string(repo_root.join("docs/Template 48 tracks.rpp")).unwrap();

        let mut opts = VscSessionOptions::default();
        opts.sample_rate_hz = 48_000;
        opts.record_path = "VSC_Audio".to_string();
        opts.selected_channels[0].name = "Kick In".to_string();
        opts.selected_channels[0].color = 2;
        opts.selected_channels[1].name = "Snare Top".to_string();
        opts.selected_channels[1].color = 3;

        // 1. Adapt docs/X32Template.rpp (has 8 DCA tracks + audio <ITEM>s + <FXCHAIN>)
        let adapted1 = adapt_rpp(&tpl1, &opts).unwrap();
        assert!(adapted1.contains("NAME \"DCA 1 %1 %7\""));
        assert!(adapted1.contains("NAME \"Kick In %1 %2\""));
        assert!(adapted1.contains("NAME \"Snare Top %2 %3\""));
        assert!(adapted1.contains("Bass.wav"));
        assert!(adapted1.contains("<FXCHAIN"));
        assert!(adapted1.contains("RECORD_PATH \"VSC_Audio\" \"\""));
        assert!(adapted1.contains("HWOUT 1024 0 1 0 0 0 0 -1:U -1"));

        // 2. Adapt docs/Template 48 tracks.rpp (has 20 MARKERs, ReaEQ, 48 tracks, <EXTENSIONS)
        let adapted2 = adapt_rpp(&tpl2, &opts).unwrap();
        assert!(adapted2.contains("MARKER 5 30.33416666666545 AMIN"));
        assert!(adapted2.contains("NAME \"Kick In %1 %2\""));
        assert!(adapted2.contains("<EXTENSIONS"));
        assert!(adapted2.contains("MAINSEND 0 0"));
        assert!(adapted2.contains("REC 1 0 0 0 0 0 0"));
    }

    #[test]
    fn test_generate_x32_reaper_ini_roundtrips_with_x32_reaper_config() {
        let opts = ReaperIniOptions {
            x32_ip: "192.168.1.77:10023".to_string(),
            reaper_ip: "192.168.1.20".to_string(),
            reaper_send_port: 8000,
            reaper_recv_port: 9000,
            transport_on: true,
            ch_bank_on: true,
            marker_btn_on: true,
            eq_ctrl_on: true,
            master_on: true,
            trk_min: 1,
            trk_max: 32,
            aux_min: 33,
            aux_max: 40,
            fxr_min: 0,
            fxr_max: 0,
            bus_min: 41,
            bus_max: 56,
            dca_min: 0,
            dca_max: 0,
            bank_size: 16,
        };
        let ini_text = generate_x32_reaper_ini(&opts);

        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", ini_text).unwrap();

        let parsed = x32_reaper::config::Config::load(tmp.path())
            .expect("Generated .X32Reaper.ini should parse cleanly in x32_reaper");
        assert_eq!(parsed.x32_ip, "192.168.1.77");
        assert_eq!(parsed.reaper_ip, "192.168.1.20");
        assert_eq!(parsed.reaper_send_port, 8000);
        assert_eq!(parsed.reaper_recv_port, 9000);
        assert!(parsed.transport_on);
        assert!(parsed.ch_bank_on);
        assert!(parsed.eq_ctrl_on);
        assert!(parsed.master_on);
        assert_eq!(parsed.trk_min, 1);
        assert_eq!(parsed.trk_max, 32);
        assert_eq!(parsed.aux_min, 33);
        assert_eq!(parsed.aux_max, 40);
        assert_eq!(parsed.bus_min, 41);
        assert_eq!(parsed.bus_max, 56);
        assert_eq!(parsed.bank_size, 16);
    }

    #[test]
    fn test_generate_rpp_with_matrix_tracks() {
        let mut opts = VscSessionOptions::default();
        opts.include_matrix_tracks = true;
        opts.matrix_names = vec![
            "Lobby".to_string(),
            "Livestream".to_string(),
            "Sub".to_string(),
        ];
        let rpp = generate_rpp(&opts);
        assert_eq!(rpp.matches("<TRACK ").count(), 38); // 32 ch + 6 matrices
        assert!(rpp.contains("NAME \"Lobby %1 %5\""));
        assert!(rpp.contains("NAME \"Livestream %1 %5\""));
        assert!(rpp.contains("NAME \"Mtx 06 %1 %5\""));
    }

    #[test]
    fn test_source_aware_dedup_digital_twin_and_uout_preservation() {
        let mut st = MixerSnapshotState::default();
        st.channels[0].name = "Mic A".to_string();
        st.channels[0].source = 15; // Non-1:1 patch
        st.channels[0].fader = 0.75; // 0 dB -> amp 1.0
        st.channels[0].on = true;
        st.channels[0].pan = 0.0; // Hard left -> -1.0

        st.channels[1].name = "Mic A Copy".to_string();
        st.channels[1].source = 15; // Shares source 15!
        st.channels[2].name = "Spare".to_string();
        st.channels[2].source = 27;
        st.channels[2].on = false; // Muted -> MUTESOLO 1 0 0

        let mut opts = VscSessionOptions::from_mixer_state(&st);
        // Should automatically exclude unnamed generic channels when custom names exist
        assert!(!opts.include_empty_channels);
        opts.include_aux_tracks = true;
        let tracks = resolve_channel_tracks(&opts);
        // Ch 2 shares card_out 15 with Ch 1 -> deduplicated! Only Mic A (src 15) + Spare (src 27)
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].raw_name, "Mic A");
        assert_eq!(tracks[0].rec_input, 14); // 0-based input 14 = X32 source 15
        assert_eq!(tracks[0].hwout_index, 1024 + 14);
        assert_eq!(tracks[1].raw_name, "Spare");
        assert_eq!(tracks[1].rec_input, 26); // 0-based input 26 = X32 source 27

        let rpp = generate_rpp(&opts);
        assert!(rpp.contains("VOLPAN 1.000000 -1.0000 -1 -1 1"));
        assert!(rpp.contains("MUTESOLO 1 0 0")); // Muted Spare track
        assert!(rpp.contains("NAME \"Aux 01 %1 %3\""));

        // UOUT preservation test
        let mut routing = RoutingState::default();
        routing.card_blocks = [26, 17, 18, 19]; // UOUT1-8 on block 1-8
        routing.in_aux = 10; // CARD1-2
        let plan = compute_vsc_routing_plan_with_options(&routing, true, true);
        assert_eq!(plan.card_blocks[0], 26); // UOUT preserved
        assert_eq!(plan.play_aux, Some(10)); // Existing CARD1-2 aux kept
        assert!(!plan.warnings.is_empty());
    }
}
