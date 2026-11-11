//! `x32_tap` is a command-line tool for setting the tempo of X32 delay effects by tapping.
//!
//! It connects to the mixer and listens for user input (pressing Enter). By measuring the
//! time interval between taps, it calculates the tempo in milliseconds and updates the
//! corresponding parameter of the delay effect in the specified FX slot.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200
//!

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use osc_lib::OscArg;
use std::io::{self, Write};
use std::time::Instant;
use x32_lib::{MixerClient, MixerModel};

/// Returns the maximum number of FX slots supported by the mixer model.
pub fn max_fx_slots_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 => 8,
        MixerModel::Wing => 16,
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
    }
}

/// Returns the maximum number of input channels supported by the mixer model.
pub fn max_channels_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::Wing => 40,
        MixerModel::X32 => 32,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

/// Set the delay time of an effects unit by tapping.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Digital mixer model (X32, Wing, XR18, XR16, XR12)
    #[arg(short = 'M', long, default_value = "X32")]
    pub model: MixerModel,

    /// The IP address of the mixer.
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    #[arg(long, default_value = "auto")]
    pub transport: String,

    #[arg(long, default_value = "")]
    pub usb_port: String,

    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    /// The FX slot number containing the delay effect.
    #[arg(short, long, default_value_t = 1)]
    pub slot: u8,

    /// Enable auto-tap mode by monitoring a channel's meter level.
    #[arg(short, long)]
    pub auto: bool,

    /// In auto mode, the channel number to monitor.
    #[arg(short, long, default_value_t = 1)]
    pub channel: u8,

    /// In auto mode, the gate meter threshold level (e.g. 0.5) to trigger a tap.
    #[arg(short = 't', long, default_value_t = 0.5)]
    pub threshold: f32,
}

// Stereo delay FX number (from C source): 10
// Other delay types from C source: 11, 12, 21, 24, 25, 26

/// The main entry point for the application.
pub async fn run(args: Args) -> Result<()> {
    let max_slots = max_fx_slots_for_model(args.model);
    if args.slot < 1 || args.slot > max_slots {
        return Err(anyhow!(
            "FX slot must be between 1 and {} for model {}.",
            max_slots,
            args.model
        ));
    }

    let max_ch = max_channels_for_model(args.model);
    if args.channel < 1 || args.channel > max_ch {
        return Err(anyhow!(
            "Channel must be between 1 and {} for model {}.",
            max_ch,
            args.model
        ));
    }

    let target_ip = args.model.resolve_target_address(&args.ip);
    println!("Connecting to {} at {}...", args.model, target_ip);
    let (client, _) = MixerClient::connect_with_transport(
        &target_ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    )
    .await
    .with_context(|| format!("Failed to connect to {}", args.model))?;
    let client = std::sync::Arc::new(client);
    println!("Connected!");

    println!("Checking FX slot {}...", args.slot);
    let mut fx_type = 0;
    if let Ok(OscArg::Int(t)) = client.query_value(&format!("/fx/{}/type", args.slot)).await {
        fx_type = t;
    }

    // List of known delay FX types from C source
    let valid_delays = [10, 11, 12, 21, 24, 25, 26];
    if !valid_delays.contains(&fx_type) {
        eprintln!(
            "Warning: FX slot {} does not appear to contain a standard delay effect (Type ID: {}).",
            args.slot, fx_type
        );
        eprintln!("Proceeding anyway, but commands may not work as expected.");
    } else {
        println!("Found valid delay effect (Type ID: {}).", fx_type);
    }

    if args.auto {
        println!("X32Tap - Auto Mode");
        println!(
            "Monitoring channel {} with threshold {}",
            args.channel, args.threshold
        );
        println!("Press Ctrl+C to quit.");

        let mut last_tap: Option<Instant> = None;
        let mut was_above_threshold = false;
        let mut last_keepalive = Instant::now() - std::time::Duration::from_secs(10);
        let param_idx = if fx_type == 10 { 2 } else { 1 };
        let address = format!("/fx/{}/par/{:02}", args.slot, param_idx);

        let mut rx = client.subscribe();

        loop {
            // Send keepalives and meter subscriptions every 9 seconds
            let now = Instant::now();
            if now.duration_since(last_keepalive).as_secs() >= 9 {
                // Keep the connection alive
                let _ = client.send_message("/xremote", vec![]).await;

                // Meter 6 subscription with channel index
                let _ = client
                    .send_message(
                        "/meters",
                        vec![
                            OscArg::String("/meters/6".to_string()),
                            OscArg::Int(0),
                            OscArg::Int(0),
                            OscArg::Int((args.channel - 1) as i32),
                        ],
                    )
                    .await;

                last_keepalive = now;
            }

            // Read incoming OSC messages
            if let Ok(Ok(msg)) =
                tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await
            {
                if msg.path == "/meters/6" {
                    if let Some(OscArg::Blob(data)) = msg.args.first() {
                        // A blob in /meters/6 contains 4 floats: 4 * 4 = 16 bytes.
                        if data.len() >= 16 {
                            let mut f_bytes = [0u8; 4];
                            if let Some(slice) = data.get(12..16) {
                                f_bytes.copy_from_slice(slice);
                            } else {
                                continue;
                            }
                            // X32 sends floats in Little Endian in blobs.
                            let level = f32::from_le_bytes(f_bytes);

                            if level > args.threshold {
                                if !was_above_threshold {
                                    let tap_time = Instant::now();
                                    if let Some(last) = last_tap {
                                        let delta = tap_time.duration_since(last);
                                        let delta_ms = delta.as_millis() as f32;

                                        // Minimum resolution is 60ms to avoid rapid-fire updates
                                        if delta_ms > 60.0 {
                                            let f_val = (delta_ms / 3000.0).clamp(0.0, 1.0);
                                            let tempo_ms = (f_val * 3000.0) as i32;
                                            println!(
                                                "Auto Tap: {}ms (level: {:.2})",
                                                tempo_ms, level
                                            );

                                            if let Err(e) = client
                                                .send_message(&address, vec![OscArg::Float(f_val)])
                                                .await
                                            {
                                                eprintln!("Failed to update FX parameter: {}", e);
                                            }
                                            last_tap = Some(tap_time);
                                        }
                                    } else {
                                        println!("First auto tap... (level: {:.2})", level);
                                        last_tap = Some(tap_time);
                                    }
                                    was_above_threshold = true;
                                }
                            } else {
                                was_above_threshold = false;
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("X32Tap - Manual Mode");
        println!("Press <Enter> repeatedly to set tempo.");
        println!("Type 'q' and <Enter> to quit.");

        let mut last_tap: Option<Instant> = None;
        let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
        use tokio::io::{AsyncBufReadExt, AsyncReadExt};
        let mut input_buffer = Vec::new();

        loop {
            print!("> ");
            io::stdout().flush()?;
            input_buffer.clear();

            // Limit the amount of bytes read to 4096 to prevent memory DoS on large inputs
            let mut take_reader = (&mut stdin).take(4096);
            let bytes_read = take_reader.read_until(b'\n', &mut input_buffer).await?;

            if bytes_read == 0 && input_buffer.is_empty() {
                // EOF reached
                break;
            }

            if bytes_read == 4096 && !input_buffer.ends_with(b"\n") {
                // Line too long, discard remainder
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Input line too long",
                )
                .into());
            }

            let input_str = match std::str::from_utf8(&input_buffer) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Invalid UTF-8 sequence in input, discarded.");
                    continue;
                }
            };

            let input = input_str.trim();

            if input.eq_ignore_ascii_case("q") {
                break;
            }

            // Treat empty line (just Enter) or any non-command input as a tap
            let now = Instant::now();

            if let Some(last) = last_tap {
                let delta = now.duration_since(last);
                let delta_ms = delta.as_millis() as f32;

                // Calculate parameter value (0.0 - 1.0 represents 0ms - 3000ms)
                let f_val = (delta_ms / 3000.0).clamp(0.0, 1.0);

                let tempo_ms = (f_val * 3000.0) as i32;
                println!("Tempo: {}ms", tempo_ms);

                let param_idx = if fx_type == 10 { 2 } else { 1 };
                let address = format!("/fx/{}/par/{:02}", args.slot, param_idx);

                if let Err(e) = client
                    .send_message(&address, vec![OscArg::Float(f_val)])
                    .await
                {
                    eprintln!("Failed to send OSC message: {}", e);
                }
            } else {
                println!("First tap...");
            }

            last_tap = Some(now);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_fx_slots_for_model() {
        assert_eq!(max_fx_slots_for_model(MixerModel::X32), 8);
        assert_eq!(max_fx_slots_for_model(MixerModel::Wing), 16);
        assert_eq!(max_fx_slots_for_model(MixerModel::XR18), 4);
        assert_eq!(max_fx_slots_for_model(MixerModel::XR16), 4);
        assert_eq!(max_fx_slots_for_model(MixerModel::XR12), 4);
    }

    #[test]
    fn test_max_channels_for_model() {
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);
    }

    #[test]
    fn test_args_default_model() {
        let args = Args::parse_from(&["x32_tap", "--ip", "192.168.1.10"]);
        assert_eq!(args.model, MixerModel::X32);
        assert_eq!(
            args.model.resolve_target_address(&args.ip),
            "192.168.1.10:10023"
        );
    }

    #[test]
    fn test_args_model_parsing() {
        let models = [
            ("X32", MixerModel::X32, "10023"),
            ("Wing", MixerModel::Wing, "2222"),
            ("XR18", MixerModel::XR18, "10024"),
            ("XR16", MixerModel::XR16, "10024"),
            ("XR12", MixerModel::XR12, "10024"),
        ];

        for (model_str, expected_model, expected_port) in models {
            let args_short = Args::parse_from(&["x32_tap", "-M", model_str, "--ip", "10.0.0.1"]);
            assert_eq!(args_short.model, expected_model);
            assert_eq!(
                args_short.model.resolve_target_address(&args_short.ip),
                format!("10.0.0.1:{}", expected_port)
            );

            let args_long =
                Args::parse_from(&["x32_tap", "--model", model_str, "--ip", "10.0.0.1"]);
            assert_eq!(args_long.model, expected_model);
        }
    }

    #[test]
    fn test_target_address_with_explicit_port() {
        let args = Args::parse_from(&["x32_tap", "-M", "Wing", "--ip", "192.168.1.55:9999"]);
        assert_eq!(args.model, MixerModel::Wing);
        assert_eq!(
            args.model.resolve_target_address(&args.ip),
            "192.168.1.55:9999"
        );
    }
}
