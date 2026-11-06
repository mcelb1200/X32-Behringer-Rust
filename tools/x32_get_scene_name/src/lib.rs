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
use osc_lib::OscArg;
use std::time::Duration;
use tokio::time::{self, Instant};
use x32_lib::{MixerClient, MixerModel, error::X32Error};

/// A command line utility to get scene names when a scene change takes place.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Digital mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short = 'M', long, default_value = "X32")]
    pub model: MixerModel,

    /// Console IP address
    #[arg(short, long, default_value = "192.168.1.62")]
    pub ip: String,

    #[arg(long, default_value = "auto")]
    pub transport: String,

    #[arg(long, default_value = "")]
    pub usb_port: String,

    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    /// Prints welcome and connection status messages (0 or 1)
    #[arg(short, long, default_value_t = 1)]
    pub verbose: u8,

    /// Exits at first occurrence (0 or 1)
    #[arg(short, long, default_value_t = 1)]
    pub onetime: u8,
}

pub async fn run(args: Args) -> Result<(), X32Error> {
    let target_ip = args.model.resolve_target_address(&args.ip);

    if args.verbose != 0 {
        println!("GetSceneName - v0.2 - (c)2018 Patrick-Gilles Maillot");
        println!("Connecting to {} at {}...", args.model, &target_ip);
    }

    let (client, _transport_used) = MixerClient::connect_with_transport(
        &target_ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        true, // needs heartbeat for /xremote
    )
    .await?;
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
        println!("Timeout connecting to {}", args.model);
    }

    // Subscribe to all incoming OSC messages
    let mut rx = client.subscribe();
    let mut last_show_control_sent = Instant::now();

    // Initial show control send
    client
        .send_message("/-prefs/show_control", vec![OscArg::Int(1)])
        .await?;

    loop {
        // We still need to send /-prefs/show_control occasionally, maybe every 9s alongside the heartbeat which client does automatically for /xremote
        if last_show_control_sent.elapsed() >= Duration::from_secs(9) {
            let _ = client
                .send_message("/-prefs/show_control", vec![OscArg::Int(1)])
                .await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default_model() {
        let args = Args::parse_from(&["x32_get_scene_name", "--ip", "192.168.1.10"]);
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
            let args_short =
                Args::parse_from(&["x32_get_scene_name", "-M", model_str, "--ip", "10.0.0.1"]);
            assert_eq!(args_short.model, expected_model);
            assert_eq!(
                args_short.model.resolve_target_address(&args_short.ip),
                format!("10.0.0.1:{}", expected_port)
            );

            let args_long = Args::parse_from(&[
                "x32_get_scene_name",
                "--model",
                model_str,
                "--ip",
                "10.0.0.1",
            ]);
            assert_eq!(args_long.model, expected_model);
        }
    }

    #[test]
    fn test_target_address_with_explicit_port() {
        let args = Args::parse_from(&[
            "x32_get_scene_name",
            "-M",
            "Wing",
            "--ip",
            "192.168.1.55:9999",
        ]);
        assert_eq!(args.model, MixerModel::Wing);
        assert_eq!(
            args.model.resolve_target_address(&args.ip),
            "192.168.1.55:9999"
        );
    }
}
