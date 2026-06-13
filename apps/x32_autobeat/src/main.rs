//! `x32_autobeat` is a command-line tool for automatic beat detection and synchronization.
//!
//! It listens to an audio source (via system audio or OSC meter monitoring), detects the
//! beat (BPM), and then automatically adjusts the time parameters of an effect in a
//! specific slot on the X32 mixer. This allows delays and other time-based effects to
//! stay in sync with the music automatically.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use crate::audio::AudioEngine;
use crate::compressor::CompressorHandler;
use crate::detection::{BeatDetector, EnergyDetector, OscLevelDetector, SpectralFluxDetector};
use crate::effects::{get_handler, EffectConfig, EffectHandler};
use crate::network::{NetworkEvent, NetworkManager, Source};
use crate::ui::{AppState, Tui, UIEvent};
use anyhow::Result;
use clap::{Parser, Subcommand};
use crossbeam_channel::unbounded;
use std::sync::Arc;
use std::time::{Duration, Instant};

mod audio;
mod compressor;
mod detection;
mod effects;
mod musical_theory;
mod network;
mod scaling;
mod ui;

/// Command-line arguments for the `x32_autobeat` tool.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// X32 IP Address.
    #[arg(long, default_value = "192.168.1.50")]
    ip: String,

    /// Audio Device Name (Substring match).
    #[arg(long)]
    device: Option<String>,

    /// Audio source for beat detection (e.g. "ch1", "bus8", "aux2", "main", or "1")
    #[arg(long, default_value = "ch1")]
    channel: String,

    /// Target Effect Slot(s) (1-8, e.g. "1", "1,2", "1-4").
    #[arg(long, default_value = "1")]
    slot: String,

    /// OSC Path segment for Panic Button (substring match)
    #[arg(long, default_value = "A/btn/5")]
    panic_btn: String,

    /// OSC Path segment for Preset Encoder (substring match)
    #[arg(long, default_value = "A/enc/5")]
    preset_enc: String,

    /// OSC Path for feedback LED to flash with the beat
    #[arg(long, default_value = "/config/usercontrols/assign/A/btn/5/led")]
    btn_led_path: String,

    /// OSC Path for target scribble strip to show BPM and preset style
    #[arg(long, default_value = "/ch/32/config/name")]
    info_scribble_path: String,

    /// Target Channels for Compressor Sync (e.g., "1,2,3" or "1-4")
    #[arg(long)]
    target_channels: Option<String>,

    /// Compressor Release Subdivision (default: 1.0 = Quarter Note)
    #[arg(long, default_value_t = 1.0)]
    compressor_subdivision: f32,
}

/// Subcommands for the tool.
#[derive(Subcommand)]
enum Commands {
    /// List available audio devices.
    ListDevices,
}

#[derive(Debug, PartialEq)]
enum Algorithm {
    Energy,
    Spectral,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::Energy => f.write_str("Energy"),
            Algorithm::Spectral => f.write_str("Spectral"),
        }
    }
}

fn parse_channels(s: &str) -> Vec<usize> {
    let mut channels = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let ranges: Vec<&str> = part.split('-').collect();
            if ranges.len() == 2 {
                if let (Ok(start), Ok(end)) =
                    (ranges[0].parse::<usize>(), ranges[1].parse::<usize>())
                {
                    for i in start..=end {
                        channels.push(i);
                    }
                }
            }
        } else if let Ok(n) = part.parse::<usize>() {
            channels.push(n);
        }
    }
    channels
}
fn parse_slots(s: &str) -> Vec<usize> {
    let mut slots = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let ranges: Vec<&str> = part.split('-').collect();
            if ranges.len() == 2 {
                if let (Ok(start), Ok(end)) =
                    (ranges[0].parse::<usize>(), ranges[1].parse::<usize>())
                {
                    for i in start..=end {
                        if (1..=8).contains(&i) {
                            slots.push(i - 1); // convert to 0-indexed internally
                        }
                    }
                }
            }
        } else if let Ok(n) = part.parse::<usize>() {
            if (1..=8).contains(&n) {
                slots.push(n - 1); // convert to 0-indexed internally
            }
        }
    }
    slots
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::ListDevices) = cli.command {
        let devices = AudioEngine::list_devices()?;
        println!("Available Audio Devices:");
        for (i, d) in devices.iter().enumerate() {
            println!("{}: {}", i + 1, d);
        }
        return Ok(());
    }

    // Setup Communication Channels
    let (audio_sender, audio_receiver) = unbounded::<Vec<f32>>();
    let (net_sender, net_receiver) = unbounded::<NetworkEvent>();

    let source: Source = cli
        .channel
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    let local_audio_idx = match source {
        Source::Channel(ch) => ch - 1,
        Source::Bus(b) => b - 1,
        Source::Aux(a) => a - 1,
        Source::MainL => 0,
        Source::MainR => 1,
    };

    // Initialize Audio
    let audio_result = AudioEngine::start(cli.device.clone(), local_audio_idx + 1, audio_sender);

    let mut audio_sample_rate = 48000; // Default fallback
    let audio_started = match &audio_result {
        Ok((_, rate)) => {
            audio_sample_rate = *rate;
            true
        }
        Err(e) => {
            eprintln!(
                "Info: Audio init failed ({}). Starting in fallback mode.",
                e
            );
            std::thread::sleep(Duration::from_secs(1));
            false
        }
    };

    // Initialize Network
    let network = Arc::new(
        NetworkManager::new(&cli.ip, source, net_sender, &cli.panic_btn, &cli.preset_enc).await?,
    );

    network.connect()?;
    network.start_polling(0); // Start polling everything

    // Initialize Detectors
    let mut energy_detector = EnergyDetector::new(1.5, audio_sample_rate);
    let mut spectral_detector = SpectralFluxDetector::new(audio_sample_rate, 1024);
    let mut osc_detector = OscLevelDetector::new();

    // Initialize Compressor Handler
    let comp_channels = if let Some(s) = &cli.target_channels {
        parse_channels(s)
    } else {
        Vec::new()
    };
    let mut comp_handler = CompressorHandler::new(comp_channels);
    comp_handler.release_subdivision = cli.compressor_subdivision;

    // Initialize UI
    let mut tui = Tui::new()?;

    // State
    let mut is_panic = false;
    let target_slots = parse_slots(&cli.slot);
    let mut selected_slot = if !target_slots.is_empty() {
        target_slots[0]
    } else {
        0
    };

    // Per-slot state
    // Effect Configs
    let mut effect_configs: [EffectConfig; 8] = Default::default();
    #[allow(clippy::needless_range_loop)]
    for i in 0..8 {
        effect_configs[i].enabled = target_slots.contains(&i);
    }
    // Loaded effect handlers
    let mut effect_handlers: [Option<Box<dyn EffectHandler + Send + Sync>>; 8] = Default::default();
    // Names of active effects
    let mut active_effects: [String; 8] = Default::default();

    let mut last_ui_update = Instant::now();
    let mut last_level = 0.0;
    let mut last_audio_packet = Instant::now();

    let mut selected_algorithm = Algorithm::Energy;

    // Available options for UI cycling
    let subdivisions = ["1/4", "1/8", "1/8d", "1/8t", "1/1", "1/2", "1/16"];
    let styles = ["Standard", "Tight", "Natural", "Big", "Huge"];

    let mut last_sent_bpm: Option<u32> = None;
    let mut last_sent_style = String::new();

    loop {
        // 1. Process Audio Data (Primary)
        let mut audio_received_this_frame = false;
        while let Ok(chunk) = audio_receiver.try_recv() {
            audio_received_this_frame = true;
            last_audio_packet = Instant::now();

            if !is_panic {
                match selected_algorithm {
                    Algorithm::Energy => energy_detector.process(&chunk, audio_sample_rate),
                    Algorithm::Spectral => spectral_detector.process(&chunk, audio_sample_rate),
                }

                if !chunk.is_empty() {
                    let sum_sq: f32 = chunk.iter().map(|s| s * s).sum();
                    let rms = (sum_sq / chunk.len() as f32).sqrt();
                    last_level = rms * 5.0;
                }
            }
        }

        // Dynamic Failover Logic
        let use_fallback =
            !audio_started || (last_audio_packet.elapsed() > Duration::from_millis(200));

        // 2. Process Network Events (Fallback / Controls)
        while let Ok(event) = net_receiver.try_recv() {
            match event {
                NetworkEvent::MeterLevel(lvl) => {
                    if use_fallback {
                        if !is_panic {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64;
                            osc_detector.process_level(lvl, now);
                        }
                        last_level = lvl;
                    }
                }
                NetworkEvent::PanicTriggered => {
                    is_panic = true;
                    // Trigger panic on all slots
                    for (i, handler_opt) in effect_handlers.iter().enumerate() {
                        if let Some(h) = handler_opt {
                            let _ = h.panic(&network, i + 1).await;
                        }
                    }
                    let _ = network.set_scribble_text(source, "PANIC!").await;
                }
                NetworkEvent::EncoderTurned(val) => {
                    let cfg = &mut effect_configs[selected_slot];
                    let current_idx = styles.iter().position(|&s| s == cfg.style).unwrap_or(0);
                    let new_idx = if val > 0 {
                        (current_idx + 1) % styles.len()
                    } else {
                        if current_idx == 0 {
                            styles.len() - 1
                        } else {
                            current_idx - 1
                        }
                    };
                    cfg.style = styles[new_idx].to_string();
                }
                NetworkEvent::EffectLoaded(slot, name) => {
                    if (1..=8).contains(&slot) {
                        let idx = slot - 1;
                        if active_effects[idx] != name {
                            active_effects[idx] = name.clone();
                            effect_handlers[idx] = get_handler(&name);
                        }
                    }
                }
            }
        }

        // 2b. Flash Beat LED if onset detected
        let is_beat_detected = if !is_panic {
            if !use_fallback {
                if audio_received_this_frame {
                    match selected_algorithm {
                        Algorithm::Energy => energy_detector.is_beat(),
                        Algorithm::Spectral => spectral_detector.is_beat(),
                    }
                } else {
                    false
                }
            } else {
                osc_detector.is_beat()
            }
        } else {
            false
        };

        if is_beat_detected {
            let network_clone = network.clone();
            let path_clone = cli.btn_led_path.clone();
            tokio::spawn(async move {
                let _ = network_clone.send_osc_float(&path_clone, 1.0).await;
                tokio::time::sleep(Duration::from_millis(80)).await;
                let _ = network_clone.send_osc_float(&path_clone, 0.0).await;
            });
        }

        // 3. Determine Active BPM
        let active_bpm = if !use_fallback {
            match selected_algorithm {
                Algorithm::Energy => energy_detector.current_bpm(),
                Algorithm::Spectral => spectral_detector.current_bpm(),
            }
        } else {
            osc_detector.current_bpm()
        };

        // Update User Assigned Scribble Strip Display
        let rounded_bpm = active_bpm.map(|b| b.round() as u32);
        let current_style = effect_configs[selected_slot].style.clone();
        if rounded_bpm != last_sent_bpm || current_style != last_sent_style {
            last_sent_bpm = rounded_bpm;
            last_sent_style = current_style.clone();
            let bpm_str = rounded_bpm
                .map(|b| format!("{}BPM", b))
                .unwrap_or_else(|| "---BPM".to_string());
            let text = format!("{} {}", bpm_str, current_style.to_uppercase());
            let _ = network
                .set_scribble_target(&cli.info_scribble_path, &text)
                .await;
        }

        // 4. Update Effects (All Slots)
        if !is_panic {
            if let Some(bpm) = active_bpm {
                for (i, handler_opt) in effect_handlers.iter().enumerate() {
                    if let Some(h) = handler_opt {
                        let cfg = &effect_configs[i];
                        if cfg.enabled {
                            let _ = h.update(&network, i + 1, bpm, cfg).await;
                        }
                    }
                }
                // Update Compressors
                let _ = comp_handler.update(&network, bpm).await;
            }
        }

        // 5. UI Update & Input
        if last_ui_update.elapsed() > Duration::from_millis(50) {
            let state = AppState {
                source: source.to_string(),
                current_bpm: active_bpm,
                input_level: last_level,
                active_effects: active_effects.clone(),
                effect_configs: effect_configs.clone(),
                is_supported: {
                    let mut is_supp = [false; 8];
                    #[allow(clippy::needless_range_loop)]
                    for i in 0..8 {
                        is_supp[i] = effect_handlers[i].is_some();
                    }
                    is_supp
                },
                selected_slot,
                is_fallback: use_fallback,
                is_panic,
                message: if use_fallback {
                    "Fallback (OSC)".to_string()
                } else {
                    "Audio OK".to_string()
                },
                algorithm: selected_algorithm.to_string(),
            };

            tui.draw(&state)?;

            if let Some(event) = tui.handle_events()? {
                match event {
                    UIEvent::Panic => {
                        is_panic = true;
                        for (i, handler_opt) in effect_handlers.iter().enumerate() {
                            if let Some(h) = handler_opt {
                                let _ = h.panic(&network, i + 1).await;
                            }
                        }
                    }
                    UIEvent::Quit => break,
                    UIEvent::Reset => {
                        is_panic = false;
                    }
                    UIEvent::SwitchAlgorithm => {
                        selected_algorithm = match selected_algorithm {
                            Algorithm::Energy => Algorithm::Spectral,
                            Algorithm::Spectral => Algorithm::Energy,
                        };
                    }
                    UIEvent::NextSlot => {
                        selected_slot = (selected_slot + 1) % 8;
                    }
                    UIEvent::PrevSlot => {
                        if selected_slot == 0 {
                            selected_slot = 7;
                        } else {
                            selected_slot -= 1;
                        }
                    }
                    UIEvent::NextSubdiv => {
                        let cfg = &mut effect_configs[selected_slot];
                        let current_idx = subdivisions
                            .iter()
                            .position(|&s| s == cfg.subdivision)
                            .unwrap_or(0);
                        let new_idx = (current_idx + 1) % subdivisions.len();
                        cfg.subdivision = subdivisions[new_idx].to_string();
                    }
                    UIEvent::PrevSubdiv => {
                        let cfg = &mut effect_configs[selected_slot];
                        let current_idx = subdivisions
                            .iter()
                            .position(|&s| s == cfg.subdivision)
                            .unwrap_or(0);
                        let new_idx = if current_idx == 0 {
                            subdivisions.len() - 1
                        } else {
                            current_idx - 1
                        };
                        cfg.subdivision = subdivisions[new_idx].to_string();
                    }
                    UIEvent::NextStyle => {
                        let cfg = &mut effect_configs[selected_slot];
                        let current_idx = styles.iter().position(|&s| s == cfg.style).unwrap_or(0);
                        let new_idx = (current_idx + 1) % styles.len();
                        cfg.style = styles[new_idx].to_string();
                    }
                    UIEvent::PrevStyle => {
                        let cfg = &mut effect_configs[selected_slot];
                        let current_idx = styles.iter().position(|&s| s == cfg.style).unwrap_or(0);
                        let new_idx = if current_idx == 0 {
                            styles.len() - 1
                        } else {
                            current_idx - 1
                        };
                        cfg.style = styles[new_idx].to_string();
                    }
                    UIEvent::ToggleSync => {
                        let cfg = &mut effect_configs[selected_slot];
                        cfg.enabled = !cfg.enabled;
                    }
                }

                let cfg = &effect_configs[selected_slot];
                let text = format!("FX{}:{}", selected_slot + 1, cfg.subdivision);
                let _ = network.set_scribble_text(source, &text).await;
            }

            last_ui_update = Instant::now();
        }

        if !audio_received_this_frame {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    tui.cleanup()?;
    Ok(())
}

#[cfg(test)]
mod tests_scaling;
