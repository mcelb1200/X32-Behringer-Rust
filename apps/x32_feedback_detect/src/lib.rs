use anyhow::{Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use x32_lib::{MixerClient, MixerModel};

pub mod detector;
pub mod mixer;
pub mod tui;

use detector::FeedbackDetector;
use mixer::MixerState;
use tui::{AppTui, TuiEvent};

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

pub async fn run(args: Args) -> Result<()> {
    let max_ch = max_channels_for_model(args.model);
    if args.channel < 1 || args.channel > max_ch {
        anyhow::bail!(
            "Invalid target channel {} for model {} (valid range: 1-{})",
            args.channel,
            args.model,
            max_ch
        );
    }
    // 1. Set up audio capture using cpal
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No input device available")?;
    let config: cpal::StreamConfig = device.default_input_config()?.into();
    let sample_rate = config.sample_rate.0;

    let (mut producer, mut consumer) = HeapRb::<f32>::new(4096 * 4).split();

    let sample_format = device.default_input_config()?.sample_format();

    let err_fn = move |err| {
        eprintln!("Audio input stream error: {}", err);
    };

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for &sample in data {
                    let _ = producer.push(sample);
                }
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                for &sample in data {
                    let _ = producer.push(sample as f32 / i16::MAX as f32);
                }
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                for &sample in data {
                    let _ = producer
                        .push((sample as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0));
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
