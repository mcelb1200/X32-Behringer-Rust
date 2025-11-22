use anyhow::Result;
use crossbeam_channel::Sender;

#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioEngine {
    #[cfg(feature = "audio")]
    _stream: cpal::Stream,
}

impl AudioEngine {
    pub fn list_devices() -> Result<Vec<String>> {
        #[cfg(feature = "audio")]
        {
            let host = cpal::default_host();
            let devices = host.input_devices()?;
            let mut names = Vec::new();
            for device in devices {
                if let Ok(name) = device.name() {
                    names.push(name);
                }
            }
            Ok(names)
        }
        #[cfg(not(feature = "audio"))]
        {
            Ok(vec!["(Audio feature disabled)".to_string()])
        }
    }

    // Return (Self, sample_rate)
    #[allow(unused_variables)]
    pub fn start(
        device_name_query: Option<String>,
        target_channel: usize, // 1-based index
        data_sender: Sender<Vec<f32>>,
    ) -> Result<(Self, u32)> {
        #[cfg(feature = "audio")]
        {
            let host = cpal::default_host();

            let device = if let Some(query) = device_name_query {
                host.input_devices()?
                    .find(|x| x.name().map(|n| n.contains(&query)).unwrap_or(false))
                    .context("Audio device not found matching query")?
            } else {
                host.default_input_device()
                    .context("No default audio device available")?
            };

            let config: cpal::StreamConfig = device.default_input_config()?.into();
            let sample_rate = config.sample_rate.0;
            let channels = config.channels as usize;

            // println!("Starting audio on device: {}", device.name()?); // Don't print to stdout in TUI app

            if target_channel > channels || target_channel == 0 {
                anyhow::bail!(
                    "Target channel {} is out of range (1-{})",
                    target_channel,
                    channels
                );
            }

            let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

            let stream = device.build_input_stream(
                &config,
                move |data: &[f32], _: &_| {
                    let mut mono_chunk = Vec::with_capacity(data.len() / channels);
                    for frame in data.chunks(channels) {
                        if let Some(&sample) = frame.get(target_channel - 1) {
                            mono_chunk.push(sample);
                        }
                    }
                    let _ = data_sender.send(mono_chunk);
                },
                err_fn,
                None,
            )?;

            stream.play()?;

            Ok((Self { _stream: stream }, sample_rate))
        }

        #[cfg(not(feature = "audio"))]
        {
            anyhow::bail!("Audio feature is disabled in this build.")
        }
    }
}
