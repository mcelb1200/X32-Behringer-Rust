use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use osc_lib::OscArg;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;
use x32_lib::{MixerClient, MixerModel};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Auto-Ringout / Smart Monitor Tuning for X32/M32", long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short = 'm', long, default_value = "X32")]
    pub model: MixerModel,

    /// Comma-separated list of bus numbers to ringout (e.g. 1,2,5)
    #[arg(short, long)]
    pub buses: String,

    /// Target level in dBFS to ringout up to (e.g. -6.0)
    #[arg(short, long, default_value_t = -6.0)]
    pub target_dbfs: f32,

    /// Maximum number of notches to apply per bus
    #[arg(short = 'n', long, default_value_t = 5)]
    pub max_notches: u8,

    /// Optional offline/emulator mode using synthetic notch frequencies instead of live /meters/4 RTA
    #[arg(long, default_value_t = false)]
    pub simulate: bool,
}

/// Returns the maximum number of mix buses supported by the given mixer model.
pub fn max_buses_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 => 16,
        MixerModel::Wing => 28,
        MixerModel::XR18 => 6,
        MixerModel::XR16 => 4,
        MixerModel::XR12 => 2,
    }
}

/// Returns the maximum number of parametric EQ bands per bus for the given mixer model.
pub fn max_eq_bands_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::Wing => 8,
        MixerModel::X32 => 6,
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
    }
}

/// Formats the bus fader path according to mixer model requirements.
pub fn get_bus_fader_path(model: MixerModel, bus: u8) -> String {
    x32_lib::model::bus_fader_path(model, bus)
}

/// Writes the bus EQ parameter path into `buf` according to mixer model requirements.
pub fn write_bus_eq_path(buf: &mut String, model: MixerModel, bus: u8, notch: usize, param: &str) {
    use std::fmt::Write;
    buf.clear();
    match model {
        MixerModel::X32 | MixerModel::Wing => {
            let _ = write!(buf, "/bus/{:02}/eq/{}/{}", bus, notch, param);
        }
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => {
            let _ = write!(buf, "/bus/{}/eq/{}/{}", bus, notch, param);
        }
    }
}

/// Formats the bus EQ parameter path according to mixer model requirements.
pub fn get_bus_eq_path(model: MixerModel, bus: u8, notch: usize, param: &str) -> String {
    let mut buf = String::with_capacity(32);
    write_bus_eq_path(&mut buf, model, bus, notch, param);
    buf
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusStatus {
    Disarmed,
    Armed,
    Active, // Currently ramping / ringing out
    Done,
}

#[derive(Debug, Clone)]
pub struct Notch {
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q: f32,
}

/// Returns the 0-based float index of a 1-based mix bus within the `/meters/1` blob
/// (channels followed by aux inputs, then mix buses).
pub fn bus_meter_index(model: MixerModel, bus_idx: u8) -> usize {
    let base = match model {
        MixerModel::X32 => 32 + 8,
        MixerModel::Wing => 40 + 8,
        MixerModel::XR18 | MixerModel::XR16 => 16 + 2,
        MixerModel::XR12 => 12 + 2,
    };
    base + bus_idx.saturating_sub(1) as usize
}

#[derive(Debug, Clone)]
pub struct BusState {
    pub bus_idx: u8,
    pub status: BusStatus,
    pub current_level_db: f32,
    pub original_level_db: f32,
    pub target_level_db: f32,
    /// Live measured bus level in dBFS from `/meters/1` (`-90.0` floor).
    pub measured_dbfs: f32,
    pub notches: Vec<Notch>,
    pub fader_path: String,
}

pub struct AppState {
    // Store buses as a sorted Vec instead of HashMap to eliminate per-frame Vec allocation and sorting in the TUI render loop.
    pub buses: Vec<BusState>,
    pub should_quit: bool,
    pub pause_ramp: bool,
}

impl AppState {
    pub fn new(args: &Args) -> Self {
        let mut buses_vec = Vec::new();
        let max_buses = max_buses_for_model(args.model);

        for part in args.buses.split(',') {
            if let Ok(ch) = part.trim().parse::<u8>() {
                if (1..=max_buses).contains(&ch) {
                    let fader_path = get_bus_fader_path(args.model, ch);
                    buses_vec.push(BusState {
                        bus_idx: ch,
                        status: BusStatus::Disarmed,
                        current_level_db: -90.0,
                        original_level_db: -90.0,
                        target_level_db: args.target_dbfs,
                        measured_dbfs: -90.0,
                        notches: Vec::new(),
                        fader_path,
                    });
                }
            }
        }

        buses_vec.sort_by_key(|b| b.bus_idx);

        Self {
            buses: buses_vec,
            should_quit: false,
            pause_ramp: false,
        }
    }
}

struct LayoutCache {
    area: Rect,
    chunks: std::rc::Rc<[Rect]>,
    bus_names: Vec<String>,
    bus_statuses: Vec<String>,
    notch_strs: Vec<String>,
    waiting_notch_strs: Vec<String>,
}

/// Detects a narrow-band resonant feedback peak from a 100-band X32 RTA blob (`/meters/4`).
///
/// Returns the center frequency in Hz of the strongest bin in `80..=12000 Hz` that exceeds
/// both the `-45 dBFS` noise floor and its local neighborhood average by at least
/// `min_prominence_db` (default `9.0 dB`).
pub fn detect_rta_feedback_peak(bins: &[f32], min_prominence_db: f32) -> Option<f32> {
    let n = bins.len();
    if n < 16 {
        return None;
    }

    let mut best_bin: Option<usize> = None;
    let mut best_prominence = min_prominence_db;

    for i in 2..(n - 2) {
        let hz = x32_lib::meters::rta_bin_to_hz(i, n);
        if !(80.0..=12_000.0).contains(&hz) {
            continue;
        }
        let peak_db = x32_lib::meters::linear_to_dbfs(bins[i]);
        if peak_db < -45.0 {
            continue;
        }
        // Local neighborhood average excluding immediate peak bin
        let local_lin = (bins[i - 2] + bins[i - 1] + bins[i + 1] + bins[i + 2]) * 0.25;
        let local_db = x32_lib::meters::linear_to_dbfs(local_lin);
        let prominence = peak_db - local_db;
        if prominence >= best_prominence {
            best_prominence = prominence;
            best_bin = Some(i);
        }
    }

    best_bin.map(|b| x32_lib::meters::rta_bin_to_hz(b, n))
}

pub async fn run(args: Args) -> Result<()> {
    if args.buses.is_empty() {
        println!("No valid buses provided. Expected format: --buses 1,2,5");
        return Ok(());
    }

    println!("Connecting to {}...", args.ip);
    let ip = x32_lib::model::resolve_target_address(&args.ip, args.model);
    let client = MixerClient::connect(&ip, true).await?;
    println!("Connected.");

    // TUI setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_state = Arc::new(Mutex::new(AppState::new(&args)));

    // Subscribe to OSC
    let mut rx = client.subscribe();

    let mut ticker = interval(Duration::from_millis(500)); // Render & ramp ticker
    let mut layout_cache = LayoutCache {
        area: Rect::default(),
        chunks: std::rc::Rc::from([]),
        bus_names: Vec::new(),
        bus_statuses: Vec::new(),
        notch_strs: Vec::new(),
        waiting_notch_strs: Vec::new(),
    };
    let mut eq_path_buf = String::with_capacity(32);
    let mut latest_rta_peak_hz: Option<f32> = None;

    loop {
        // Render
        {
            let state = app_state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            terminal.draw(|f| ui(f, &state, &mut layout_cache))?;
            if state.should_quit {
                break;
            }
        }

        // Event loop
        tokio::select! {
            // Crossterm events
            event_result = tokio::task::spawn_blocking(move || {
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    event::read().ok()
                } else {
                    None
                }
            }) => {
                if let Ok(Some(Event::Key(key))) = event_result {
                    if key.kind == KeyEventKind::Press {
                        let mut state = app_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => state.should_quit = true,
                            KeyCode::Char('p') => state.pause_ramp = !state.pause_ramp,
                            KeyCode::Char('a') => {
                                // Toggle arm all
                                let all_armed = state.buses.iter().all(|b| b.status == BusStatus::Armed || b.status == BusStatus::Active);
                                for bus in state.buses.iter_mut() {
                                    if all_armed {
                                        bus.status = BusStatus::Disarmed;
                                    } else if bus.status == BusStatus::Disarmed {
                                        bus.status = BusStatus::Armed;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            _ = ticker.tick() => {
                // Request meter updates (/meters/1 for bus levels, /meters/4 for live 100-band RTA)
                let _ = client.send_message("/meters", vec![OscArg::String("/meters/1".to_string())]).await;
                if !args.simulate {
                    let _ = client.send_message("/meters", vec![OscArg::String("/meters/4".to_string())]).await;
                }

                let mut fader_updates = Vec::new();
                {
                    let mut state = app_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    if state.pause_ramp {
                        continue;
                    }

                    // Level ramping logic
                    for bus in state.buses.iter_mut() {
                        if bus.status == BusStatus::Armed || bus.status == BusStatus::Active {
                            bus.status = BusStatus::Active;

                            if bus.current_level_db < bus.target_level_db {
                                bus.current_level_db += 1.0;
                                if bus.current_level_db > bus.target_level_db {
                                    bus.current_level_db = bus.target_level_db;
                                    bus.status = BusStatus::Done;
                                }

                                // Send fader update
                                let float_val = ((bus.current_level_db + 90.0) / 100.0).clamp(0.0, 1.0);

                                fader_updates.push((bus.fader_path.clone(), float_val));
                            }
                        }
                    }
                }

                for (path, float_val) in fader_updates {
                    // Using the pre-calculated string from fader_path to prevent per-tick string allocations.
                    let _ = client.send_message(&path, vec![OscArg::Float(float_val)]).await;
                }
            }

            msg = rx.recv() => {
                let msg = match msg {
                    Ok(m) => m,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                };

                if msg.path == "/meters/4" && !args.simulate {
                    if let Some(OscArg::Blob(data)) = msg.args.first() {
                        let bins = x32_lib::meters::parse_meter_blob(data);
                        latest_rta_peak_hz = detect_rta_feedback_peak(&bins, 9.0);
                    }
                    continue;
                }

                if msg.path == "/meters/1" {
                    if let Some(OscArg::Blob(data)) = msg.args.first() {
                        if data.len() < 4 + 16 * 4 {
                            continue;
                        }

                        struct NotchUpdate {
                            bus_idx: u8,
                            notch_idx: usize,
                            freq: f32,
                            gain: f32,
                        }

                        // Maximum 16 buses on X32/M32
                        let mut updates: [Option<NotchUpdate>; 16] = [
                            None, None, None, None, None, None, None, None,
                            None, None, None, None, None, None, None, None,
                        ];
                        let mut update_count = 0;

                        let max_bands = max_eq_bands_for_model(args.model) as usize;
                        let max_notches = (args.max_notches as usize).min(max_bands);

                        {
                            let mut state = app_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                            for bus in state.buses.iter_mut() {
                                // Read the model-accurate bus meter offset from `/meters/1`
                                // (X32: 32 ch + 8 aux = offset 40 + bus_idx - 1).
                                let meter_idx = bus_meter_index(args.model, bus.bus_idx);
                                let Some(val) = x32_lib::meters::meter_value_at(data, meter_idx) else {
                                    continue;
                                };

                                bus.measured_dbfs = x32_lib::meters::linear_to_dbfs(val).clamp(-90.0, 0.0);

                                if bus.status != BusStatus::Active || bus.notches.len() >= max_notches {
                                    continue;
                                }

                                if val > 0.00001 {
                                    let current_db = bus.measured_dbfs;
                                    // Threshold for "feedback"
                                    if current_db > -6.0 {
                                        let detected_freq = if args.simulate {
                                            Some(1000.0 + (bus.notches.len() as f32 * 500.0))
                                        } else {
                                            latest_rta_peak_hz.take()
                                        };

                                        let Some(freq) = detected_freq else {
                                            continue;
                                        };

                                        // Avoid stacking duplicate notches within ±5% of an existing notch
                                        if bus
                                            .notches
                                            .iter()
                                            .any(|n| ((n.freq_hz - freq).abs() / freq.max(1.0)) < 0.05)
                                        {
                                            continue;
                                        }

                                        // Trigger notch
                                        let notch_idx = bus.notches.len() + 1; // 1-based EQ band
                                        let gain = -6.0;
                                        let q = 15.0;

                                        bus.notches.push(Notch {
                                            freq_hz: freq,
                                            gain_db: gain,
                                            q,
                                        });

                                        if update_count < 16 {
                                            updates[update_count] = Some(NotchUpdate {
                                                bus_idx: bus.bus_idx,
                                                notch_idx,
                                                freq,
                                                gain,
                                            });
                                            update_count += 1;
                                        }

                                        // Pause ramp briefly to let notch settle
                                        bus.current_level_db -= 2.0; // Pull back slightly on feedback
                                    }
                                }
                            }
                        }

                        #[allow(clippy::needless_range_loop)]
                        for i in 0..update_count {
                            if let Some(update) = &updates[i] {
                                // Apply notch via OSC

                                // type = 3 (PEQ)
                                write_bus_eq_path(
                                    &mut eq_path_buf,
                                    args.model,
                                    update.bus_idx,
                                    update.notch_idx,
                                    "type",
                                );
                                let _ = client
                                    .send_message(&eq_path_buf, vec![OscArg::Int(3)])
                                    .await;

                                // Map freq: log scale 20Hz - 20kHz to 0.0 - 1.0 (approx)
                                write_bus_eq_path(
                                    &mut eq_path_buf,
                                    args.model,
                                    update.bus_idx,
                                    update.notch_idx,
                                    "freq",
                                );
                                let freq_float = ((update.freq.log10() - 20f32.log10())
                                    / (20000f32.log10() - 20f32.log10()))
                                .clamp(0.0, 1.0);
                                let _ = client
                                    .send_message(&eq_path_buf, vec![OscArg::Float(freq_float)])
                                    .await;

                                // Map gain: -15 to +15 is 0.0 to 1.0.  (-15 is 0.0, 0 is 0.5, +15 is 1.0)
                                write_bus_eq_path(
                                    &mut eq_path_buf,
                                    args.model,
                                    update.bus_idx,
                                    update.notch_idx,
                                    "gain",
                                );
                                let gain_float = ((update.gain + 15.0) / 30.0).clamp(0.0, 1.0);
                                let _ = client
                                    .send_message(&eq_path_buf, vec![OscArg::Float(gain_float)])
                                    .await;

                                // Map q: 10.0-0.3 mapped 0.0-1.0
                                write_bus_eq_path(
                                    &mut eq_path_buf,
                                    args.model,
                                    update.bus_idx,
                                    update.notch_idx,
                                    "q",
                                );
                                let q_float = 0.8; // Approx narrow Q
                                let _ = client
                                    .send_message(&eq_path_buf, vec![OscArg::Float(q_float)])
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }

    // TUI teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, state: &AppState, cache: &mut LayoutCache) {
    use std::fmt::Write;

    let size = f.size();
    if cache.area != size || cache.chunks.is_empty() {
        cache.area = size;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
            .split(size);
        cache.chunks = chunks;
    }
    let chunks = &*cache.chunks;

    if chunks.len() < 2 {
        return;
    }

    let mut lines = vec![Line::from(vec![Span::styled(
        "  AUTO-RINGOUT",
        Style::default().fg(Color::Yellow),
    )])];

    // Resize buffers to match dynamic state length using resize_with to ensure
    // each new String actually starts with the requested capacity, preventing per-frame allocations.
    let num_buses = state.buses.len();
    if cache.bus_names.len() < num_buses {
        cache
            .bus_names
            .resize_with(num_buses, || String::with_capacity(32));
        cache
            .bus_statuses
            .resize_with(num_buses, || String::with_capacity(32));
        cache
            .waiting_notch_strs
            .resize_with(num_buses, || String::with_capacity(32));
    }

    let total_notches = state.buses.iter().map(|b| b.notches.len()).sum();
    if cache.notch_strs.len() < total_notches {
        cache
            .notch_strs
            .resize_with(total_notches, || String::with_capacity(64));
    }

    let mut notch_idx_global = 0;

    // Clear and populate stateful string buffers using `write!` instead
    // of `format!` to completely eliminate per-frame allocations in the hot render loop.
    for (bus_idx, bus) in state.buses.iter().enumerate() {
        let status_str = match bus.status {
            BusStatus::Disarmed => "DISARMED",
            BusStatus::Armed => "ARMED",
            BusStatus::Active => "ACTIVE",
            BusStatus::Done => "DONE",
        };

        cache.bus_names[bus_idx].clear();
        write!(cache.bus_names[bus_idx], "  Bus {:02}   ", bus.bus_idx).unwrap();

        cache.bus_statuses[bus_idx].clear();
        write!(
            cache.bus_statuses[bus_idx],
            "  {}  ({:.1} dB)",
            status_str, bus.current_level_db
        )
        .unwrap();

        for (i, notch) in bus.notches.iter().enumerate() {
            cache.notch_strs[notch_idx_global].clear();
            write!(
                cache.notch_strs[notch_idx_global],
                "    Notch {}: {:.0} Hz   ({:.1} dB, Q={:.1})",
                i + 1,
                notch.freq_hz,
                notch.gain_db,
                notch.q
            )
            .unwrap();
            notch_idx_global += 1;
        }

        if bus.notches.len() < 5 && matches!(bus.status, BusStatus::Active) {
            cache.waiting_notch_strs[bus_idx].clear();
            write!(
                cache.waiting_notch_strs[bus_idx],
                "    Notch {}: — waiting —",
                bus.notches.len() + 1
            )
            .unwrap();
        }
    }

    let mut notch_idx_global = 0;
    for (bus_idx, bus) in state.buses.iter().enumerate() {
        let color = match bus.status {
            BusStatus::Disarmed => Color::DarkGray,
            BusStatus::Armed => Color::Cyan,
            BusStatus::Active => Color::Green,
            BusStatus::Done => Color::Blue,
        };

        // Live bus meter bar (-90 dBFS to 0 dBFS from /meters/1)
        let meter_len = ((bus.measured_dbfs + 90.0) / 90.0 * 10.0).clamp(0.0, 10.0) as usize;
        const METER_BARS: [&str; 11] = [
            "░░░░░░░░░░",
            "█░░░░░░░░░",
            "██░░░░░░░░",
            "███░░░░░░░",
            "████░░░░░░",
            "█████░░░░░",
            "██████░░░░",
            "███████░░░",
            "████████░░",
            "█████████░",
            "██████████",
        ];
        let meter_str = METER_BARS[meter_len];

        lines.push(Line::from(vec![
            Span::raw(cache.bus_names[bus_idx].as_str()),
            Span::styled(meter_str, Style::default().fg(color)),
            Span::styled(
                cache.bus_statuses[bus_idx].as_str(),
                Style::default().fg(color),
            ),
        ]));

        for _ in bus.notches.iter() {
            lines.push(Line::from(vec![Span::raw(
                cache.notch_strs[notch_idx_global].as_str(),
            )]));
            notch_idx_global += 1;
        }

        if bus.notches.len() < 5 && matches!(bus.status, BusStatus::Active) {
            lines.push(Line::from(vec![Span::raw(
                cache.waiting_notch_strs[bus_idx].as_str(),
            )]));
        }
    }

    let main_block =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" STATUS "));
    f.render_widget(main_block, chunks[0]);

    let footer = Paragraph::new(Line::from(vec![Span::raw(
        "  [A]rm/disarm all  [P]ause  [Q]uit",
    )]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_buses_for_model() {
        assert_eq!(max_buses_for_model(MixerModel::X32), 16);
        assert_eq!(max_buses_for_model(MixerModel::Wing), 28);
        assert_eq!(max_buses_for_model(MixerModel::XR18), 6);
        assert_eq!(max_buses_for_model(MixerModel::XR16), 4);
        assert_eq!(max_buses_for_model(MixerModel::XR12), 2);
    }

    #[test]
    fn test_max_eq_bands_for_model() {
        assert_eq!(max_eq_bands_for_model(MixerModel::Wing), 8);
        assert_eq!(max_eq_bands_for_model(MixerModel::X32), 6);
        assert_eq!(max_eq_bands_for_model(MixerModel::XR18), 4);
        assert_eq!(max_eq_bands_for_model(MixerModel::XR16), 4);
        assert_eq!(max_eq_bands_for_model(MixerModel::XR12), 4);
    }

    #[test]
    fn test_get_bus_fader_path() {
        assert_eq!(get_bus_fader_path(MixerModel::X32, 1), "/bus/01/mix/fader");
        assert_eq!(
            get_bus_fader_path(MixerModel::Wing, 28),
            "/bus/28/fdr"
        );
        assert_eq!(get_bus_fader_path(MixerModel::XR18, 5), "/bus/5/mix/fader");
        assert_eq!(get_bus_fader_path(MixerModel::XR12, 2), "/bus/2/mix/fader");
    }

    #[test]
    fn test_get_bus_eq_path() {
        assert_eq!(
            get_bus_eq_path(MixerModel::X32, 2, 1, "freq"),
            "/bus/02/eq/1/freq"
        );
        assert_eq!(
            get_bus_eq_path(MixerModel::Wing, 15, 3, "gain"),
            "/bus/15/eq/3/gain"
        );
        assert_eq!(
            get_bus_eq_path(MixerModel::XR18, 4, 2, "type"),
            "/bus/4/eq/2/type"
        );
    }

    #[test]
    fn test_app_state_bus_filtering_by_model() {
        let args_xr12 = Args {
            ip: "127.0.0.1".to_string(),
            model: MixerModel::XR12,
            buses: "1,2,3,5,12".to_string(),
            target_dbfs: -6.0,
            max_notches: 3,
            simulate: true,
        };
        let state_xr12 = AppState::new(&args_xr12);
        assert_eq!(state_xr12.buses.len(), 2);
        assert_eq!(state_xr12.buses[0].bus_idx, 1);
        assert_eq!(state_xr12.buses[1].bus_idx, 2);
        assert_eq!(state_xr12.buses[0].fader_path, "/bus/1/mix/fader");

        let args_x32 = Args {
            ip: "127.0.0.1".to_string(),
            model: MixerModel::X32,
            buses: "1,2,3,5,20".to_string(),
            target_dbfs: -6.0,
            max_notches: 3,
            simulate: true,
        };
        let state_x32 = AppState::new(&args_x32);
        assert_eq!(state_x32.buses.len(), 4);
        assert_eq!(state_x32.buses[3].bus_idx, 5);
        assert_eq!(state_x32.buses[0].fader_path, "/bus/01/mix/fader");

        let args_wing = Args {
            ip: "127.0.0.1".to_string(),
            model: MixerModel::Wing,
            buses: "1,16,20,28,30".to_string(),
            target_dbfs: -6.0,
            max_notches: 5,
            simulate: true,
        };
        let state_wing = AppState::new(&args_wing);
        assert_eq!(state_wing.buses.len(), 4);
        assert_eq!(state_wing.buses[2].bus_idx, 20);
        assert_eq!(state_wing.buses[3].bus_idx, 28);
    }
}
