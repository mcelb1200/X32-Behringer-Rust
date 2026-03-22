//! `x32_tap` is a command-line tool for setting the tempo of X32 delay effects by tapping.
//!
//! It connects to the mixer and listens for user input (pressing Enter). By measuring the
//! time interval between taps, it calculates the tempo in milliseconds and updates the
//! corresponding parameter of the delay effect in the specified FX slot.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, Write};
use std::time::Instant;
use x32_lib::create_socket;

/// Set the delay time of an X32 effects unit by tapping.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 mixer.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

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
    #[arg(short = 't', long, default_value_t = 0.5)]
    threshold: f32,
}

// Stereo delay FX number (from C source): 10
// Other delay types from C source: 11, 12, 21, 24, 25, 26

/// The main entry point for the application.
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.slot < 1 || args.slot > 4 {
        return Err(anyhow!("FX slot must be between 1 and 4."));
    }

    if args.channel < 1 || args.channel > 32 {
        return Err(anyhow!("Channel must be between 1 and 32."));
    }

    println!("Connecting to X32 at {}...", args.ip);
    let std_socket = create_socket(&args.ip, 500).context("Failed to create socket")?;
    std_socket.set_nonblocking(true)?;
    let socket = tokio::net::UdpSocket::from_std(std_socket)?;

    // Check connection with /info
    let info_msg = OscMessage::new("/info".to_string(), vec![]);
    socket.send(&info_msg.to_bytes()?).await?;

    let mut buf = [0u8; 512];
    match tokio::time::timeout(std::time::Duration::from_millis(500), socket.recv(&mut buf)).await {
        Ok(Ok(_)) => println!("Connected!"),
        Ok(Err(e)) => return Err(anyhow!("Failed to connect to X32: {}", e)),
        Err(_) => return Err(anyhow!("Connection to X32 timed out. Is the IP address correct?")),
    }

    // We can't reuse get_fx_type easily because it takes std::net::UdpSocket.
    // So let's write our own async check.
    println!("Checking FX slot {}...", args.slot);
    let type_req = OscMessage::new(format!("/fx/{}/type", args.slot), vec![]);
    socket.send(&type_req.to_bytes()?).await?;
    let mut fx_type = 0;

    // Read response with timeout
    if let Ok(res) = tokio::time::timeout(std::time::Duration::from_millis(500), socket.recv(&mut buf)).await {
        if let Ok(len) = res {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                if msg.path == type_req.path {
                    if let Some(OscArg::Int(t)) = msg.args.first() {
                        fx_type = *t;
                    }
                }
            }
        }
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
        println!("Monitoring channel {} with threshold {}", args.channel, args.threshold);
        println!("Press Ctrl+C to quit.");

        let mut last_tap: Option<Instant> = None;
        let mut last_keepalive = Instant::now() - std::time::Duration::from_secs(10);
        let param_idx = if fx_type == 10 { 2 } else { 1 };
        let address = format!("/fx/{}/par/{:02}", args.slot, param_idx);

        loop {
            // Send keepalives every 9 seconds
            let now = Instant::now();
            if now.duration_since(last_keepalive).as_secs() >= 9 {
                // Keep the connection alive
                let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
                if let Err(e) = socket.send(&xremote_msg.to_bytes()?).await {
                    eprintln!("Failed to send /xremote: {}", e);
                }

                // Meter 6 subscription with channel index in format. Wait, C code uses:
                // "/meters\0,siii\0\0\0/meters/6\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
                // followed by injecting the channel index at offset 31 (the 3rd int).
                // Let's use osc_lib: path: "/meters", args: String("/meters/6"), Int(0), Int(0), Int(args.channel - 1), Int(0)
                // Actually the C format is ,siii: string, int, int, int.
                let meter_req = OscMessage::new(
                    "/meters".to_string(),
                    vec![
                        OscArg::String("/meters/6".to_string()),
                        OscArg::Int(0),
                        OscArg::Int(0),
                        OscArg::Int((args.channel - 1) as i32),
                    ]
                );
                if let Err(e) = socket.send(&meter_req.to_bytes()?).await {
                    eprintln!("Failed to send /meters request: {}", e);
                }

                last_keepalive = now;
            }

            // Read UDP packets
            if let Ok(Ok(len)) = tokio::time::timeout(std::time::Duration::from_millis(100), socket.recv(&mut buf)).await {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.path == "/meters/6" {
                        if let Some(OscArg::Blob(data)) = msg.args.first() {
                            // C code reads float at offset 12 of the data (which is byte 28 from start of packet).
                            // A blob in /meters/6 contains 4 floats: 4 * 4 = 16 bytes.
                            if data.len() >= 16 {
                                let mut f_bytes = [0u8; 4];
                                // Rust OSC blobs usually come out as raw bytes. The float at offset 12:
                                f_bytes.copy_from_slice(&data[12..16]);
                                // X32 sends floats in Little Endian in blobs.
                                let level = f32::from_le_bytes(f_bytes);

                                if level > args.threshold {
                                    let tap_time = Instant::now();
                                    if let Some(last) = last_tap {
                                        let delta = tap_time.duration_since(last);
                                        let delta_ms = delta.as_millis() as f32;

                                        // Minimum resolution is 60ms to avoid rapid-fire updates
                                        if delta_ms > 60.0 {
                                            let f_val = (delta_ms / 3000.0).clamp(0.0, 1.0);
                                            let tempo_ms = (f_val * 3000.0) as i32;
                                            println!("Auto Tap: {}ms (level: {:.2})", tempo_ms, level);

                                            let update_msg = OscMessage::new(address.clone(), vec![OscArg::Float(f_val)]);
                                            if let Err(e) = socket.send(&update_msg.to_bytes()?).await {
                                                eprintln!("Failed to update FX parameter: {}", e);
                                            }
                                            last_tap = Some(tap_time);
                                        }
                                    } else {
                                        println!("First auto tap... (level: {:.2})", level);
                                        last_tap = Some(tap_time);
                                    }
                                }
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
        use tokio::io::AsyncBufReadExt;
        let mut input_buffer = String::new();

        loop {
            print!("> ");
            io::stdout().flush()?;
            input_buffer.clear();
            stdin.read_line(&mut input_buffer).await?;

            let input = input_buffer.trim();

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

                let msg = OscMessage::new(address, vec![OscArg::Float(f_val)]);
                if let Err(e) = socket.send(&msg.to_bytes()?).await {
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
