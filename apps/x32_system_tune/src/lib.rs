use anyhow::Result;
use clap::Parser;
use osc_lib::OscArg;
use std::time::Instant;
use tokio::time::Duration;
use x32_lib::automix::eq_gain_db_to_osc;
use x32_lib::discovery::discover_mixers;
use x32_lib::meters::parse_meter_blob;
use x32_lib::model::{eq_freq_hz_to_osc, resolve_target_address};
use x32_lib::room_tune::{
    average_rta_captures, find_mic_profile, list_mic_profiles, suggest_cut_only_peq,
    MicModelProfile, PeqCutSuggestion,
};
use x32_lib::safety::ConsoleSnapshot;
use x32_lib::{MixerClient, MixerModel};

pub mod ui;
use ui::{AppState, Tui, UIEvent};

/// Strict EBU R128 limit for oscillator level (-18 dBFS digital headroom limit)
pub const MAX_EBU_R128_LEVEL: f32 = 0.25;

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Automated System Tuning (Oscillator & What-We-Have Mic RTA)",
    long_about = "Guided oscillator testing and cut-only room calibration using common stage/venue microphones."
)]
pub struct Args {
    /// IP address of the console, or "auto" for LAN discovery
    #[arg(short, long, default_value = "auto")]
    pub ip: String,

    /// Mixer model (X32, Wing, XR18, XR16, XR12)
    #[arg(short = 'M', visible_short_alias = 'm', long, default_value = "X32")]
    pub model: MixerModel,

    /// Max level as a float 0.0 to 0.25 (capped at -18 dBFS for EBU R128 compliance)
    #[arg(long, default_value_t = 0.25)]
    pub max_level: f32,

    /// Interval in milliseconds for ramping volume
    #[arg(short = 'r', long, default_value_t = 250)]
    pub ramp_interval_ms: u64,

    /// Built-in microphone profile ID (e.g. sm58, e835, c747, m5, c5, ecm8000)
    #[arg(long)]
    pub mic_profile: Option<String>,

    /// Input channel (1-based) where the measurement microphone is plugged in
    #[arg(long, default_value_t = 1)]
    pub mic_channel: u8,

    /// Optional override for maximum automated cut depth in dB (1.0 to 9.0)
    #[arg(long)]
    pub max_cut_db: Option<f32>,

    /// List all built-in microphone compensation profiles and exit
    #[arg(long, default_value_t = false)]
    pub list_mics: bool,
}

impl Args {
    /// Returns the oscillator level clamped to the strict EBU R128 safety limit (-18 dBFS / 0.25 float).
    pub fn safe_level(&self) -> f32 {
        self.max_level.clamp(0.0, MAX_EBU_R128_LEVEL)
    }
}

/// Represents an output destination for Guided Output Verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputDest {
    pub name: &'static str,
    pub path: &'static str,
    pub dest_index: i32,
}

/// Returns model-specific main output destinations for Guided Output Verification.
pub fn get_main_outputs(model: MixerModel) -> Vec<OutputDest> {
    match model {
        MixerModel::X32 => vec![
            OutputDest {
                name: "Main L",
                path: "/config/osc/dest",
                dest_index: 23,
            },
            OutputDest {
                name: "Main R",
                path: "/config/osc/dest",
                dest_index: 24,
            },
            OutputDest {
                name: "Main C/M",
                path: "/config/osc/dest",
                dest_index: 25,
            },
        ],
        MixerModel::Wing => vec![
            OutputDest {
                name: "Main L",
                path: "/config/osc/dest",
                dest_index: 1,
            },
            OutputDest {
                name: "Main R",
                path: "/config/osc/dest",
                dest_index: 2,
            },
        ],
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => vec![OutputDest {
            name: "Main L/R",
            path: "/config/osc/dest",
            dest_index: 1,
        }],
    }
}

/// Returns the maximum number of mix/aux buses supported by the mixer model.
pub fn get_max_buses(model: MixerModel) -> u8 {
    x32_lib::model::limits_for_model(model).buses as u8
}

/// Formats the OSC path for the main stereo EQ band.
pub fn get_main_eq_path(model: MixerModel, band: u8, param: &str) -> String {
    match model {
        MixerModel::X32 => format!("/main/st/eq/{}/{}", band, param),
        MixerModel::Wing => format!("/main/1/eq/{}/{}", band, param),
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => {
            format!("/main/b/eq/{}/{}", band, param)
        }
    }
}

/// Formats the OSC path for a bus EQ band.
pub fn get_bus_eq_path(model: MixerModel, bus_idx: u8, band: u8, param: &str) -> String {
    match model {
        MixerModel::X32 | MixerModel::Wing => format!("/bus/{:02}/eq/{}/{}", bus_idx, band, param),
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => {
            format!("/bus/{}/eq/{}/{}", bus_idx, band, param)
        }
    }
}

/// Prints the built-in microphone compensation library table to stdout.
pub fn print_mic_profiles() {
    println!(
        "{:<10} {:<32} {:<16} {:<8} {:<5} {}",
        "ID", "Model", "Valid Band", "MaxCut", "+48V", "Placement Hint"
    );
    println!("{}", "-".repeat(100));
    for m in list_mic_profiles() {
        println!(
            "{:<10} {:<32} {:>3.0}-{:<5.0} Hz     -{:<4.1}dB {:<5} {}",
            m.id,
            m.display_name,
            m.valid_min_hz,
            m.valid_max_hz,
            m.max_auto_cut_db,
            if m.requires_phantom_48v { "Yes" } else { "No" },
            m.placement_hint
        );
    }
}

fn format_mic_summary(profile: &MicModelProfile, max_cut_override: Option<f32>) -> String {
    let cut = max_cut_override.unwrap_or(profile.max_auto_cut_db);
    let p48 = if profile.requires_phantom_48v {
        " | +48V REQ"
    } else {
        ""
    };
    format!(
        "{} [{}] ({:.0}-{:.0} Hz, max -{:.1} dB{})",
        profile.display_name, profile.id, profile.valid_min_hz, profile.valid_max_hz, cut, p48
    )
}

enum Phase {
    MicSelect(usize),
    GuidedOutputVerification(usize),
    AssistedGainStaging,
    RoomTuning,
    MonitorRinging,
    Done,
}

async fn apply_peq_suggestions(
    client: &MixerClient,
    model: MixerModel,
    cuts: &[PeqCutSuggestion],
    bypassed: bool,
) {
    for (idx, cut) in cuts.iter().enumerate() {
        let band = (idx + 1) as u8;
        let g = if bypassed {
            eq_gain_db_to_osc(0.0)
        } else {
            cut.osc_gain
        };
        let _ = client
            .send_message(
                &get_main_eq_path(model, band, "f"),
                vec![OscArg::Float(cut.osc_freq)],
            )
            .await;
        let _ = client
            .send_message(
                &get_main_eq_path(model, band, "q"),
                vec![OscArg::Float(cut.osc_q)],
            )
            .await;
        let _ = client
            .send_message(&get_main_eq_path(model, band, "g"), vec![OscArg::Float(g)])
            .await;
    }
}

pub async fn run(args: Args) -> Result<()> {
    if args.list_mics {
        print_mic_profiles();
        return Ok(());
    }

    let profiles = list_mic_profiles();
    let initial_mic_idx = args
        .mic_profile
        .as_deref()
        .and_then(|id| {
            find_mic_profile(id).and_then(|found| profiles.iter().position(|p| p.id == found.id))
        })
        .unwrap_or(0);
    let mut selected_mic_idx = initial_mic_idx;
    let mut max_cut_override = args.max_cut_db.map(|v| v.abs().clamp(1.0, 9.0));

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

    let client = MixerClient::connect(&target_addr, true).await?;
    let mut rx = client.subscribe();
    let mut tui = Tui::new()?;

    let safe_level = args.safe_level();
    let main_outputs = get_main_outputs(active_model);

    // Capture initial Main PEQ state (bands 1..=4) for [U]ndo safety
    let mut pre_tune_snapshot = ConsoleSnapshot::new();
    for band in 1..=4u8 {
        let g_path = get_main_eq_path(active_model, band, "g");
        let f_path = get_main_eq_path(active_model, band, "f");
        if let Ok(OscArg::Float(g)) = client.query_value(&g_path).await {
            pre_tune_snapshot.record_float(g_path, g);
        } else {
            pre_tune_snapshot.record_float(g_path, eq_gain_db_to_osc(0.0));
        }
        if let Ok(OscArg::Float(f)) = client.query_value(&f_path).await {
            pre_tune_snapshot.record_float(f_path, f);
        } else {
            pre_tune_snapshot.record_float(f_path, eq_freq_hz_to_osc(1000.0));
        }
    }

    let mut current_phase = if args.mic_profile.is_some() {
        Phase::GuidedOutputVerification(0)
    } else {
        Phase::MicSelect(selected_mic_idx)
    };

    let mut current_phase_str = if args.mic_profile.is_some() {
        "Phase 1: Guided Output Verification"
    } else {
        "Step 0: Select 'What We Have' Microphone"
    };
    let mut current_step = if args.mic_profile.is_some() {
        format!("Testing {}", main_outputs[0].name)
    } else {
        profiles[selected_mic_idx].placement_hint.to_string()
    };
    let mut status_message = if args.mic_profile.is_some() {
        "Press 'y' if you hear pink noise, 'n' if not.".to_string()
    } else {
        "Use [↑/↓] to choose your mic, [ / ] to adjust MaxCut guardrail, and [Enter] to start."
            .to_string()
    };
    let mut current_level = 0.0;

    if args.mic_profile.is_some() {
        let _ = client
            .send_message("/config/osc/type", vec![OscArg::Int(1)])
            .await;
        let _ = client
            .send_message(
                main_outputs[0].path,
                vec![OscArg::Int(main_outputs[0].dest_index)],
            )
            .await;
        let _ = client
            .send_message("/config/osc/level", vec![OscArg::Float(safe_level)])
            .await;
        let _ = client
            .send_message("/config/osc", vec![OscArg::Int(1)])
            .await;
        current_level = safe_level;
    }

    let mut is_active = true;
    let mut last_ui_update = Instant::now();
    let mut last_rta_req = Instant::now();
    let mut latest_rta_bins: Vec<f32> = Vec::new();
    let mut captured_positions: Vec<Vec<f32>> = Vec::new();
    let mut pending_sample = false;
    let mut applied_cuts: Vec<PeqCutSuggestion> = Vec::new();
    let mut is_bypassed = false;

    while is_active {
        while let Ok(msg) = rx.try_recv() {
            if msg.path == "/meters/4" {
                if let Some(OscArg::Blob(data)) = msg.args.first() {
                    let bins = parse_meter_blob(data);
                    if !bins.is_empty() {
                        latest_rta_bins = bins;
                        if pending_sample {
                            pending_sample = false;
                            captured_positions.push(latest_rta_bins.clone());
                            status_message = format!(
                                "Captured seat sample {}/5! Move to next seat and press [Space], or [y] to compute cut-only EQ.",
                                captured_positions.len()
                            );
                        }
                    }
                }
            }
        }

        if matches!(current_phase, Phase::RoomTuning)
            && active_model == MixerModel::X32
            && last_rta_req.elapsed() > Duration::from_millis(80)
        {
            let _ = client
                .send_message("/meters", vec![OscArg::String("/meters/4".to_string())])
                .await;
            last_rta_req = Instant::now();
        }

        if last_ui_update.elapsed() > Duration::from_millis(50) {
            let mic_sum = format_mic_summary(&profiles[selected_mic_idx], max_cut_override);
            let state = AppState {
                current_phase: current_phase_str,
                current_step: current_step.as_str(),
                mic_summary: mic_sum.as_str(),
                level: current_level,
                status_message: status_message.as_str(),
            };
            tui.draw(&state)?;
            last_ui_update = Instant::now();
        }

        if let Some(event) = tui.handle_events()? {
            match event {
                UIEvent::Quit => {
                    is_active = false;
                }
                UIEvent::Up => {
                    if let Phase::MicSelect(ref mut idx) = current_phase {
                        *idx = idx.checked_sub(1).unwrap_or(profiles.len() - 1);
                        selected_mic_idx = *idx;
                        current_step = profiles[selected_mic_idx].placement_hint.to_string();
                    }
                }
                UIEvent::Down => {
                    if let Phase::MicSelect(ref mut idx) = current_phase {
                        *idx = (*idx + 1) % profiles.len();
                        selected_mic_idx = *idx;
                        current_step = profiles[selected_mic_idx].placement_hint.to_string();
                    }
                }
                UIEvent::TightenGuardrail => {
                    let base =
                        max_cut_override.unwrap_or(profiles[selected_mic_idx].max_auto_cut_db);
                    max_cut_override = Some((base - 0.5).clamp(1.0, 9.0));
                }
                UIEvent::RelaxGuardrail => {
                    let base =
                        max_cut_override.unwrap_or(profiles[selected_mic_idx].max_auto_cut_db);
                    max_cut_override = Some((base + 0.5).clamp(1.0, 9.0));
                }
                UIEvent::Sample => {
                    if matches!(current_phase, Phase::RoomTuning) {
                        if captured_positions.len() < 5 {
                            if !latest_rta_bins.is_empty() {
                                captured_positions.push(latest_rta_bins.clone());
                                status_message = format!(
                                    "Captured seat sample {}/5! Press [Space] at next seat or [y] to compute cut-only EQ.",
                                    captured_positions.len()
                                );
                            } else {
                                pending_sample = true;
                                status_message = "Waiting for /meters/4 RTA frame...".to_string();
                            }
                        } else {
                            status_message =
                                "5/5 seat samples captured! Press [y] to compute & apply cut-only EQ."
                                    .to_string();
                        }
                    }
                }
                UIEvent::Bypass => {
                    if !applied_cuts.is_empty() {
                        is_bypassed = true;
                        apply_peq_suggestions(&client, active_model, &applied_cuts, true).await;
                        status_message = "A/B Mode: [B]YPASS (Flat) active — press [t] for Tuned or [u] to Undo.".to_string();
                    }
                }
                UIEvent::Tuned => {
                    if !applied_cuts.is_empty() {
                        is_bypassed = false;
                        apply_peq_suggestions(&client, active_model, &applied_cuts, false).await;
                        status_message = "A/B Mode: [T]UNED (Cut-Only EQ) active — press [b] for Bypass or [u] to Undo.".to_string();
                    }
                }
                UIEvent::Undo => {
                    let _ = pre_tune_snapshot.restore(&client).await;
                    applied_cuts.clear();
                    is_bypassed = false;
                    status_message = "↩️  Restored pre-tune Main PEQ snapshot safely.".to_string();
                }
                UIEvent::Next => match current_phase {
                    Phase::MicSelect(_) => {
                        current_phase = Phase::GuidedOutputVerification(0);
                        current_phase_str = "Phase 1: Guided Output Verification";
                        current_step = format!("Testing {}", main_outputs[0].name);
                        status_message =
                            "Press 'y' if you hear pink noise, 'n' if not.".to_string();

                        // Safety: ensure measurement mic channel is unrouted from Main LR
                        let mic_st_path = format!("/ch/{:02}/mix/st", args.mic_channel);
                        let _ = client
                            .send_message(&mic_st_path, vec![OscArg::Int(0)])
                            .await;
                        let _ = client
                            .send_message("/config/osc/type", vec![OscArg::Int(1)])
                            .await;
                        let _ = client
                            .send_message(
                                main_outputs[0].path,
                                vec![OscArg::Int(main_outputs[0].dest_index)],
                            )
                            .await;
                        let _ = client
                            .send_message("/config/osc/level", vec![OscArg::Float(safe_level)])
                            .await;
                        let _ = client
                            .send_message("/config/osc", vec![OscArg::Int(1)])
                            .await;
                        current_level = safe_level;
                    }
                    Phase::GuidedOutputVerification(_) => {
                        current_phase = Phase::AssistedGainStaging;
                        current_phase_str = "Phase 2: Assisted Gain Staging";
                        current_step = "Playing reference tone".to_string();
                        status_message =
                            "Adjust amplifier knobs comfortably. Press [Enter] for Room Tuning."
                                .to_string();
                        let _ = client
                            .send_message("/config/osc/type", vec![OscArg::Int(0)])
                            .await;
                        let _ = client
                            .send_message(
                                main_outputs[0].path,
                                vec![OscArg::Int(main_outputs[0].dest_index)],
                            )
                            .await;
                        let _ = client
                            .send_message("/config/osc/level", vec![OscArg::Float(safe_level)])
                            .await;
                        let _ = client
                            .send_message("/config/osc", vec![OscArg::Int(1)])
                            .await;
                    }
                    Phase::AssistedGainStaging => {
                        current_phase = Phase::RoomTuning;
                        current_phase_str = "Phase 3: Guided Mic Room Tuning (Pink Noise)";
                        current_step = profiles[selected_mic_idx].placement_hint.to_string();
                        status_message =
                            "Walk 3-5 seats pressing [Space] to sample RTA, then [y] to apply cut-only EQ (or [Enter] to skip)."
                                .to_string();
                        let _ = client
                            .send_message("/config/osc/type", vec![OscArg::Int(1)])
                            .await;
                        let _ = client
                            .send_message(
                                main_outputs[0].path,
                                vec![OscArg::Int(main_outputs[0].dest_index)],
                            )
                            .await;
                        let _ = client
                            .send_message("/config/osc/level", vec![OscArg::Float(safe_level)])
                            .await;
                        let _ = client
                            .send_message("/config/osc", vec![OscArg::Int(1)])
                            .await;
                        let _ = client
                            .send_message("/meters", vec![OscArg::String("/meters/4".to_string())])
                            .await;
                    }
                    Phase::RoomTuning => {
                        current_phase = Phase::MonitorRinging;
                        current_phase_str = "Phase 4: Monitor Ringing";
                        current_step = "Ringing Monitor Bus 1".to_string();
                        status_message = "Press [Enter] to finish.".to_string();
                        let _ = client
                            .send_message("/config/osc/type", vec![OscArg::Int(0)])
                            .await;
                        let _ = client
                            .send_message("/config/osc/dest", vec![OscArg::Int(1)])
                            .await;
                        let _ = client
                            .send_message("/config/osc/level", vec![OscArg::Float(safe_level)])
                            .await;
                        let _ = client
                            .send_message("/config/osc", vec![OscArg::Int(1)])
                            .await;
                    }
                    Phase::MonitorRinging => {
                        current_phase = Phase::Done;
                        current_phase_str = "Finished";
                        current_step = "All phases complete.".to_string();
                        status_message =
                            "Press [b]/[t] to A/B EQ, [u] to Undo, or [q] to exit.".to_string();
                        let _ = client
                            .send_message("/config/osc", vec![OscArg::Int(0)])
                            .await;
                        current_level = 0.0;
                    }
                    Phase::Done => {
                        is_active = false;
                    }
                },
                UIEvent::Yes | UIEvent::No => match current_phase {
                    Phase::GuidedOutputVerification(idx) => {
                        let next_idx = idx + 1;
                        if next_idx < main_outputs.len() {
                            current_phase = Phase::GuidedOutputVerification(next_idx);
                            current_step = format!("Testing {}", main_outputs[next_idx].name);
                            let _ = client
                                .send_message(
                                    main_outputs[next_idx].path,
                                    vec![OscArg::Int(main_outputs[next_idx].dest_index)],
                                )
                                .await;
                        } else {
                            current_phase = Phase::AssistedGainStaging;
                            current_phase_str = "Phase 2: Assisted Gain Staging";
                            current_step = "Playing reference tone".to_string();
                            status_message =
                                "Adjust amplifier knobs comfortably. Press [Enter] for Room Tuning."
                                    .to_string();
                            let _ = client
                                .send_message("/config/osc/type", vec![OscArg::Int(0)])
                                .await;
                            let _ = client
                                .send_message(
                                    main_outputs[0].path,
                                    vec![OscArg::Int(main_outputs[0].dest_index)],
                                )
                                .await;
                            let _ = client
                                .send_message("/config/osc/level", vec![OscArg::Float(safe_level)])
                                .await;
                            let _ = client
                                .send_message("/config/osc", vec![OscArg::Int(1)])
                                .await;
                        }
                    }
                    Phase::RoomTuning if matches!(event, UIEvent::Yes) => {
                        let avg = average_rta_captures(&captured_positions);
                        let cuts = suggest_cut_only_peq(
                            &avg,
                            &profiles[selected_mic_idx],
                            4,
                            max_cut_override,
                        );
                        if cuts.is_empty() {
                            status_message = if captured_positions.is_empty() {
                                "No seat samples yet — press [Space] to sample /meters/4 first."
                                    .to_string()
                            } else {
                                "Room response looks balanced within mic guardrails — no cuts needed!"
                                    .to_string()
                            };
                        } else {
                            apply_peq_suggestions(&client, active_model, &cuts, false).await;
                            let summary: Vec<String> = cuts
                                .iter()
                                .map(|c| format!("{:.0}Hz({:.1}dB)", c.freq_hz, c.gain_db))
                                .collect();
                            applied_cuts = cuts;
                            is_bypassed = false;
                            let _ = client
                                .send_message("/config/osc", vec![OscArg::Int(0)])
                                .await;
                            current_level = 0.0;
                            status_message = format!(
                                "Applied cut-only EQ: {} | [b] Bypass, [t] Tuned, [u] Undo, [Enter] Next",
                                summary.join(", ")
                            );
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    let _ = client
        .send_message("/config/osc/level", vec![OscArg::Float(0.0)])
        .await;
    let _ = client
        .send_message("/config/osc", vec![OscArg::Int(0)])
        .await;
    let _ = is_bypassed;
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::net::UdpSocket;
    use x32_core::Mixer;

    #[test]
    fn test_ebu_r128_safety_level_clamping() {
        let args_over = Args {
            ip: "127.0.0.1:0".to_string(),
            model: MixerModel::X32,
            max_level: 0.8,
            ramp_interval_ms: 10,
            mic_profile: None,
            mic_channel: 1,
            max_cut_db: None,
            list_mics: false,
        };
        assert_eq!(args_over.safe_level(), MAX_EBU_R128_LEVEL);

        let args_safe = Args {
            ip: "127.0.0.1:0".to_string(),
            model: MixerModel::X32,
            max_level: 0.15,
            ramp_interval_ms: 10,
            mic_profile: None,
            mic_channel: 1,
            max_cut_db: None,
            list_mics: false,
        };
        assert_eq!(args_safe.safe_level(), 0.15);

        let args_under = Args {
            ip: "127.0.0.1:0".to_string(),
            model: MixerModel::X32,
            max_level: -0.1,
            ramp_interval_ms: 10,
            mic_profile: None,
            mic_channel: 1,
            max_cut_db: None,
            list_mics: false,
        };
        assert_eq!(args_under.safe_level(), 0.0);
    }

    #[test]
    fn test_cli_mic_profile_flags() {
        let args_m5 = Args::parse_from(&[
            "x32_system_tune",
            "--ip",
            "127.0.0.1",
            "--mic-profile",
            "m5",
            "--mic-channel",
            "3",
            "--max-cut-db",
            "4.5",
        ]);
        assert_eq!(args_m5.mic_profile.as_deref(), Some("m5"));
        assert_eq!(args_m5.mic_channel, 3);
        assert_eq!(args_m5.max_cut_db, Some(4.5));

        let args_list = Args::parse_from(&["x32_system_tune", "--list-mics"]);
        assert!(args_list.list_mics);
        assert_eq!(args_list.ip, "auto");
    }

    #[test]
    fn test_main_outputs_per_model() {
        let x32_outs = get_main_outputs(MixerModel::X32);
        assert_eq!(x32_outs.len(), 3);
        assert_eq!(x32_outs[0].dest_index, 23);
        assert_eq!(x32_outs[1].dest_index, 24);
        assert_eq!(x32_outs[2].dest_index, 25);

        let wing_outs = get_main_outputs(MixerModel::Wing);
        assert_eq!(wing_outs.len(), 2);
        assert_eq!(wing_outs[0].dest_index, 1);
        assert_eq!(wing_outs[1].dest_index, 2);

        let xr18_outs = get_main_outputs(MixerModel::XR18);
        assert_eq!(xr18_outs.len(), 1);
        assert_eq!(xr18_outs[0].dest_index, 1);
    }

    #[test]
    fn test_max_buses_per_model() {
        assert_eq!(get_max_buses(MixerModel::X32), 16);
        assert_eq!(get_max_buses(MixerModel::Wing), 28);
        assert_eq!(get_max_buses(MixerModel::XR18), 6);
        assert_eq!(get_max_buses(MixerModel::XR16), 4);
        assert_eq!(get_max_buses(MixerModel::XR12), 2);
    }

    #[test]
    fn test_eq_paths_per_model() {
        assert_eq!(
            get_main_eq_path(MixerModel::X32, 1, "freq"),
            "/main/st/eq/1/freq"
        );
        assert_eq!(
            get_main_eq_path(MixerModel::Wing, 2, "gain"),
            "/main/1/eq/2/gain"
        );
        assert_eq!(get_main_eq_path(MixerModel::XR18, 3, "q"), "/main/b/eq/3/q");

        assert_eq!(
            get_bus_eq_path(MixerModel::X32, 5, 2, "freq"),
            "/bus/05/eq/2/freq"
        );
        assert_eq!(
            get_bus_eq_path(MixerModel::Wing, 12, 1, "gain"),
            "/bus/12/eq/1/gain"
        );
        assert_eq!(
            get_bus_eq_path(MixerModel::XR18, 4, 3, "q"),
            "/bus/4/eq/3/q"
        );
    }

    #[tokio::test]
    async fn test_x32_system_tune_args() {
        let args = Args {
            ip: "127.0.0.1:0".to_string(),
            model: MixerModel::Wing,
            max_level: 0.2,
            ramp_interval_ms: 10,
            mic_profile: Some("c5".to_string()),
            mic_channel: 2,
            max_cut_db: Some(5.0),
            list_mics: false,
        };
        assert_eq!(args.max_level, 0.2);
        assert_eq!(args.safe_level(), 0.2);
        assert_eq!(args.model, MixerModel::Wing);
        assert_eq!(args.mic_profile.as_deref(), Some("c5"));
    }

    #[tokio::test]
    async fn test_x32_system_tune_integration() {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = socket.local_addr().unwrap().port();
        let addr = format!("127.0.0.1:{}", port);

        let mut mixer = Mixer::new();
        let socket_arc = Arc::new(socket);
        let socket_rx = socket_arc.clone();

        let _ = tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while let Ok((len, src)) = socket_rx.recv_from(&mut buf).await {
                let responses_opt = mixer.dispatch(&buf[..len], src).ok();
                if let Some(responses) = responses_opt {
                    for (addr, response_bytes) in responses {
                        let _ = socket_rx.send_to(&response_bytes, addr).await;
                    }
                }
            }
        });

        let client = MixerClient::connect(&addr, true).await.unwrap();
        let _ = client
            .send_message("/config/osc/type", vec![OscArg::Int(1)])
            .await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        if let Ok(OscArg::Int(val)) = client.query_value("/config/osc/type").await {
            assert_eq!(val, 1);
        } else {
            panic!("Failed to verify OSC");
        }
    }
}
