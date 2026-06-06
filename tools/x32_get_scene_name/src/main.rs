//! `x32_get_scene_name` is a command-line utility that connects to a Behringer X32 digital
//! mixer, listens for scene change events, and prints the name of the new scene to standard
//! output. It is a Rust implementation of the original `GetSceneName.c` tool by
//! Patrick-gilles Maillot.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use clap::Parser;
use osc_lib::{OscArg, };
use std::time::Duration;
use tokio::time::{self, Instant};
use x32_lib::{MixerClient, error::X32Error};

/// A command line utility to get scene names when a scene change takes place.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// X32 console IP address
    #[arg(short, long, default_value = "192.168.1.62")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    /// Prints welcome and connection status messages (0 or 1)
    #[arg(short, long, default_value_t = 1)]
    verbose: u8,

    /// Exits at first occurrence (0 or 1)
    #[arg(short, long, default_value_t = 1)]
    onetime: u8,
}

#[tokio::main]
async fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    if args.verbose != 0 {
        println!("GetSceneName - v0.2 - (c)2018 Patrick-Gilles Maillot");
        println!("Connecting to X32 at {}...", &args.ip);
    }

    let (client, _transport_used) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        true, // needs heartbeat for /xremote
    ).await?;
    let client = std::sync::Arc::new(client);

    // Initial connection validation via /info
    let mut rx = client.subscribe();
    client.send_message("/info", vec![]).await?;

    let timeout = Duration::from_secs(2);
    let start = Instant::now();
    let mut connected = false;

    while start.elapsed() < timeout {
        match time::timeout(timeout - start.elapsed(), rx.recv()).await {
            Ok(Ok(msg)) => {
                if msg.path == "/info" {
                    connected = true;
                    if args.verbose != 0 {
                        println!("Connected!");
                    }
                    break;
                }
            }
            _ => {
                client.send_message("/info", vec![]).await?;
                if args.verbose != 0 {
                    print!(".");
                }
            }
        }
    }

    if !connected && args.verbose != 0 {
        println!("Timeout connecting to X32");
    }

    // Subscribe to all incoming OSC messages
    let mut rx = client.subscribe();
    let mut last_show_control_sent = Instant::now();

    // Initial show control send
    client.send_message("/-prefs/show_control", vec![OscArg::Int(1)]).await?;

    loop {
        // We still need to send /-prefs/show_control occasionally, maybe every 9s alongside the heartbeat which client does automatically for /xremote
        if last_show_control_sent.elapsed() >= Duration::from_secs(9) {
            let _ = client.send_message("/-prefs/show_control", vec![OscArg::Int(1)]).await;
            last_show_control_sent = Instant::now();
        }

        // We use timeout to ensure we can loop to send the show_control heartbeat
        match time::timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Ok(msg)) => {
                if msg.path == "/-show/prepos/current" {
                    if let Some(OscArg::Int(scene_index)) = msg.args.first() {
                        let path = format!("/-show/showfile/scene/{:03}", scene_index);
                        let _ = client.send_message(&path, vec![]).await;
                    }
                } else if msg.path.starts_with("/-show/showfile/scene") {
                    if let Some(OscArg::String(scene_name)) = msg.args.first() {
                        if let Some(OscArg::Int(scene_index)) = msg.args.get(1) {
                            println!("{:02} - {}", scene_index, scene_name);
                            if args.onetime != 0 {
                                break;
                            }
                        }
                    }
                }
            }
            _ => continue,
        }
    }

    Ok(())
}
