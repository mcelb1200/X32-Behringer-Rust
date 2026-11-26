//! Automatic feedback detection and parametric EQ notch insertion for Behringer mixers.

use anyhow::{Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use x32_lib::{MixerClient, MixerModel};

/// FFT-based feedback detection module.
pub mod detector;
/// Mixer EQ notch state and OSC control module.
pub mod mixer;
/// Terminal UI module.
pub mod tui;

use detector::FeedbackDetector;
use mixer::MixerState;
use tui::{AppTui, TuiEvent};

/// Command-line arguments for `x32_feedback_detect`.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Automatic Feedback Detection and Management", long_about = None)]
pub struct Args {
    /// IP address of the console
    #[arg(short, long, default_value = "192.168.1.100")]
    pub ip: String,

    /// Target channel to insert EQ notches (e.g., 1 for Ch 01)
    #[arg(short, long, default_value_t = 1)]
    pub channel: u8,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short = 'M', long, default_value = "X32")]
    pub model: MixerModel,

    /// Audio input device name substring (defaults to system default input)
    #[arg(long)]
    pub device: Option<String>,

    /// 1-based audio interface input channel to analyze (0 = mono sum of all channels)
    #[arg(long, default_value_t = 1)]
    pub input_channel: usize,

    /// List available audio input devices and exit
    #[arg(long, default_value_t = false)]
    pub list_devices: bool,
}

/// Extracts a single mono sample stream from an interleaved multi-channel audio frame slice.
///
/// If `input_channel == 0`, averages all channels in each frame (mono sum).
/// Otherwise extracts the 1-based channel index (clamped to `channels`).
pub fn extract_mono_sample(frame: &[f32], input_channel: usize) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    if input_channel == 0 {
        frame.iter().copied().sum::<f32>() / frame.len() as f32
    } else {
        let idx = (input_channel - 1).min(frame.len() - 1);
        frame[idx]
    }
}

/// Returns maximum allowed input channels for the specified mixer model.
pub fn max_channels_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 => 32,
        MixerModel::Wing => 40,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

/// Runs the `x32_feedback_detect` audio capture, detection, and TUI loop.
pub async fn run(args: Args) -> Result<()> {
    let host = cpal::default_host();

    if args.list_devices {
        println!("Available audio input devices:");
        if let Ok(devices) = host.input_devices() {
            for (idx, dev) in devices.enumerate() {
                let name = dev.name().unwrap_or_else(|_| "<unknown>".to_string());
                let ch_info = dev
                    .default_input_config()
                    .map(|c| format!("{} ch @ {} Hz", c.channels(), c.sample_rate().0))
                    .unwrap_or_else(|_| "unknown format".to_string());
                println!("  [{}] {} ({})", idx + 1, name, ch_info);
            }
        }
        return Ok(());
    }

    let max_ch = max_channels_for_model(args.model);
    if args.channel < 1 || args.channel > max_ch {
        anyhow::bail!(
            "Invalid target channel {} for model {} (valid range: 1-{})",
            args.channel,
            args.model,
            max_ch
        );
    }

    // 1. Set up audio capture using cpal (with device selection and multi-channel de-interleaving)
    let device = if let Some(ref filter) = args.device {
        let needle = filter.to_ascii_lowercase();
        host.input_devices()?
            .find(|d| {
                d.name()
                    .map(|n| n.to_ascii_lowercase().contains(&needle))
                    .unwrap_or(false)
            })
            .with_context(|| format!("No audio input device matching '{}'", filter))?
    } else {
        host.default_input_device()
            .context("No input device available")?
    };

    let supported_config = device.default_input_config()?;
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();
    let sample_rate = config.sample_rate.0;
    let ch_count = (config.channels as usize).max(1);
    let selected_input_ch = args.input_channel;

    let (mut producer, mut consumer) = HeapRb::<f32>::new(4096 * 4).split();

    let err_fn = move |err| {
        eprintln!("Audio input stream error: {}", err);
    };

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for frame in data.chunks_exact(ch_count) {
                    let _ = producer.push(extract_mono_sample(frame, selected_input_ch));
                }
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                for frame in data.chunks_exact(ch_count) {
                    let s = if selected_input_ch == 0 {
                        frame
                            .iter()
                            .map(|&v| v as f32 / i16::MAX as f32)
                            .sum::<f32>()
                            / ch_count as f32
                    } else {
                        let idx = (selected_input_ch - 1).min(ch_count - 1);
                        frame[idx] as f32 / i16::MAX as f32
                    };
                    let _ = producer.push(s);
                }
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                let half = u16::MAX as f32 / 2.0;
                for frame in data.chunks_exact(ch_count) {
                    let s = if selected_input_ch == 0 {
                        frame.iter().map(|&v| (v as f32 - half) / half).sum::<f32>()
                            / ch_count as f32
                    } else {
                        let idx = (selected_input_ch - 1).min(ch_count - 1);
                        (frame[idx] as f32 - half) / half
                    };
                    let _ = producer.push(s);
                }
            },
            err_fn,
            None,
        )?,
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    };

    stream.play()?;

    // 2. Setup Mixer connection
    let client = MixerClient::connect(&args.ip, true).await?;
    let mixer_state = Arc::new(Mutex::new(MixerState::new(
        client,
        args.channel,
        args.model,
    )));

    // 3. Setup UI
    let mut tui = AppTui::new()?;
    let mut detector = FeedbackDetector::new(sample_rate, 2048);

    let mut last_tick = Instant::now();
    let mut status = "Listening...".to_string();

    let mut audio_buffer = Vec::with_capacity(2048);

    loop {
        // TUI Events
        match tui.handle_events()? {
            TuiEvent::Quit => break,
            TuiEvent::ResetNotches => {
                let mut state = mixer_state.lock().await;
                state.reset_notches().await?;
                status = "Notches reset. Listening...".to_string();
            }
            TuiEvent::None => {}
        }

        // Process audio
        let to_read = consumer.len();
        if to_read >= 2048 {
            audio_buffer.clear();
            for _ in 0..2048 {
                if let Some(s) = consumer.pop() {
                    audio_buffer.push(s);
                }
            }

            let now = Instant::now();
            let delta = now.duration_since(last_tick).as_millis() as u64;
            last_tick = now;

            let feedback_events = detector.process(&audio_buffer, delta);

            if !feedback_events.is_empty() {
                let mut state = mixer_state.lock().await;
                for fb in feedback_events {
                    if let Err(e) = state.apply_notch(fb.frequency).await {
                        status = format!("Err applying notch: {}", e);
                    } else {
                        status = format!("Feedback detected at {:.1} Hz!", fb.frequency);
                    }
                }
            } else if last_tick.elapsed() > Duration::from_secs(2)
                && status.contains("Feedback detected")
            {
                status = "Listening...".to_string();
            }
        }

        // Draw TUI
        {
            let state = mixer_state.lock().await;
            tui.draw(&status, &state.applied_notches)?;
        }

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    tui.cleanup()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_channels_for_model() {
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);
    }

    #[test]
    fn test_channel_bounds_validation() {
        let models = [
            (MixerModel::X32, 32),
            (MixerModel::Wing, 40),
            (MixerModel::XR18, 16),
            (MixerModel::XR16, 16),
            (MixerModel::XR12, 12),
        ];

        for (model, max_ch) in models {
            assert!(1 <= max_ch);
            assert_eq!(max_channels_for_model(model), max_ch);
        }
    }
}
