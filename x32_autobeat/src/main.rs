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
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]

use crate::audio::AudioEngine;
use crate::detection::{BeatDetector, EnergyDetector, OscLevelDetector};
use crate::effects::{get_handler, EffectHandler};
use crate::network::{NetworkEvent, NetworkManager};
use crate::ui::{AppState, Tui, UIEvent};
use anyhow::Result;
use clap::{Parser, Subcommand};
use crossbeam_channel::unbounded;
use std::sync::Arc;
use std::time::{Duration, Instant};

mod audio;
mod detection;
mod effects;
mod network;
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

    /// Audio Input Channel (1-32).
    #[arg(long, default_value_t = 1)]
    channel: usize,

    /// Target Effect Slot (1-8).
    #[arg(long, default_value_t = 1)]
    slot: usize,

    /// OSC Path segment for Panic Button (substring match).
    /// Example: "A/btn/5" matches "/config/userctrl/A/btn/5"
    #[arg(long, default_value = "A/btn/5")]
    panic_btn: String,

    /// OSC Path segment for Preset Encoder (substring match).
    /// Example: "A/enc/5" matches "/config/userctrl/A/enc/5"
    #[arg(long, default_value = "A/enc/5")]
    preset_enc: String,
}

/// Subcommands for the tool.
#[derive(Subcommand)]
enum Commands {
    /// List available audio devices.
    ListDevices,
}

/// The main entry point for the application.
///
/// This function initializes the audio engine, network connection, and user interface.
/// It then enters a main loop that processes audio data, network events, and user input
/// to perform beat detection and update the mixer's effects.
fn main() -> Result<()> {
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

    // Initialize Audio
    let audio_result = AudioEngine::start(cli.device.clone(), cli.channel, audio_sender);

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
    let channel_idx = if cli.channel > 0 { cli.channel - 1 } else { 0 };
    let network = Arc::new(NetworkManager::new(
        &cli.ip,
        channel_idx,
        net_sender,
        &cli.panic_btn,
        &cli.preset_enc,
    )?);

    network.connect()?;
    network.start_polling(cli.slot);

    // Initialize Detectors
    let mut energy_detector = EnergyDetector::new(1.5, audio_sample_rate);
    let mut osc_detector = OscLevelDetector::new();

    // Initialize UI
    let mut tui = Tui::new()?;

    // State
    let mut is_panic = false;
    let mut subdivision = 1.0;
    let subdivisions = [1.0, 0.75, 0.5, 0.375, 0.25];
    let mut subdiv_idx = 0;
    let mut active_effect = "Detecting...".to_string(); // Initial state
    let mut current_handler: Option<Box<dyn EffectHandler + Send + Sync>> = None; // Wait for detection

    let mut last_ui_update = Instant::now();
    let mut last_level = 0.0;
    let mut last_audio_packet = Instant::now();

    loop {
        // 1. Process Audio Data (Primary)
        let mut audio_received_this_frame = false;
        while let Ok(chunk) = audio_receiver.try_recv() {
            audio_received_this_frame = true;
            last_audio_packet = Instant::now();

            if !is_panic {
                energy_detector.process(&chunk, audio_sample_rate);
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
                                .unwrap()
                                .as_millis() as u64;
                            osc_detector.process_level(lvl, now);
                        }
                        last_level = lvl;
                    }
                }
                NetworkEvent::PanicTriggered => {
                    is_panic = true;
                    if let Some(h) = &current_handler {
                        let _ = h.panic(&network, cli.slot);
                    }
                    let _ = network.set_scribble_text(cli.channel, "PANIC!");
                }
                NetworkEvent::EncoderTurned(_val) => {
                    subdiv_idx = (subdiv_idx + 1) % subdivisions.len();
                    subdivision = subdivisions[subdiv_idx];
                }
                NetworkEvent::EffectLoaded(name) => {
                    if name != active_effect {
                        active_effect = name.clone();
                        current_handler = get_handler(&name);
                    }
                }
            }
        }

        // 3. Determine Active BPM
        let active_bpm = if !use_fallback {
            energy_detector.current_bpm()
        } else {
            osc_detector.current_bpm()
        };

        // 4. Update Effect
        if !is_panic {
            if let Some(bpm) = active_bpm {
                if let Some(h) = &current_handler {
                    let _ = h.update(&network, cli.slot, bpm, subdivision);
                }
            }
        }

        // 5. UI Update & Input
        if last_ui_update.elapsed() > Duration::from_millis(50) {
            let state = AppState {
                current_bpm: active_bpm,
                input_level: last_level,
                active_effect: active_effect.clone(),
                subdivision: format!("{:.3}", subdivision),
                is_fallback: use_fallback,
                is_panic,
                message: if use_fallback {
                    "Fallback (OSC)".to_string()
                } else {
                    "Audio OK".to_string()
                },
            };

            tui.draw(&state)?;

            if let Some(event) = tui.handle_events()? {
                match event {
                    UIEvent::Panic => {
                        is_panic = true;
                        if let Some(h) = &current_handler {
                            let _ = h.panic(&network, cli.slot);
                        }
                    }
                    UIEvent::Quit => break,
                    UIEvent::Reset => {
                        is_panic = false;
                    }
                    // Removed unreachable patterns or explicit ignore
                    _ => {}
                }
            }

            last_ui_update = Instant::now();
        }

        if !audio_received_this_frame {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    tui.cleanup()?;
    Ok(())
}
