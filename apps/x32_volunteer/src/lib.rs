pub mod network;
pub mod state;
pub mod ui;

use anyhow::Result;
use clap::Parser;
use network::ChannelState;
use osc_lib::OscArg;
use serde::Deserialize;
use state::AppState;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use ui::{Tui, UIEvent};
use x32_lib::automix::{eq_gain_db_to_osc, BusAutomixSpec, SpectrumAutomixer};
use x32_lib::discovery::discover_mixers;
use x32_lib::meters::{linear_to_dbfs, meter_value_at, parse_rta_snapshot};
use x32_lib::model::{
    bus_eq_freq_path, bus_eq_gain_path, bus_fader_path, limits_for_model, main_fader_path,
    main_mute_path,
};
use x32_lib::safety::{smooth_fade, ConsoleSnapshot};
use x32_lib::streaming::{StreamConfig, StreamMonitor, StreamStatus};
use x32_lib::transport::{udp::UdpTransport, MixerTransport};
use x32_lib::{MixerClient, MixerModel};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Simplified TUI Dashboard (Volunteer Mode)")]
pub struct Args {
    /// IP address or hostname of the digital mixer, or "auto" for LAN discovery
    #[arg(short, long, default_value = "auto")]
    pub ip: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short = 'M', visible_short_alias = 'm', long, default_value = "X32")]
    pub model: MixerModel,

    /// Comma-separated list of channel/DCA numbers to monitor (e.g. "ch1,ch2,dca1"). If empty, defaults to ch1..ch8.
    #[arg(short, long, default_value = "")]
    pub channels: String,

    /// Path to JSON configuration file (e.g. x32_config.json)
    #[arg(short = 'c', long)]
    pub config: Option<String>,

    /// Maximum fader limit threshold in dBFS (e.g. -3.0)
    #[arg(long)]
    pub max_fader_limit_db: Option<f32>,
}

/// Mixbus Dugan automix configuration section in `x32_config.json`.
#[derive(Debug, Deserialize, Clone)]
pub struct MixbusAutomixConfig {
    #[serde(default)]
    pub enabled_on_start: bool,
    #[serde(default)]
    pub rta_assisted: bool,
    #[serde(default = "default_max_cut")]
    pub max_cut_db: f32,
    #[serde(default = "default_max_boost")]
    pub max_boost_db: f32,
    #[serde(default = "default_noise_floor")]
    pub noise_floor: f32,
    pub buses: Vec<BusAutomixSpec>,
}

fn default_max_cut() -> f32 {
    4.0
}
fn default_max_boost() -> f32 {
    1.5
}
fn default_noise_floor() -> f32 {
    0.005
}

#[derive(Debug, Deserialize, Default)]
pub struct VolunteerConfigSection {
    pub max_fader_limit_db: Option<f32>,
    pub visible_channels: Option<Vec<u32>>,
    pub visible_dcas: Option<Vec<u32>>,
    #[serde(default)]
    pub mixbus_automix: Option<MixbusAutomixConfig>,
    #[serde(default)]
    pub stream_monitor: Option<StreamConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    pub volunteer_mode: Option<VolunteerConfigSection>,
}

/// Returns the maximum number of input channels supported by the given mixer model.
pub fn max_channels_for_model(model: MixerModel) -> u32 {
    limits_for_model(model).channels
}

/// Returns the maximum number of DCAs supported by the given mixer model.
pub fn max_dcas_for_model(model: MixerModel) -> u32 {
    limits_for_model(model).dcas
}

/// Resolves default target OSC port based on mixer model when port is omitted.
pub fn resolve_target_address(ip: &str, model: MixerModel) -> String {
    x32_lib::model::resolve_target_address(ip, model)
}

/// Converts a 0.0-1.0 float fader value to approximate dBFS (-90 dB to +10 dB curve).
pub fn fader_float_to_db(fader: f32) -> f32 {
    x32_lib::model::fader_to_db(fader)
}

/// Converts a dBFS value (-90.0 to +10.0) to a 0.0-1.0 float fader position.
pub fn db_to_fader_float(db: f32) -> f32 {
    x32_lib::model::db_to_fader(db)
}

/// Clamps a target fader float within `[0.0, 1.0]` and any configured `max_limit_db` ceiling.
pub fn clamp_fader_with_limit(fader: f32, max_limit_db: Option<f32>) -> f32 {
    x32_lib::model::clamp_fader_with_limit(fader, max_limit_db)
}

/// Computes overall dashboard health status from active alerts, panic arming, main mute, stream status, and connectivity.
pub fn compute_status(state: &AppState) -> state::Status {
    let has_clipping = state
        .active_alerts
        .iter()
        .any(|a| matches!(a, state::Alert::Clipping { .. }));
    let stream_problem = state
        .stream_telemetry
        .map(|s| matches!(s.status, StreamStatus::Clipping | StreamStatus::Silent))
        .unwrap_or(false);
    if has_clipping || !state.is_online() || stream_problem {
        return state::Status::Problem;
    }

    let has_fader_limit = state
        .active_alerts
        .iter()
        .any(|a| matches!(a, state::Alert::FaderLimit { .. }));
    let stream_caution = state
        .stream_telemetry
        .map(|s| matches!(s.status, StreamStatus::TooQuiet | StreamStatus::Hot))
        .unwrap_or(false);
    if has_fader_limit || state.is_panic_armed() || state.main_muted || stream_caution {
        return state::Status::Caution;
    }

    state::Status::Ok
}

pub fn load_config_file(path: &str) -> Result<ConfigFile> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::Read::take(file, 1024 * 1024);
    let config: ConfigFile = serde_json::from_reader(&mut reader)?;
    Ok(config)
}

/// Maps a bus/matrix/main source string (e.g. `"bus15"`, `"mtx1"`, `"main"`) to a `/meters/1` float index.
pub fn stream_source_meter_index(source: &str, model: MixerModel) -> usize {
    let limits = limits_for_model(model);
    let bus_base = (limits.channels + limits.aux_ins) as usize;
    let s = source.trim().to_ascii_lowercase();
    if let Some(num) = s.strip_prefix("bus").and_then(|n| n.parse::<usize>().ok()) {
        return bus_base + num.saturating_sub(1);
    }
    if let Some(num) = s.strip_prefix("mtx").and_then(|n| n.parse::<usize>().ok()) {
        return bus_base + limits.buses as usize + num.saturating_sub(1);
    }
    // Main LR L channel
    bus_base + limits.buses as usize + limits.matrices as usize + 1
}

/// Maps a stream source string (e.g. `"bus15"`, `"mtx1"`, `"main"`) to its fader OSC path.
pub fn stream_source_fader_path(source: &str, model: MixerModel) -> String {
    let s = source.trim().to_ascii_lowercase();
    if let Some(num) = s.strip_prefix("bus").and_then(|n| n.parse::<u8>().ok()) {
        return bus_fader_path(model, num);
    }
    if let Some(num) = s.strip_prefix("mtx").and_then(|n| n.parse::<u8>().ok()) {
        return format!("/mtx/{:02}/mix/fader", num);
    }
    main_fader_path(model).to_string()
}

fn capture_channel_snapshot(state: &AppState) -> ConsoleSnapshot {
    let mut snap = ConsoleSnapshot::new();
    for ch in &state.channels {
        snap.record_float(&ch.fader_path, ch.fader);
        snap.record_int(&ch.mute_path, if ch.muted { 0 } else { 1 });
    }
    snap
}

pub async fn run(args: Args) -> Result<()> {
    let config_file = if let Some(ref config_path) = args.config {
        Some(load_config_file(config_path)?)
    } else if std::path::Path::new("x32_config.json").exists() {
        load_config_file("x32_config.json").ok()
    } else {
        None
    };

    let volunteer_sec = config_file.as_ref().and_then(|c| c.volunteer_mode.as_ref());

    // 1. Resolve target IP & model (supporting `--ip auto` LAN discovery)
    let (target_addr, active_model) = if args.ip.eq_ignore_ascii_case("auto") {
        let found = discover_mixers(Duration::from_millis(1200))
            .await
            .unwrap_or_default();
        if let Some(first) = found.first() {
            (first.addr.to_string(), first.model)
        } else {
            (resolve_target_address("127.0.0.1", args.model), args.model)
        }
    } else {
        (resolve_target_address(&args.ip, args.model), args.model)
    };

    let max_channels = max_channels_for_model(active_model);
    let max_dcas = max_dcas_for_model(active_model);

    let udp = UdpTransport::connect(&target_addr).await?;
    let transport: Arc<dyn MixerTransport> = Arc::new(udp);
    let network = MixerClient::new(transport, true);
    let mut rx = network.subscribe();

    // 2. Parse Channel Configuration merging CLI and Config file settings
    let mut configured_channels = vec![];

    if !args.channels.is_empty() {
        for part in args.channels.split(',') {
            let part = part.trim().to_lowercase();
            if let Some(stripped) = part.strip_prefix("ch") {
                if let Ok(num) = stripped.parse::<u32>() {
                    if num >= 1 && num <= max_channels {
                        configured_channels.push(ChannelState::new(
                            format!("/ch/{:02}", num),
                            false,
                            num,
                        ));
                    }
                }
            } else if let Some(stripped) = part.strip_prefix("dca") {
                if let Ok(num) = stripped.parse::<u32>() {
                    if num >= 1 && num <= max_dcas {
                        configured_channels.push(ChannelState::new(
                            format!("/dca/{}", num),
                            true,
                            num,
                        ));
                    }
                }
            }
        }
    } else if let Some(sec) = volunteer_sec {
        if let Some(ref vis_chs) = sec.visible_channels {
            for &num in vis_chs {
                if num >= 1 && num <= max_channels {
                    configured_channels.push(ChannelState::new(
                        format!("/ch/{:02}", num),
                        false,
                        num,
                    ));
                }
            }
        }
        if let Some(ref vis_dcas) = sec.visible_dcas {
            for &num in vis_dcas {
                if num >= 1 && num <= max_dcas {
                    configured_channels.push(ChannelState::new(format!("/dca/{}", num), true, num));
                }
            }
        }
    }

    if configured_channels.is_empty() {
        let default_count = 8.min(max_channels);
        for i in 1..=default_count {
            configured_channels.push(ChannelState::new(format!("/ch/{:02}", i), false, i));
        }
    }

    let max_fader_limit = args
        .max_fader_limit_db
        .or_else(|| volunteer_sec.and_then(|s| s.max_fader_limit_db));

    let mut state = AppState::new(configured_channels);
    state.model = active_model;
    state.max_fader_limit_db = max_fader_limit;

    if let Some(sec) = volunteer_sec {
        if let Some(ref am_cfg) = sec.mixbus_automix {
            state.automix_enabled = am_cfg.enabled_on_start;
            state.rta_assisted = am_cfg.rta_assisted;
            state.automixer = Some(SpectrumAutomixer::new(
                am_cfg.buses.clone(),
                am_cfg.max_cut_db,
                am_cfg.max_boost_db,
                am_cfg.noise_floor,
            ));
        }
        if let Some(ref st_cfg) = sec.stream_monitor {
            state.stream_monitor = Some(StreamMonitor::new(st_cfg.clone()));
        }
    }

    let mut tui = Tui::new()?;
    let result = run_tui_loop(&network, &mut rx, state, &mut tui).await;
    let _ = tui.cleanup();

    result
}

async fn run_tui_loop(
    network: &MixerClient,
    rx: &mut tokio::sync::broadcast::Receiver<osc_lib::OscMessage>,
    mut state: AppState,
    tui: &mut Tui,
) -> Result<()> {
    let main_f_path = main_fader_path(state.model);
    let main_m_path = main_mute_path(state.model);
    let stream_f_path = state
        .stream_monitor
        .as_ref()
        .map(|m| stream_source_fader_path(&m.config().source, state.model));
    let stream_meter_idx = state
        .stream_monitor
        .as_ref()
        .map(|m| stream_source_meter_index(&m.config().source, state.model));
    let bus_meter_base = {
        let l = limits_for_model(state.model);
        (l.channels + l.aux_ins) as usize
    };

    // Initial state queries
    for ch in &state.channels {
        network.send_message(ch.fader_path.as_str(), vec![]).await?;
        network.send_message(ch.mute_path.as_str(), vec![]).await?;
        network.send_message(ch.name_path.as_str(), vec![]).await?;
    }
    network.send_message(main_f_path, vec![]).await?;
    network.send_message(main_m_path, vec![]).await?;
    if let Some(ref sf) = stream_f_path {
        network.send_message(sf, vec![]).await?;
    }
    network.send_message("/xremote", vec![]).await?;
    network
        .send_message("/meters", vec![OscArg::String("/meters/1".to_string())])
        .await?;

    let mut last_ui_update = Instant::now();
    let mut last_meter_req = Instant::now();
    let mut last_xremote_req = Instant::now();
    let mut last_automix_step = Instant::now();

    loop {
        while let Ok(msg) = rx.try_recv() {
            state.last_rx_at = Some(Instant::now());
            let path = msg.path;

            if path == main_f_path {
                if let Some(OscArg::Float(v)) = msg.args.first() {
                    state.main_fader = *v;
                }
                continue;
            } else if path == main_m_path {
                if let Some(OscArg::Int(v)) = msg.args.first() {
                    state.main_muted = if state.model == MixerModel::Wing {
                        *v != 0
                    } else {
                        *v == 0
                    };
                }
                continue;
            } else if stream_f_path.as_deref() == Some(path.as_str()) {
                if let Some(OscArg::Float(v)) = msg.args.first() {
                    state.stream_fader = *v;
                }
                continue;
            }

            for ch in &mut state.channels {
                if path == ch.fader_path {
                    if let Some(OscArg::Float(v)) = msg.args.first() {
                        ch.fader = *v;
                    }
                } else if path == ch.mute_path {
                    if let Some(OscArg::Int(v)) = msg.args.first() {
                        ch.muted = *v == 0;
                    }
                } else if path == ch.name_path {
                    if let Some(OscArg::String(s)) = msg.args.first() {
                        if !s.is_empty() {
                            ch.name = s.clone();
                        }
                    }
                }
            }

            if path == "/meters/1" {
                if let Some(OscArg::Blob(data)) = msg.args.first() {
                    for ch in &mut state.channels {
                        if !ch.is_dca {
                            if let Some(float_val) = meter_value_at(data, (ch.num - 1) as usize) {
                                ch.level_db = linear_to_dbfs(float_val);
                            }
                        }
                    }

                    if let (Some(idx), Some(mon)) =
                        (stream_meter_idx, state.stream_monitor.as_mut())
                    {
                        if let Some(lvl) = meter_value_at(data, idx) {
                            let tel = mon.update(lvl);
                            state.stream_telemetry = Some(tel);
                            if tel.trim_nudge_db != 0.0 {
                                if let Some(ref sf) = stream_f_path {
                                    let cur_db = fader_float_to_db(state.stream_fader);
                                    let next = clamp_fader_with_limit(
                                        db_to_fader_float(cur_db + tel.trim_nudge_db),
                                        state.max_fader_limit_db,
                                    );
                                    if (next - state.stream_fader).abs() > 0.005 {
                                        state.stream_fader = next;
                                        let _ = network
                                            .send_message(sf, vec![OscArg::Float(next)])
                                            .await;
                                    }
                                }
                            }
                        }
                    }

                    if state.automix_enabled
                        && last_automix_step.elapsed() >= Duration::from_millis(50)
                    {
                        if let Some(am) = state.automixer.as_mut() {
                            let bus_levels: Vec<f32> = am
                                .buses()
                                .iter()
                                .map(|b| {
                                    meter_value_at(
                                        data,
                                        bus_meter_base + b.bus.saturating_sub(1) as usize,
                                    )
                                    .unwrap_or(0.0)
                                })
                                .collect();
                            let step = am.step(&bus_levels, state.max_fader_limit_db);
                            for (bus, fader) in step.fader_updates {
                                let p = bus_fader_path(state.model, bus);
                                let _ = network.send_message(&p, vec![OscArg::Float(fader)]).await;
                            }
                            for (bus, band, gain_osc) in step.eq_updates {
                                let p = bus_eq_gain_path(state.model, bus, band);
                                let _ = network
                                    .send_message(&p, vec![OscArg::Float(gain_osc)])
                                    .await;
                            }
                            for (bus, band, freq_osc) in step.eq_freq_updates {
                                let p = bus_eq_freq_path(state.model, bus, band);
                                let _ = network
                                    .send_message(&p, vec![OscArg::Float(freq_osc)])
                                    .await;
                            }
                            last_automix_step = Instant::now();
                        }
                    }
                }
            } else if path == "/meters/4" {
                if let Some(OscArg::Blob(data)) = msg.args.first() {
                    if let Some(am) = state.automixer.as_mut() {
                        am.set_rta_snapshot(parse_rta_snapshot(data));
                    }
                }
            }
        }

        if last_xremote_req.elapsed() > Duration::from_secs(8) {
            network.send_message("/xremote", vec![]).await?;
            last_xremote_req = Instant::now();
        }

        if last_meter_req.elapsed() > Duration::from_millis(50) {
            network
                .send_message("/meters", vec![OscArg::String("/meters/1".to_string())])
                .await?;
            if state.model == MixerModel::X32 && state.automix_enabled && state.rta_assisted {
                network
                    .send_message("/meters", vec![OscArg::String("/meters/4".to_string())])
                    .await?;
            }
            last_meter_req = Instant::now();
        }

        if last_ui_update.elapsed() > Duration::from_millis(30) {
            state.alerts.clear();
            state.active_alerts.clear();

            for (i, ch) in state.channels.iter().enumerate() {
                if !ch.muted && ch.level_db > -5.0 && !ch.is_dca {
                    state.alerts.push(i);
                    state.active_alerts.push(state::Alert::Clipping {
                        channel_idx: i,
                        level_db: ch.level_db,
                    });
                }

                if let Some(limit) = state.max_fader_limit_db {
                    let fader_db = fader_float_to_db(ch.fader);
                    if fader_db > limit {
                        if !state.alerts.contains(&i) {
                            state.alerts.push(i);
                        }
                        state.active_alerts.push(state::Alert::FaderLimit {
                            channel_idx: i,
                            fader_db,
                            limit,
                        });
                    }
                }
            }

            state.status = compute_status(&state);
            tui.draw(&state)?;

            if let Some(event) = tui.handle_events()? {
                match event {
                    UIEvent::Quit => break,
                    UIEvent::MuteAll => {
                        state.confirm_panic_at = None;
                        state.last_snapshot = Some(capture_channel_snapshot(&state));
                        for ch in &mut state.channels {
                            ch.muted = true;
                            network
                                .send_message(&ch.mute_path, vec![OscArg::Int(0)])
                                .await?;
                        }
                    }
                    UIEvent::Panic => {
                        if state.is_panic_armed() {
                            state.confirm_panic_at = None;
                            state.last_snapshot = Some(capture_channel_snapshot(&state));
                            let fade_targets: Vec<(String, f32)> = state
                                .channels
                                .iter()
                                .map(|c| (c.fader_path.clone(), c.fader))
                                .collect();
                            let _ =
                                smooth_fade(network, &fade_targets, 5, Duration::from_millis(200))
                                    .await;
                            for ch in &mut state.channels {
                                ch.muted = true;
                                ch.fader = 0.0;
                                network
                                    .send_message(&ch.mute_path, vec![OscArg::Int(0)])
                                    .await?;
                                network
                                    .send_message(&ch.fader_path, vec![OscArg::Float(0.0)])
                                    .await?;
                            }
                        } else {
                            state.confirm_panic_at = Some(Instant::now());
                        }
                    }
                    UIEvent::Undo => {
                        state.confirm_panic_at = None;
                        if let Some(snap) = state.last_snapshot.take() {
                            snap.restore(network).await?;
                            for ch in &mut state.channels {
                                if let Some(x32_lib::safety::SnapshotValue::Float(f)) =
                                    snap.entries.get(&ch.fader_path)
                                {
                                    ch.fader = *f;
                                }
                                if let Some(x32_lib::safety::SnapshotValue::Int(on)) =
                                    snap.entries.get(&ch.mute_path)
                                {
                                    ch.muted = *on == 0;
                                }
                            }
                            state.check_status_msg =
                                Some("↩️  Restored pre-action fader & mute snapshot".to_string());
                        } else {
                            state.check_status_msg =
                                Some("ℹ️  No snapshot available to undo".to_string());
                        }
                    }
                    UIEvent::ToggleAutomix => {
                        state.confirm_panic_at = None;
                        if let Some(ref am) = state.automixer {
                            state.automix_enabled = !state.automix_enabled;
                            if !state.automix_enabled {
                                for bus in am.buses() {
                                    if let Some(ref eq) = bus.pocket_eq {
                                        let p = bus_eq_gain_path(state.model, bus.bus, eq.band);
                                        let _ = network
                                            .send_message(
                                                &p,
                                                vec![OscArg::Float(eq_gain_db_to_osc(0.0))],
                                            )
                                            .await;
                                    }
                                }
                            }
                        } else {
                            state.check_status_msg = Some(
                                "ℹ️  Add mixbus_automix to x32_config.json to enable [A]utomix"
                                    .to_string(),
                            );
                        }
                    }
                    UIEvent::PreServiceCheck => {
                        state.confirm_panic_at = None;
                        let mut snap = capture_channel_snapshot(&state);
                        snap.record_float(main_f_path, state.main_fader);
                        snap.record_int(
                            main_m_path,
                            if state.model == MixerModel::Wing {
                                i32::from(state.main_muted)
                            } else {
                                i32::from(!state.main_muted)
                            },
                        );
                        state.last_snapshot = Some(snap.clone());
                        let epoch = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let backup_name = format!("volunteer_backup_{}.json", epoch);
                        let saved_note = match snap.save_to_file(std::path::Path::new(&backup_name))
                        {
                            Ok(()) => format!("Saved {}", backup_name),
                            Err(e) => format!("In-memory only ({})", e),
                        };
                        let main_str = if state.main_muted { "MUTED" } else { "LIVE" };
                        let conn_str = if state.is_online() {
                            "ONLINE"
                        } else {
                            "OFFLINE"
                        };
                        state.check_status_msg = Some(format!(
                            "✅ Pre-Service Check: {} | Main LR: {} | {}",
                            conn_str, main_str, saved_note
                        ));
                    }
                    UIEvent::SelectPrev => {
                        state.confirm_panic_at = None;
                        if !state.channels.is_empty() {
                            state.selected_idx = state
                                .selected_idx
                                .checked_sub(1)
                                .unwrap_or(state.channels.len() - 1);
                        }
                    }
                    UIEvent::SelectNext => {
                        state.confirm_panic_at = None;
                        if !state.channels.is_empty() {
                            state.selected_idx = (state.selected_idx + 1) % state.channels.len();
                        }
                    }
                    UIEvent::ToggleSelectedMute => {
                        state.confirm_panic_at = None;
                        if let Some(ch) = state.channels.get_mut(state.selected_idx) {
                            ch.muted = !ch.muted;
                            let osc_on = if ch.muted { 0 } else { 1 };
                            network
                                .send_message(&ch.mute_path, vec![OscArg::Int(osc_on)])
                                .await?;
                        }
                    }
                    UIEvent::FaderUp => {
                        state.confirm_panic_at = None;
                        let limit_db = state.max_fader_limit_db;
                        if let Some(ch) = state.channels.get_mut(state.selected_idx) {
                            let next = clamp_fader_with_limit(ch.fader + 0.02, limit_db);
                            ch.fader = next;
                            network
                                .send_message(&ch.fader_path, vec![OscArg::Float(next)])
                                .await?;
                        }
                    }
                    UIEvent::FaderDown => {
                        state.confirm_panic_at = None;
                        let limit_db = state.max_fader_limit_db;
                        if let Some(ch) = state.channels.get_mut(state.selected_idx) {
                            let next = clamp_fader_with_limit(ch.fader - 0.02, limit_db);
                            ch.fader = next;
                            network
                                .send_message(&ch.fader_path, vec![OscArg::Float(next)])
                                .await?;
                        }
                    }
                    UIEvent::CancelConfirm => {
                        state.confirm_panic_at = None;
                    }
                }
            }

            last_ui_update = Instant::now();
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn test_args_parsing() {
        let args = Args {
            ip: "127.0.0.1".to_string(),
            model: MixerModel::X32,
            channels: "ch1,dca2".to_string(),
            config: None,
            max_fader_limit_db: Some(-3.0),
        };

        assert_eq!(args.ip, "127.0.0.1");
        assert_eq!(args.model, MixerModel::X32);
        assert_eq!(args.channels, "ch1,dca2");
        assert_eq!(args.max_fader_limit_db, Some(-3.0));
    }

    #[test]
    fn test_max_channels_and_dcas_for_model() {
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);

        assert_eq!(max_dcas_for_model(MixerModel::X32), 8);
        assert_eq!(max_dcas_for_model(MixerModel::Wing), 16);
        assert_eq!(max_dcas_for_model(MixerModel::XR18), 4);
        assert_eq!(max_dcas_for_model(MixerModel::XR16), 4);
        assert_eq!(max_dcas_for_model(MixerModel::XR12), 4);
    }

    #[test]
    fn test_resolve_target_address() {
        assert_eq!(
            resolve_target_address("192.168.1.100", MixerModel::X32),
            "192.168.1.100:10023"
        );
        assert_eq!(
            resolve_target_address("192.168.1.100", MixerModel::Wing),
            "192.168.1.100:2222"
        );
        assert_eq!(
            resolve_target_address("192.168.1.100", MixerModel::XR18),
            "192.168.1.100:10024"
        );
        assert_eq!(
            resolve_target_address("192.168.1.100:10000", MixerModel::X32),
            "192.168.1.100:10000"
        );
    }

    #[test]
    fn test_volunteer_config_with_automix_and_stream_deserialization() -> Result<()> {
        let json_data = r#"{
            "volunteer_mode": {
                "max_fader_limit_db": -2.0,
                "visible_channels": [1, 2, 3],
                "visible_dcas": [1],
                "mixbus_automix": {
                    "enabled_on_start": true,
                    "buses": [
                        { "bus": 1, "label": "Vox", "role": "vocals", "priority": 1.5, "base_db": 0.0 },
                        { "bus": 2, "label": "Drums", "role": "drums", "priority": 0.9, "base_db": -3.0, "pocket_eq": { "band": 3, "max_dip_db": 2.0 } }
                    ]
                },
                "stream_monitor": {
                    "source": "bus15",
                    "target_rms_db": -16.0,
                    "min_rms_db": -24.0,
                    "max_peak_db": -2.0,
                    "auto_trim": false
                }
            }
        }"#;

        let config: ConfigFile = serde_json::from_str(json_data)?;
        let sec = config
            .volunteer_mode
            .expect("Expected volunteer_mode section");
        let am = sec.mixbus_automix.expect("Expected mixbus_automix");
        assert!(am.enabled_on_start);
        assert_eq!(am.buses.len(), 2);
        assert_eq!(am.buses[1].pocket_eq.as_ref().unwrap().band, 3);
        let st = sec.stream_monitor.expect("Expected stream_monitor");
        assert_eq!(st.source, "bus15");
        assert!(!st.auto_trim);
        assert_eq!(stream_source_meter_index("bus15", MixerModel::X32), 54);
        assert_eq!(
            stream_source_fader_path("bus15", MixerModel::X32),
            "/bus/15/mix/fader"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_connection() -> Result<()> {
        let server = UdpSocket::bind("127.0.0.1:0").await?;
        let addr = server.local_addr()?;
        let transport = x32_lib::transport::udp::UdpTransport::connect(&addr.to_string()).await?;
        let _network = MixerClient::new(Arc::new(transport), false);
        Ok(())
    }

    #[test]
    fn test_db_to_fader_roundtrip_and_clamp() {
        assert!((db_to_fader_float(0.0) - 0.75).abs() < 0.001);
        assert!((db_to_fader_float(-10.0) - 0.50).abs() < 0.001);
        assert!((db_to_fader_float(-30.0) - 0.25).abs() < 0.001);
        assert_eq!(db_to_fader_float(-90.0), 0.0);
        assert_eq!(db_to_fader_float(10.0), 1.0);

        let clamped = clamp_fader_with_limit(0.95, Some(-3.0));
        assert!((clamped - db_to_fader_float(-3.0)).abs() < 0.001);
        assert_eq!(clamp_fader_with_limit(-0.2, None), 0.0);
    }

    #[test]
    fn test_compute_status_transitions() {
        let ch = ChannelState::new("/ch/01".to_string(), false, 1);
        let mut state = AppState::new(vec![ch]);
        assert_eq!(compute_status(&state), state::Status::Ok);

        state.main_muted = true;
        assert_eq!(compute_status(&state), state::Status::Caution);

        state.main_muted = false;
        state.active_alerts.push(state::Alert::FaderLimit {
            channel_idx: 0,
            fader_db: 2.0,
            limit: 0.0,
        });
        assert_eq!(compute_status(&state), state::Status::Caution);

        state.active_alerts.push(state::Alert::Clipping {
            channel_idx: 0,
            level_db: -2.0,
        });
        assert_eq!(compute_status(&state), state::Status::Problem);
    }
}
