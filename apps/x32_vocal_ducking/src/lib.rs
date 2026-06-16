//! `x32_vocal_ducking` is a tool that implements dynamic EQ sidechaining
//! and surgical frequency carving on an instrument bus to make space for vocals.

use anyhow::{Result, anyhow};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use osc_lib::{OscArg, OscMessage};
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use x32_lib::MixerClient;

#[derive(Parser, Debug)]
#[command(
    name = "x32_vocal_ducking",
    version = "0.1.0",
    about = "Dynamic Vocal Ducking & Spectral Carver for X32"
)]
pub struct Cli {
    /// X32 IP Address
    #[arg(long, default_value = "192.168.1.50")]
    pub ip: String,

    /// Substring match for audio device name (USB Mode)
    #[arg(long)]
    pub audio_device: Option<String>,

    /// Vocal input channel index on the USB interface (1-based, 1-32)
    #[arg(long)]
    pub card_channel: Option<usize>,

    /// Vocal bus index on the X32 (1-16) to act as trigger
    #[arg(long)]
    pub vocal_bus: Option<usize>,

    /// Instrument bus index on the X32 to duck (1-16)
    #[arg(long, default_value_t = 1)]
    pub instrument_bus: usize,

    /// Ducking threshold in dBFS
    #[arg(long, default_value_t = -35.0)]
    pub threshold: f32,

    /// Compression Ratio (e.g. 4.0 for 4:1)
    #[arg(long, default_value_t = 4.0)]
    pub ratio: f32,

    /// Attack time in milliseconds
    #[arg(long, default_value_t = 15.0)]
    pub attack: f32,

    /// Release time in milliseconds
    #[arg(long, default_value_t = 150.0)]
    pub release: f32,

    /// Maximum gain reduction in dB
    #[arg(long, default_value_t = 6.0)]
    pub max_duck: f32,

    /// Dynamically tune X32 bus compressor key filter frequency based on vocal peak
    #[arg(long)]
    pub use_key_filter: bool,

    /// List available audio devices and exit
    #[arg(long)]
    pub list_devices: bool,
}

// Convert frequency [20..20000] Hz to logarithmic float [0..1]
fn log2float(val: f32, min: f32, range_log: f32) -> f32 {
    if val <= min {
        return 0.0;
    }
    let res = (val / min).ln() / range_log;
    res.clamp(0.0, 1.0)
}

// Convert dB [-15..15] to EQ gain [0..1]
fn db_to_eq_gain(db: f32) -> f32 {
    ((db + 15.0) / 30.0).clamp(0.0, 1.0)
}

struct AudioMsg {
    db: f32,
    peak_freq: f32,
}

fn list_devices() -> Result<()> {
    let host = cpal::default_host();
    println!("Available input devices:");
    for device in host.input_devices()? {
        if let Ok(name) = device.name() {
            println!(" - {}", name);
        }
    }
    Ok(())
}

pub async fn run(cli: Cli) -> Result<()> {
    if cli.list_devices {
        return list_devices();
    }

    // Connect to X32
    println!("Connecting to X32 at {}...", cli.ip);
    let client = Arc::new(MixerClient::connect(&cli.ip, true).await?);
    if !client.probe().await {
        return Err(anyhow!("Could not verify connection to X32 at {}", cli.ip));
    }
    println!("Connected successfully to X32.");

    // Setup communication channels
    let (tx, mut rx) = mpsc::channel::<AudioMsg>(100);
    let running = Arc::new(AtomicBool::new(true));
    let r_clone = running.clone();

    // Spawn Audio thread if USB mode is selected
    let _stream = if let (Some(device_query), Some(channel)) = (&cli.audio_device, cli.card_channel)
    {
        println!("Initializing USB Audio Mode...");
        let host = cpal::default_host();
        let device = host
            .input_devices()?
            .find(|d| d.name().map(|n| n.contains(device_query)).unwrap_or(false))
            .ok_or_else(|| anyhow!("Audio device containing '{}' not found", device_query))?;

        let config: cpal::StreamConfig = device.default_input_config()?.into();
        let sample_rate = config.sample_rate.0;
        let channels = config.channels as usize;

        if channel > channels || channel == 0 {
            return Err(anyhow!(
                "Target channel {} out of range (1-{})",
                channel,
                channels
            ));
        }

        println!("Using device: {}", device.name()?);
        println!("Sample Rate: {} Hz, Channels: {}", sample_rate, channels);

        // Circular buffer for FFT
        let mut audio_buffer = vec![0.0f32; 1024];
        let mut buffer_idx = 0;

        // Envelope tracking state
        let mut envelope = 0.0f32;
        let attack_coef = f32::exp(-1.0 / (sample_rate as f32 * (cli.attack / 1000.0)));
        let release_coef = f32::exp(-1.0 / (sample_rate as f32 * (cli.release / 1000.0)));

        let err_fn = |err| eprintln!("Audio stream error: {}", err);
        let tx_clone = tx.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| {
                for frame in data.chunks(channels) {
                    if let Some(&sample) = frame.get(channel - 1) {
                        // Envelope tracking
                        let val = sample.abs();
                        if val > envelope {
                            envelope = attack_coef * envelope + (1.0 - attack_coef) * val;
                        } else {
                            envelope = release_coef * envelope + (1.0 - release_coef) * val;
                        }

                        // Store in circular buffer for FFT
                        audio_buffer[buffer_idx] = sample;
                        buffer_idx = (buffer_idx + 1) % 1024;

                        // Every 512 samples, run FFT
                        if buffer_idx % 512 == 0 {
                            let db = if envelope > 1e-5 {
                                20.0 * envelope.log10()
                            } else {
                                -100.0
                            };

                            // Only run FFT if vocal is active to avoid picking up noise floor
                            let mut peak_freq = 2500.0f32; // Default fallback frequency
                            if db > cli.threshold {
                                // Extract window in order
                                let mut window = vec![0.0f32; 1024];
                                for i in 0..1024 {
                                    window[i] = audio_buffer[(buffer_idx + i) % 1024];
                                }

                                // Apply Hanning window
                                let mut fft_input: Vec<Complex<f32>> = window
                                    .iter()
                                    .enumerate()
                                    .map(|(idx, &val)| {
                                        let w = 0.5
                                            * (1.0
                                                - f32::cos(
                                                    2.0 * std::f32::consts::PI * idx as f32
                                                        / 1023.0,
                                                ));
                                        Complex::new(val * w, 0.0)
                                    })
                                    .collect();

                                let mut planner = FftPlanner::new();
                                let fft = planner.plan_fft_forward(1024);
                                fft.process(&mut fft_input);

                                // Find max magnitude in 1200Hz - 3500Hz range
                                // Bin frequency = bin * sample_rate / 1024
                                let bin_width = sample_rate as f32 / 1024.0;
                                let min_bin = (1200.0 / bin_width).floor() as usize;
                                let max_bin = (3500.0 / bin_width).ceil() as usize;

                                let mut max_mag = 0.0f32;
                                let mut max_bin_idx = min_bin;
                                for b in min_bin..=max_bin {
                                    if b < fft_input.len() {
                                        let mag = fft_input[b].norm();
                                        if mag > max_mag {
                                            max_mag = mag;
                                            max_bin_idx = b;
                                        }
                                    }
                                }
                                peak_freq = max_bin_idx as f32 * bin_width;
                            }

                            let _ = tx_clone.try_send(AudioMsg { db, peak_freq });
                        }
                    }
                }
            },
            err_fn,
            None,
        )?;

        stream.play()?;
        Some(stream)
    } else {
        None
    };

    // Spawn X32 OSC Meter subscription task if in network mode (fallback)
    let client_clone = client.clone();
    if _stream.is_none() {
        if let Some(v_bus) = cli.vocal_bus {
            println!("Initializing Pure OSC Mode (Vocal Bus: Bus {})...", v_bus);
            tokio::spawn(async move {
                // Subscribe to /meters/2 (contains bus levels)
                let subscribe_msg = OscMessage::new(
                    "/meters".to_string(),
                    vec![
                        OscArg::String("/meters/2".to_string()),
                        OscArg::Int(50), // 50ms update rate
                    ],
                );
                let _ = client_clone
                    .send_message("/meters", subscribe_msg.args)
                    .await;

                let mut osc_rx = client_clone.subscribe();
                let mut moving_avg_window = [0.0f32; 10]; // ~500ms sliding window
                let mut window_idx = 0;

                while r_clone.load(Ordering::Relaxed) {
                    if let Ok(msg) = osc_rx.recv().await {
                        if msg.path == "/meters/2" {
                            if let Some(OscArg::Blob(data)) = msg.args.first() {
                                // Bus offset is (v_bus - 1) * 4 bytes
                                let start = (v_bus - 1) * 4;
                                let end = start + 4;
                                if data.len() >= end {
                                    let mut bytes = [0u8; 4];
                                    bytes.copy_from_slice(&data[start..end]);
                                    let level = f32::from_le_bytes(bytes);

                                    // Convert level to dBFS
                                    let db = if level > 1e-5 {
                                        20.0 * level.log10()
                                    } else {
                                        -100.0
                                    };

                                    // Moving average smoothing
                                    moving_avg_window[window_idx] = db;
                                    window_idx = (window_idx + 1) % 10;
                                    let avg_db = moving_avg_window.iter().sum::<f32>() / 10.0;

                                    let _ = tx.try_send(AudioMsg {
                                        db: avg_db,
                                        peak_freq: 2500.0, // Static fallback frequency
                                    });
                                }
                            }
                        }
                    }
                }
            });
        } else {
            return Err(anyhow!(
                "Must specify either (--audio-device + --card-channel) OR --vocal-bus"
            ));
        }
    }

    // Ctrl-C handler
    let r_ctrl_c = running.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("\nShutting down, restoring settings...");
        r_ctrl_c.store(false, Ordering::Relaxed);
    });

    println!("\nDynamic Ducking Active. Press Ctrl-C to quit.\n");
    println!("--------------------------------------------------");
    println!("  Vocal Level | Gain Reduction | Notch Freq");
    println!("--------------------------------------------------");

    // OSC Control State variables for deadband / throttling
    let mut last_sent_gain = 0.0f32;
    let mut last_sent_freq = 2500.0f32;
    let mut last_update_time = Instant::now();

    let range_log = (20000.0f32 / 20.0f32).ln();

    // Main control loop
    while running.load(Ordering::Relaxed) {
        if let Ok(msg) = rx.try_recv() {
            // 1. Calculate Target Gain Reduction
            let mut reduction_db = 0.0f32;
            if msg.db > cli.threshold {
                let over = msg.db - cli.threshold;
                reduction_db = (over * (1.0 - 1.0 / cli.ratio)).clamp(0.0, cli.max_duck);
            }
            let target_gain = -reduction_db;

            // 2. Deadband and rate limiting check (max 100Hz updates)
            let time_elapsed = last_update_time.elapsed() >= Duration::from_millis(10);
            let gain_changed = (target_gain - last_sent_gain).abs() > 0.15;
            let freq_changed = (msg.peak_freq - last_sent_freq).abs() / last_sent_freq > 0.02;

            if time_elapsed && (gain_changed || freq_changed) {
                last_update_time = Instant::now();

                // Send Surgical PEQ notch gain update
                // Instrument bus EQ Band 3 path: /bus/[bus_idx]/eq/3/g
                let eq_path_g = format!("/bus/{:02}/eq/3/g", cli.instrument_bus);
                let _ = client
                    .send_message(&eq_path_g, vec![OscArg::Float(db_to_eq_gain(target_gain))])
                    .await;
                last_sent_gain = target_gain;

                // Send Surgical PEQ notch frequency if peak frequency is dynamic (USB mode)
                if _stream.is_some() && freq_changed {
                    let eq_path_f = format!("/bus/{:02}/eq/3/f", cli.instrument_bus);
                    let freq_float = log2float(msg.peak_freq, 20.0, range_log);
                    let _ = client
                        .send_message(&eq_path_f, vec![OscArg::Float(freq_float)])
                        .await;

                    // Dynamically tune dynamic sidechain key filter if enabled
                    if cli.use_key_filter {
                        let filter_path = format!("/bus/{:02}/dyn/filter/f", cli.instrument_bus);
                        let _ = client
                            .send_message(&filter_path, vec![OscArg::Float(freq_float)])
                            .await;
                    }
                    last_sent_freq = msg.peak_freq;
                }

                // Render simple status line
                print!(
                    "\r  {:>8.1} dB | {:>11.1} dB | {:>7.0} Hz",
                    msg.db, target_gain, last_sent_freq
                );
                std::io::Write::flush(&mut std::io::stdout())?;
            }
        }
        sleep(Duration::from_millis(5)).await;
    }

    // Graceful cleanup: restore EQ band to flat (0dB) before exit
    let eq_path_cleanup = format!("/bus/{:02}/eq/3/g", cli.instrument_bus);
    let _ = client
        .send_message(&eq_path_cleanup, vec![OscArg::Float(db_to_eq_gain(0.0))])
        .await;
    println!("\nSettings restored. Exiting.");

    Ok(())
}
