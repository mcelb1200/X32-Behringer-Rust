//! # x32_tap
//!
//! A command-line tool for setting the tempo of delay effects on a Behringer X32/M32 mixer.
//!
//! Provides two modes:
//! - `manual`: Manually tap the tempo by pressing the Enter key.
//! - `auto`: Automatically detect the tempo from a channel's audio signal.

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use x32_lib::create_socket;
use x32_lib::fx::FxType1;
use osc_lib::{OscMessage, OscArg};

use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::io::{stdin, BufRead};

/// Command-line arguments for the x32_tap tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Defines the subcommands for the x32_tap tool.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Manually tap the tempo
    Manual {
        /// IP address of the X32 mixer
        #[arg(short, long)]
        ip: String,
        /// FX slot number (1-4)
        #[arg(short, long)]
        slot: u8,
    },
    /// Automatically detect the tempo from a channel's audio (equivalent to X32TapW.c)
    Auto {
        /// IP address of the X32 mixer
        #[arg(short, long)]
        ip: String,
        /// FX slot number (1-4)
        #[arg(short, long)]
        slot: u8,
        /// Channel number to monitor
        #[arg(short, long)]
        channel: u8,
        /// Sensitivity threshold (0.0 - 1.0)
        #[arg(short = 'e', long, default_value_t = 0.5)]
        sensitivity: f32,
    },
}

/// Checks if a given `FxType1` is a delay-based effect compatible with tap tempo.
fn is_delay_fx(fx_type: FxType1) -> bool {
    matches!(
        fx_type,
        FxType1::Dly |
        FxType1::Tap3 |
        FxType1::Tap4 |
        FxType1::DRv |
        FxType1::CrR |
        FxType1::FlR |
        FxType1::DCr |
        FxType1::DFl |
        FxType1::Modd
    )
}

/// Handles the manual tap tempo mode.
///
/// Reads lines from standard input and calculates the time between each press of the Enter key.
/// Sends an OSC message to the mixer to update the delay time.
fn manual_tap(socket: &UdpSocket, slot: u8) -> Result<()> {
    let mut last_tap = Instant::now();
    let stdin = stdin();
    println!("Press ENTER to tap tempo...");
    for _ in stdin.lock().lines() {
        let now = Instant::now();
        let delta = now.duration_since(last_tap);
        last_tap = now;

        let tempo_ms = delta.as_millis();
        let tempo_float = tempo_ms as f32 / 3000.0;

        let tempo_float = tempo_float.max(0.0).min(1.0);

        let address = format!("/fx/{}/par/02", slot);
        let msg = OscMessage::new(address, vec![OscArg::Float(tempo_float)]);
        socket.send(&msg.to_bytes()?)?;
        println!("Tempo: {}ms", tempo_ms);
    }
    Ok(())
}

/// Handles the automatic tap tempo mode.
///
/// Subscribes to meter data for a specific channel and triggers a tap when the signal level
/// exceeds a given threshold.
fn auto_tap(socket: &UdpSocket, slot: u8, channel: u8, sensitivity: f32) -> Result<()> {
    let mut last_tap = Instant::now();
    let mut last_level = 0.0;

    // Subscribe to meter data for the specified channel
    let subscribe_msg = OscMessage::new("/meters".to_string(), vec![
        OscArg::String("/meters/6".to_string()),
        OscArg::Int(channel as i32 - 1),
        OscArg::Int(0),
        OscArg::Int(0),
    ]);
    socket.send(&subscribe_msg.to_bytes()?)?;

    loop {
        let mut buf = [0u8; 1024];
        match socket.recv(&mut buf) {
            Ok(len) => {
                if len == 0 {
                    break;
                }
                let msg = OscMessage::from_bytes(&buf[..len])?;
                if msg.path == "/meters/6" {
                    if let Some(OscArg::Blob(data)) = msg.args.get(0) {
                        // The gate meter is the second float in the blob
                        if data.len() >= 8 {
                            let mut bytes = [0u8; 4];
                            bytes.copy_from_slice(&data[4..8]);
                            let level = f32::from_be_bytes(bytes);

                            if level > sensitivity && last_level <= sensitivity {
                                let now = Instant::now();
                                let delta = now.duration_since(last_tap);
                                last_tap = now;

                                let tempo_ms = delta.as_millis();
                                if tempo_ms > 60 { // Debounce
                                    let tempo_float = tempo_ms as f32 / 3000.0;
                                    let tempo_float = tempo_float.max(0.0).min(1.0);

                                    let address = format!("/fx/{}/par/02", slot);
                                    let set_tempo_msg = OscMessage::new(address, vec![OscArg::Float(tempo_float)]);
                                    socket.send(&set_tempo_msg.to_bytes()?)?;
                                    println!("Tempo: {}ms", tempo_ms);
                                }
                            }
                            last_level = level;
                        }
                    }
                }
            }
            Err(_) => {} // Ignore timeouts
        }
    }
    Ok(())
}

/// Main entry point for the application.
fn main() -> Result<()> {
    let cli = Cli::parse();

    let (ip, slot) = match &cli.command {
        Commands::Manual { ip, slot } => (ip, slot),
        Commands::Auto { ip, slot, .. } => (ip, slot),
    };

    if *slot < 1 || *slot > 4 {
        return Err(anyhow!("Invalid FX slot number. Must be between 1 and 4."));
    }

    println!("Connecting to X32 at {}...", ip);
    let socket = create_socket(ip, 1000)?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    // Verify that the selected FX slot contains a compatible delay effect
    let address = format!("/fx/{}/type", slot);
    let msg = OscMessage::new(address.clone(), vec![]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    match socket.recv_from(&mut buf) {
        Ok((len, _)) => {
            let reply = OscMessage::from_bytes(&buf[..len])?;

            if reply.path == address {
                if let Some(OscArg::Int(fx_id)) = reply.args.get(0) {
                    if let Some(fx_type) = FxType1::from_id(*fx_id as u8) {
                        if is_delay_fx(fx_type) {
                             println!("Successfully connected. Found compatible delay effect in FX slot {}.", slot);
                        } else {
                            return Err(anyhow!("FX slot {} does not contain a compatible delay effect.", slot));
                        }
                    } else {
                        return Err(anyhow!("Unknown effect type in FX slot {}.", slot));
                    }
                }
            } else {
                return Err(anyhow!("Unexpected reply from mixer."));
            }
        },
        Err(_) => {
            println!("No response from mixer. Assuming FX slot is correct.");
        }
    }


    match &cli.command {
        Commands::Manual { slot, .. } => {
            println!("Entering manual tap mode. Press Enter to tap the tempo. Press Ctrl+C to exit.");
            manual_tap(&socket, *slot)?;
        }
        Commands::Auto { slot, channel, sensitivity, .. } => {
            println!("Entering auto tap mode for channel {} with sensitivity {}. Press Ctrl+C to exit.", channel, sensitivity);
            auto_tap(&socket, *slot, *channel, *sensitivity)?;
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::net::UdpSocket;
    use std::thread;

    fn mock_server(address: &str) -> UdpSocket {
        let socket = UdpSocket::bind(address).unwrap();
        let mut buf = [0; 1024];
        loop {
            let (amt, src) = socket.recv_from(&mut buf).unwrap();
            if amt == 0 {
                break;
            }
            let msg = OscMessage::from_bytes(&buf[..amt]).unwrap();
            if msg.path.ends_with("/type") {
                let reply = OscMessage::new(
                    msg.path.clone(),
                    vec![OscArg::Int(FxType1::Dly as i32)],
                );
                socket.send_to(&reply.to_bytes().unwrap(), src).unwrap();
            }
        }
        socket
    }

    #[test]
    fn test_is_delay_fx() {
        assert!(is_delay_fx(FxType1::Dly));
        assert!(is_delay_fx(FxType1::Tap3));
        assert!(is_delay_fx(FxType1::Tap4));
        assert!(is_delay_fx(FxType1::DRv));
        assert!(is_delay_fx(FxType1::CrR));
        assert!(is_delay_fx(FxType1::FlR));
        assert!(is_delay_fx(FxType1::DCr));
        assert!(is_delay_fx(FxType1::DFl));
        assert!(is_delay_fx(FxType1::Modd));
        assert!(!is_delay_fx(FxType1::Hall));
        assert!(!is_delay_fx(FxType1::Gate));
    }

    #[test]
    fn test_cli_manual() {
        let server_addr = "127.0.0.1:10025";
        let server_thread = thread::spawn(move || mock_server(server_addr));

        let mut cmd = Command::cargo_bin("x32_tap").unwrap();
        cmd.arg("manual")
            .arg("--ip")
            .arg(server_addr)
            .arg("--slot")
            .arg("1")
            .timeout(Duration::from_secs(1))
            .assert()
            .stdout(predicate::str::contains("Entering manual tap mode"));

        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.send_to(&[], server_addr).unwrap();
        server_thread.join().unwrap();
    }

    #[test]
    fn test_cli_auto() {
        let server_addr = "127.0.0.1:10026";
        let server_thread = thread::spawn(move || mock_server(server_addr));

        let mut cmd = Command::cargo_bin("x32_tap").unwrap();
        cmd.arg("auto")
            .arg("--ip")
            .arg(server_addr)
            .arg("--slot")
            .arg("2")
            .arg("--channel")
            .arg("5")
            .arg("-e")
            .arg("0.8")
            .timeout(Duration::from_secs(1))
            .assert()
            .stdout(predicate::str::contains("Entering auto tap mode for channel 5 with sensitivity 0.8"));

        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.send_to(&[], server_addr).unwrap();
        server_thread.join().unwrap();
    }
}
