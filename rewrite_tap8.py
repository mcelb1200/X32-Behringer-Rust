with open("tools/x32_tap/src/main.rs", "r") as f:
    original = f.read()

out = """//! `x32_tap` is a command-line tool for setting the tempo of X32 delay effects by tapping.
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

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, Write};
use std::time::Instant;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tokio::io::{AsyncBufReadExt, BufReader};
use x32_lib::MixerClient;

/// Set the delay time of an X32 effects unit by tapping.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    /// The FX slot number (1-4) containing the delay effect.
    #[arg(short, long, default_value_t = 1)]
    slot: u8,

    /// Enable auto-tap mode by monitoring a channel's meter level.
    #[arg(short, long)]
    auto: bool,

    /// In auto mode, the channel number to monitor (1-32).
    #[arg(short, long, default_value_t = 1)]
    channel: u8,

    /// In auto mode, the gate meter threshold level (e.g. 0.5) to trigger a tap.
    #[arg(short, long, default_value_t = 0.5)]
    threshold: f32,

    /// The parameter index to update (usually 1 or 2 depending on the delay type).
    /// Typically 1 for most delays, 2 for Stereo Delay.
    #[arg(short, long, default_value_t = 1)]
    parameter: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.slot < 1 || args.slot > 4 {
        return Err(anyhow!("FX slot must be between 1 and 4."));
    }

    if args.channel < 1 || args.channel > 32 {
        return Err(anyhow!("Channel must be between 1 and 32."));
    }

    if args.threshold <= 0.0 || args.threshold >= 1.0 {
        return Err(anyhow!("Threshold must be between 0.0 and 1.0."));
    }

    println!("Connecting to X32 at {}...", args.ip);
    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await.context("Failed to connect to X32")?;
    let client = Arc::new(client);

    let mut rx = client.subscribe();

    let info_msg = OscMessage::new("/info".to_string(), vec![]);
    client.send_message(&info_msg.path, info_msg.args).await?;

    match timeout(Duration::from_millis(500), rx.recv()).await {
        Ok(Ok(_)) => println!("Connected!"),
        Ok(Err(e)) => return Err(anyhow!("Failed to connect to X32: {}", e)),
        Err(_) => println!("Warning: Did not receive response to /info. Proceeding anyway."),
    }

    handle_client(client, args).await?;

    Ok(())
}

async fn handle_client(client: Arc<MixerClient>, args: Args) -> Result<()> {
    let mut rx = client.subscribe();
    let mut fx_type = 0;

    let mut type_req = OscMessage::new(
        format!("/fx/{}/type", args.slot),
        vec![OscArg::String(format!("/fx/{}/type", args.slot))],
    );
    client.send_message(&type_req.path, type_req.args.clone()).await?;

    if let Ok(Ok(msg)) = timeout(Duration::from_millis(500), rx.recv()).await {
        if msg.path == type_req.path {
            if let Some(OscArg::Int(t)) = msg.args.first() {
                fx_type = *t as i32;
            }
        }
    }

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
        let mut last_keepalive = Instant::now() - Duration::from_secs(10);
        let param_idx = if fx_type == 10 { 2 } else { 1 };
        let address = format!("/fx/{}/par/{:02}", args.slot, param_idx);

        loop {
            let now = Instant::now();
            if now.duration_since(last_keepalive).as_secs() >= 9 {
                let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
                if let Err(e) = client.send_message(&xremote_msg.path, xremote_msg.args).await {
                    eprintln!("Failed to send /xremote: {}", e);
                }

                let meter_req = OscMessage::new(
                    "/meters".to_string(),
                    vec![
                        OscArg::String("/meters/6".to_string()),
                        OscArg::Int(0),
                        OscArg::Int(0),
                        OscArg::Int((args.channel - 1) as i32),
                    ],
                );
                if let Err(e) = client.send_message(&meter_req.path, meter_req.args).await {
                    eprintln!("Failed to send /meters request: {}", e);
                }

                last_keepalive = now;
            }

            if let Ok(Ok(msg)) = timeout(Duration::from_millis(100), rx.recv()).await {
                if msg.path == "/meters/6" {
                    if let Some(OscArg::Blob(data)) = msg.args.first() {
                        if data.len() >= 16 {
                            let mut f_bytes = [0u8; 4];
                            if let Some(slice) = data.get(12..16) {
                                f_bytes.copy_from_slice(slice);
                            } else {
                                continue;
                            }
                            let level = f32::from_le_bytes(f_bytes);

                            if level > args.threshold {
                                if !was_above_threshold {
                                    let tap_time = Instant::now();
                                    if let Some(last) = last_tap {
                                        let delta = tap_time.duration_since(last);
                                        let delta_ms = delta.as_millis() as f32;

                                        if delta_ms > 60.0 {
                                            let f_val = (delta_ms / 3000.0).clamp(0.0, 1.0);
                                            let tempo_ms = (f_val * 3000.0) as i32;
                                            println!(
                                                "Auto Tap: {}ms (level: {:.2})",
                                                tempo_ms, level
                                            );

                                            let update_msg = OscMessage::new(
                                                address.clone(),
                                                vec![OscArg::Float(f_val)],
                                            );
                                            if let Err(e) = client.send_message(&update_msg.path, update_msg.args).await {
                                                eprintln!(
                                                    "Failed to update FX parameter: {}",
                                                    e
                                                );
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
        let mut stdin = BufReader::new(tokio::io::stdin());
        let mut input_buffer = Vec::new();

        loop {
            print!("> ");
            io::stdout().flush()?;
            input_buffer.clear();

            // wait for stdin line without blocking the thread? Since this is Manual Mode, we don't care about background task. tokio handles it
            let mut take_reader = (&mut stdin).take(4096);
            let bytes_read = take_reader.read_until(b'\\n', &mut input_buffer).await?;

            if bytes_read == 0 && input_buffer.is_empty() {
                break;
            }

            if bytes_read == 4096 && !input_buffer.ends_with(b"\\n") {
                let mut discard = Vec::with_capacity(1024);
                loop {
                    discard.clear();
                    let mut chunk_handle = (&mut stdin).take(1024);
                    match chunk_handle.read_until(b'\\n', &mut discard).await {
                        Ok(0) => break,
                        Err(_) => break,
                        Ok(_) => {
                            if discard.ends_with(b"\\n") {
                                break;
                            }
                        }
                    }
                }
                eprintln!("Input line too long, discarded.");
                continue;
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

            let now = Instant::now();

            if let Some(last) = last_tap {
                let delta = now.duration_since(last);
                let delta_ms = delta.as_millis() as f32;

                let f_val = (delta_ms / 3000.0).clamp(0.0, 1.0);

                let tempo_ms = (f_val * 3000.0) as i32;
                println!("Tempo: {}ms", tempo_ms);

                let param_idx = if fx_type == 10 { 2 } else { 1 };
                let address = format!("/fx/{}/par/{:02}", args.slot, param_idx);

                let msg = OscMessage::new(address, vec![OscArg::Float(f_val)]);
                if let Err(e) = client.send_message(&msg.path, msg.args).await {
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
"""
with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(out)
