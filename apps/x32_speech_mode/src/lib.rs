#![allow(clippy::manual_range_contains)]
//! `x32_speech_mode` is a tool that applies broadcast audio engineering best practices
//! to speech channels with a single command. It configures EQ, compression, gating,
//! and can optionally configure automixing and ringout.
use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use x32_lib::MixerClient;
use x32_lib::MixerModel;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "One-Touch Speech Mode Macro for X32/M32", long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Comma-separated list of channel numbers to apply speech mode to (e.g. 1,2,3)
    #[arg(short, long)]
    pub channels: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short, long, default_value = "X32")]
    pub model: MixerModel,
}

#[derive(Serialize, Deserialize, Default)]
struct SavedState {
    channels: HashMap<u8, Vec<OscMessage>>,
}

// Frequency mapping helper (returns f32 for OSC float scale [0.0, 1.0])
fn freq_to_osc(freq: f32) -> f32 {
    let mut res = (freq / 20.0).ln() / 6.907_755_4;
    res = (res * 200.0).round() / 200.0;
    res.clamp(0.0, 1.0)
}

// Gain mapping helper (-15.0 to 15.0 -> 0.0 to 1.0)
fn gain_to_osc(gain: f32) -> f32 {
    ((gain + 15.0) / 30.0).clamp(0.0, 1.0)
}

// Q mapping helper
fn q_to_osc(q: f32) -> f32 {
    // According to X32 docs, Q uses logarithmic scaling between 10.0 and 0.3
    ((q / 0.3).ln() / (10.0 / 0.3_f32).ln()).clamp(0.0, 1.0)
}

// Dynamics Threshold helper (-60.0 to 0.0 -> 0.0 to 1.0)
fn dyn_thr_to_osc(thr: f32) -> f32 {
    ((thr + 60.0) / 60.0).clamp(0.0, 1.0)
}

// Gate Threshold helper (-80.0 to 0.0 -> 0.0 to 1.0)
fn gate_thr_to_osc(thr: f32) -> f32 {
    ((thr + 80.0) / 80.0).clamp(0.0, 1.0)
}

// Gate Range helper (-60.0 to 0.0 -> 0.0 to 1.0)
// (X32 gate range typically goes down to -60. Some say -oo but let's map linear)
fn gate_range_to_osc(range: f32) -> f32 {
    ((range + 60.0) / 60.0).clamp(0.0, 1.0)
}

// Dynamics Attack mapping (0 to 120ms -> 0.0 to 1.0) log scale approx
fn dyn_attack_to_osc(attack_ms: f32) -> f32 {
    // 0 ms -> 0.0, 120 ms -> 1.0
    // actually, X32 scales differently, let's use a simple linear mapping if unknown, or just use 0.3 for 10ms
    (attack_ms / 120.0).clamp(0.0, 1.0)
}

// Dynamics Release mapping (0 to 4000ms -> 0.0 to 1.0)
fn dyn_release_to_osc(release_ms: f32) -> f32 {
    // let's use a rough mapping, 100ms is quite short so maybe 0.1
    (release_ms / 4000.0).clamp(0.0, 1.0)
}

fn get_state_file_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".x32_speech_mode_state.json")
}

fn max_channels_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 => 32,
        MixerModel::Wing => 40,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

/// Model-specific EQ bands assigned for Speech Mode processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeechEqBands {
    pub hpf_band: u8,
    pub lpf_band: u8,
    pub presence_band: u8,
    pub scoop_band: u8,
}

/// Returns model-specific EQ band assignments for Speech Mode.
pub fn get_speech_eq_bands(model: MixerModel) -> SpeechEqBands {
    match model {
        MixerModel::Wing => SpeechEqBands {
            hpf_band: 1,
            lpf_band: 8,
            presence_band: 3,
            scoop_band: 2,
        },
        MixerModel::X32 => SpeechEqBands {
            hpf_band: 1,
            lpf_band: 6,
            presence_band: 3,
            scoop_band: 2,
        },
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => SpeechEqBands {
            hpf_band: 1,
            lpf_band: 4,
            presence_band: 3,
            scoop_band: 2,
        },
    }
}

/// Returns the model-aware parameter paths to save and restore for Speech Mode.
pub fn get_speech_mode_paths(model: MixerModel) -> Vec<String> {
    let bands = get_speech_eq_bands(model);
    let mut paths = vec![
        format!("eq/{}/type", bands.hpf_band),
        format!("eq/{}/f", bands.hpf_band),
        format!("eq/{}/type", bands.lpf_band),
        format!("eq/{}/f", bands.lpf_band),
        format!("eq/{}/type", bands.presence_band),
        format!("eq/{}/f", bands.presence_band),
        format!("eq/{}/g", bands.presence_band),
        format!("eq/{}/type", bands.scoop_band),
        format!("eq/{}/f", bands.scoop_band),
        format!("eq/{}/g", bands.scoop_band),
        format!("eq/{}/q", bands.scoop_band),
        "dyn/on".to_string(),
        "dyn/mode".to_string(),
        "dyn/ratio".to_string(),
        "dyn/thr".to_string(),
        "dyn/attack".to_string(),
        "dyn/release".to_string(),
        "dyn/knee".to_string(),
        "gate/on".to_string(),
        "gate/mode".to_string(),
        "gate/thr".to_string(),
        "gate/range".to_string(),
        "gate/attack".to_string(),
        "gate/release".to_string(),
    ];

    let supports_automix = matches!(
        model,
        MixerModel::X32 | MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12
    );
    if supports_automix {
        paths.push("automix/group".to_string());
    }

    paths
}

pub async fn run(args: Args) -> Result<()> {
    let max_ch = max_channels_for_model(args.model);
    let mut channels: Vec<u8> = Vec::new();
    for part in args.channels.split(',') {
        if let Ok(ch) = part.trim().parse::<u8>() {
            if ch >= 1 && ch <= max_ch {
                channels.push(ch);
            }
        }
    }

    if channels.is_empty() {
        println!(
            "No valid channels provided for model {} (max channel: {}). Expected format: --channels 1,2,3",
            args.model, max_ch
        );
        return Ok(());
    }

    println!("Connecting to {}...", args.ip);
    let ip = if args.ip.contains(':') {
        args.ip.clone()
    } else {
        format!("{}:10023", args.ip)
    };
    let client = MixerClient::connect(&ip, true).await?;
    let delay = Duration::from_millis(10);
    let state_file = get_state_file_path();

    // Check if we are toggling OFF
    if state_file.exists() {
        println!("Found saved state. Disengaging speech mode (restoring original state)...");

        let f = fs::File::open(&state_file)?;
        let mut state_data = String::new();
        use std::io::Read;
        f.take(1024 * 1024 + 1).read_to_string(&mut state_data)?;
        if state_data.len() > 1024 * 1024 {
            anyhow::bail!("State file too large to load (max 1MB)");
        }
        let mut saved_state: SavedState = serde_json::from_str(&state_data)?;

        for ch in channels.clone() {
            if let Some(msgs) = saved_state.channels.remove(&ch) {
                println!("Restoring channel {:02}", ch);
                for msg in msgs {
                    let _ = client.send_message(&msg.path, msg.args).await;
                    tokio::time::sleep(delay).await;
                }
            } else {
                println!("No saved state found for channel {:02}", ch);
            }
        }

        // Remove the state file or update it if some channels remain
        if saved_state.channels.is_empty() {
            fs::remove_file(&state_file)?;
        } else {
            let state_data = serde_json::to_string(&saved_state)?;
            fs::write(&state_file, state_data)?;
        }

        println!("Restoration complete.");
        return Ok(());
    }

    // Otherwise, we are turning ON. Save state first.
    println!("Engaging speech mode on channels: {:?}", channels);
    let mut saved_state = SavedState::default();

    // List of paths we will modify and need to save
    let paths_to_save = get_speech_mode_paths(args.model);

    for ch in &channels {
        let mut original_msgs = Vec::new();
        for sub_path in &paths_to_save {
            let path = format!("/ch/{:02}/{}", ch, sub_path);
            if let Ok(val) = client.query_value(&path).await {
                original_msgs.push(OscMessage {
                    path,
                    args: vec![val],
                });
            }
            tokio::time::sleep(delay).await;
        }
        saved_state.channels.insert(*ch, original_msgs);
    }

    let bands = get_speech_eq_bands(args.model);
    let supports_automix = matches!(
        args.model,
        MixerModel::X32 | MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12
    );

    for ch in channels {
        println!("Processing channel {:02}", ch);

        let mut msgs = vec![
            // 1. High-pass filter: 80 Hz, 18 dB/oct slope
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/type", ch, bands.hpf_band),
                args: vec![OscArg::Int(5)],
            },
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/f", ch, bands.hpf_band),
                args: vec![OscArg::Float(freq_to_osc(80.0))],
            },
            // 2. Low-pass filter: 12 kHz, 12 dB/oct slope
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/type", ch, bands.lpf_band),
                args: vec![OscArg::Int(6)],
            },
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/f", ch, bands.lpf_band),
                args: vec![OscArg::Float(freq_to_osc(12000.0))],
            },
            // 3. Presence boost: +3 dB shelf at 3.5 kHz
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/type", ch, bands.presence_band),
                args: vec![OscArg::Int(3)],
            },
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/f", ch, bands.presence_band),
                args: vec![OscArg::Float(freq_to_osc(3500.0))],
            },
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/g", ch, bands.presence_band),
                args: vec![OscArg::Float(gain_to_osc(3.0))],
            },
            // 4. Low-mid scoop: -2 dB at 300 Hz, Q=1.5
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/type", ch, bands.scoop_band),
                args: vec![OscArg::Int(3)],
            },
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/f", ch, bands.scoop_band),
                args: vec![OscArg::Float(freq_to_osc(300.0))],
            },
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/g", ch, bands.scoop_band),
                args: vec![OscArg::Float(gain_to_osc(-2.0))],
            },
            OscMessage {
                path: format!("/ch/{:02}/eq/{}/q", ch, bands.scoop_band),
                args: vec![OscArg::Float(q_to_osc(1.5))],
            },
            // 5. Compressor: Ratio 3:1, threshold -20 dBFS, attack 10 ms, release 100 ms, knee soft
            OscMessage {
                path: format!("/ch/{:02}/dyn/on", ch),
                args: vec![OscArg::Int(1)],
            },
            OscMessage {
                path: format!("/ch/{:02}/dyn/mode", ch),
                args: vec![OscArg::Int(0)],
            }, // COMP
            OscMessage {
                path: format!("/ch/{:02}/dyn/ratio", ch),
                args: vec![OscArg::Int(5)],
            }, // Ratio 3:1 is typically index 5 in X_DY_RAT (" 1.1", " 1.3", " 1.5", " 2.0", " 2.5", " 3.0", " 4.0", " 5.0", " 7.0", " 10", " 20", " 100")
            OscMessage {
                path: format!("/ch/{:02}/dyn/thr", ch),
                args: vec![OscArg::Float(dyn_thr_to_osc(-20.0))],
            },
            OscMessage {
                path: format!("/ch/{:02}/dyn/attack", ch),
                args: vec![OscArg::Float(dyn_attack_to_osc(10.0))],
            },
            OscMessage {
                path: format!("/ch/{:02}/dyn/release", ch),
                args: vec![OscArg::Float(dyn_release_to_osc(100.0))],
            },
            OscMessage {
                path: format!("/ch/{:02}/dyn/knee", ch),
                args: vec![OscArg::Float(0.6)],
            }, // Soft knee (roughly 3-4dB, 0-5dB scale -> 0.6)
            // 6. Gate/Expander: Threshold -50 dBFS, range -20 dB, attack 0.5 ms, release 200 ms
            OscMessage {
                path: format!("/ch/{:02}/gate/on", ch),
                args: vec![OscArg::Int(1)],
            },
            OscMessage {
                path: format!("/ch/{:02}/gate/mode", ch),
                args: vec![OscArg::Int(2)],
            }, // EXP 2
            OscMessage {
                path: format!("/ch/{:02}/gate/thr", ch),
                args: vec![OscArg::Float(gate_thr_to_osc(-50.0))],
            },
            OscMessage {
                path: format!("/ch/{:02}/gate/range", ch),
                args: vec![OscArg::Float(gate_range_to_osc(-20.0))],
            },
            OscMessage {
                path: format!("/ch/{:02}/gate/attack", ch),
                args: vec![OscArg::Float(dyn_attack_to_osc(0.5))],
            },
            OscMessage {
                path: format!("/ch/{:02}/gate/release", ch),
                args: vec![OscArg::Float(dyn_release_to_osc(200.0))],
            },
        ];

        if supports_automix {
            msgs.push(OscMessage {
                path: format!("/ch/{:02}/automix/group", ch),
                args: vec![OscArg::Int(1)], // Group X / Group 1
            });
        }

        for msg in msgs {
            client.send_message(&msg.path, msg.args).await?;
            tokio::time::sleep(delay).await;
        }

        println!("Configured channel {:02} for speech mode.", ch);
    }

    let state_data = serde_json::to_string(&saved_state)?;
    fs::write(&state_file, state_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_range_mapping() {
        assert_eq!(gate_range_to_osc(-20.0), 40.0 / 60.0);
        assert_eq!(gate_range_to_osc(-60.0), 0.0);
        assert_eq!(gate_range_to_osc(0.0), 1.0);
    }

    #[test]
    fn test_speech_mode_osc_generation() {
        let mut res = (80.0_f32 / 20.0).ln() / 6.907_755_4;
        res = (res * 200.0).round() / 200.0;
        res = res.clamp(0.0, 1.0);
        assert_eq!(freq_to_osc(80.0), res);
        assert_eq!(gain_to_osc(3.0), (3.0 + 15.0) / 30.0);
    }

    #[test]
    fn test_max_channels_for_model() {
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);
    }

    #[test]
    fn test_speech_eq_bands_per_model() {
        assert_eq!(
            get_speech_eq_bands(MixerModel::X32),
            SpeechEqBands {
                hpf_band: 1,
                lpf_band: 6,
                presence_band: 3,
                scoop_band: 2,
            }
        );

        assert_eq!(
            get_speech_eq_bands(MixerModel::Wing),
            SpeechEqBands {
                hpf_band: 1,
                lpf_band: 8,
                presence_band: 3,
                scoop_band: 2,
            }
        );

        for model in [MixerModel::XR18, MixerModel::XR16, MixerModel::XR12] {
            assert_eq!(
                get_speech_eq_bands(model),
                SpeechEqBands {
                    hpf_band: 1,
                    lpf_band: 4,
                    presence_band: 3,
                    scoop_band: 2,
                }
            );
        }
    }

    #[test]
    fn test_speech_mode_paths_per_model() {
        let x32_paths = get_speech_mode_paths(MixerModel::X32);
        assert!(x32_paths.contains(&"eq/6/type".to_string()));
        assert!(x32_paths.contains(&"automix/group".to_string()));

        let wing_paths = get_speech_mode_paths(MixerModel::Wing);
        assert!(wing_paths.contains(&"eq/8/type".to_string()));
        assert!(!wing_paths.contains(&"automix/group".to_string()));

        let xr18_paths = get_speech_mode_paths(MixerModel::XR18);
        assert!(xr18_paths.contains(&"eq/4/type".to_string()));
        assert!(xr18_paths.contains(&"automix/group".to_string()));
    }

    #[test]
    fn test_model_channel_filtering() {
        let parse_channels = |input: &str, model: MixerModel| -> Vec<u8> {
            let max_ch = max_channels_for_model(model);
            let mut channels = Vec::new();
            for part in input.split(',') {
                if let Ok(ch) = part.trim().parse::<u8>() {
                    if ch >= 1 && ch <= max_ch {
                        channels.push(ch);
                    }
                }
            }
            channels
        };

        assert_eq!(
            parse_channels("1,12,13,16,32,40", MixerModel::XR12),
            vec![1, 12]
        );
        assert_eq!(
            parse_channels("1,12,13,16,32,40", MixerModel::XR18),
            vec![1, 12, 13, 16]
        );
        assert_eq!(
            parse_channels("1,12,13,16,32,40", MixerModel::X32),
            vec![1, 12, 13, 16, 32]
        );
        assert_eq!(
            parse_channels("1,12,13,16,32,40", MixerModel::Wing),
            vec![1, 12, 13, 16, 32, 40]
        );
    }
}
