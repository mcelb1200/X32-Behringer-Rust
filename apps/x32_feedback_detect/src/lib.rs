use anyhow::{Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use x32_lib::MixerClient;

pub mod detector;
pub mod mixer;
pub mod tui;

use detector::FeedbackDetector;
use mixer::MixerState;
use tui::{AppTui, TuiEvent};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long, default_value = "192.168.1.100")]
    pub ip: String,

    /// Target channel to insert EQ notches (e.g., 1 for Ch 01)
    #[arg(short, long, default_value_t = 1)]
    pub channel: u8,
}

pub async fn run(args: Args) -> Result<()> {
    // 1. Set up audio capture using cpal
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No input device available")?;
    let config: cpal::StreamConfig = device.default_input_config()?.into();
    let sample_rate = config.sample_rate.0;

    let (mut producer, mut consumer) = HeapRb::<f32>::new(4096 * 4).split();

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for &sample in data {
                let _ = producer.push(sample);
            }
        },
        move |err| {
            eprintln!("Audio input stream error: {}", err);
        },
        None,
    )?;

    stream.play()?;

    // 2. Setup X32 connection
    let client = MixerClient::connect(&args.ip, true).await?;
    let mixer_state = Arc::new(Mutex::new(MixerState::new(client, args.channel)));

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
