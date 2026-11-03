//! `x32_set_scene` is a command-line tool for sending a sequence of OSC commands to a Behringer X32 mixer.
//!
//! It reads OSC commands from standard input (one per line) and sends them to the mixer.
//! This is typically used to restore a scene or apply a batch of settings.

use clap::Parser;
use osc_lib::OscMessage;
use std::io::{self, BufRead, Read};
use std::str::FromStr;
use std::time::Duration;
use x32_lib::{MixerClient, MixerModel, error::Result};

/// Command-line arguments for `x32_set_scene`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The IP address of the console.
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(
        short = 'M',
        short_alias = 'm',
        long,
        default_value = "X32",
        help = "Mixer model: X32, Wing, XR18, XR16, XR12"
    )]
    pub model: MixerModel,

    /// Transport type: auto, osc, aes50, usb
    #[arg(long, default_value = "auto")]
    pub transport: String,

    /// USB port name for USB MIDI transport
    #[arg(long, default_value = "")]
    pub usb_port: String,

    /// AES50 IP address for tunneled OSC transport
    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    /// Delay between commands in milliseconds.
    #[arg(short, long, default_value_t = 1)]
    pub delay: u64,
}

/// Resolves the target IP and port based on the provided IP string and mixer model.
pub fn resolve_target_address(ip: &str, model: MixerModel) -> String {
    if ip.contains(':') {
        ip.to_string()
    } else {
        let port = match model {
            MixerModel::X32 => 10023,
            MixerModel::Wing => 2222,
            MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 10024,
        };
        format!("{}:{}", ip, port)
    }
}

/// Validates whether an OSC node path is supported and within index bounds for the given mixer model.
pub fn is_path_valid_for_model(model: MixerModel, path: &str) -> bool {
    let clean_path = path.trim();
    if !clean_path.starts_with('/') {
        return false;
    }

    let parts: Vec<&str> = clean_path.trim_start_matches('/').split('/').collect();
    if parts.is_empty() {
        return false;
    }

    let (max_ch, max_bus, max_aux, max_mtx, max_dca, max_fx) = match model {
        MixerModel::X32 => (32, 16, 8, 6, 8, 8),
        MixerModel::Wing => (40, 28, 16, 8, 16, 16),
        MixerModel::XR18 => (16, 6, 2, 0, 4, 4),
        MixerModel::XR16 => (16, 4, 2, 0, 4, 4),
        MixerModel::XR12 => (12, 2, 2, 0, 4, 4),
    };

    match parts[0] {
        "ch" => {
            if parts.len() >= 2 {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    return idx >= 1 && idx <= max_ch;
                }
            }
            true
        }
        "bus" => {
            if parts.len() >= 2 {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    return idx >= 1 && idx <= max_bus;
                }
            }
            true
        }
        "auxin" | "aux" => {
            if parts.len() >= 2 {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    return idx >= 1 && idx <= max_aux;
                }
            }
            true
        }
        "mtx" => {
            if max_mtx == 0 {
                return false;
            }
            if parts.len() >= 2 {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    return idx >= 1 && idx <= max_mtx;
                }
            }
            true
        }
        "dca" => {
            if parts.len() >= 2 {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    return idx >= 1 && idx <= max_dca;
                }
            }
            true
        }
        "fx" => {
            if parts.len() >= 2 {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    return idx >= 1 && idx <= max_fx;
                }
            }
            true
        }
        "main" => {
            if parts.len() >= 2 && parts[1] == "m" {
                return matches!(model, MixerModel::X32 | MixerModel::Wing);
            }
            true
        }
        _ => true,
    }
}

/// The main execution function for `x32_set_scene`.
pub async fn run(args: Args) -> Result<()> {
    let target_ip = resolve_target_address(&args.ip, args.model);

    let (client, _transport) = MixerClient::connect_with_transport(
        &target_ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    )
    .await?;
    let client = std::sync::Arc::new(client);

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut parser = x32_lib::scene_parse::SceneParser::with_model(args.model);
    loop {
        let mut byte_buf = Vec::new();
        let mut handle = stdin_lock.by_ref().take(4096);
        match handle.read_until(b'\n', &mut byte_buf) {
            Ok(0) => break,                 // EOF
            Err(e) => return Err(e.into()), // Propagate I/O errors properly
            Ok(len) => {
                if len == 4096 && !byte_buf.ends_with(b"\n") {
                    // Line too long, discard remainder
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Input line too long",
                    )
                    .into());
                }
            }
        }

        let line_str = match std::str::from_utf8(&byte_buf) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Invalid UTF-8 sequence in input, discarded.");
                continue;
            }
        };

        let line = line_str.trim();
        if line.starts_with('/') {
            if !is_path_valid_for_model(args.model, line) {
                continue;
            }

            let mut messages = parser.parse_scene_line(line);

            if messages.is_empty() {
                match OscMessage::from_str(line) {
                    Ok(msg) => messages.push(msg),
                    Err(e) => eprintln!("Error parsing line: {} - {}", line, e),
                }
            }

            for msg in messages {
                if !is_path_valid_for_model(args.model, &msg.path) {
                    continue;
                }
                client.send_message(&msg.path, msg.args).await?;
                if args.delay > 0 {
                    tokio::time::sleep(Duration::from_millis(args.delay)).await;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target_address() {
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::X32),
            "192.168.1.50:10023"
        );
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::Wing),
            "192.168.1.50:2222"
        );
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::XR18),
            "192.168.1.50:10024"
        );
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::XR16),
            "192.168.1.50:10024"
        );
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::XR12),
            "192.168.1.50:10024"
        );
        assert_eq!(
            resolve_target_address("10.0.0.1:10000", MixerModel::X32),
            "10.0.0.1:10000"
        );
    }

    #[test]
    fn test_is_path_valid_for_model_channels() {
        assert!(is_path_valid_for_model(MixerModel::X32, "/ch/01/mix/fader"));
        assert!(is_path_valid_for_model(MixerModel::X32, "/ch/32/mix/fader"));
        assert!(!is_path_valid_for_model(
            MixerModel::X32,
            "/ch/33/mix/fader"
        ));

        assert!(is_path_valid_for_model(
            MixerModel::Wing,
            "/ch/40/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::Wing,
            "/ch/41/mix/fader"
        ));

        assert!(is_path_valid_for_model(
            MixerModel::XR18,
            "/ch/16/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::XR18,
            "/ch/17/mix/fader"
        ));

        assert!(is_path_valid_for_model(
            MixerModel::XR12,
            "/ch/12/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::XR12,
            "/ch/13/mix/fader"
        ));
    }

    #[test]
    fn test_is_path_valid_for_model_buses_and_matrices() {
        assert!(is_path_valid_for_model(
            MixerModel::X32,
            "/bus/16/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::X32,
            "/bus/17/mix/fader"
        ));

        assert!(is_path_valid_for_model(
            MixerModel::Wing,
            "/bus/28/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::Wing,
            "/bus/29/mix/fader"
        ));

        assert!(is_path_valid_for_model(
            MixerModel::XR18,
            "/bus/06/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::XR18,
            "/bus/07/mix/fader"
        ));

        assert!(is_path_valid_for_model(
            MixerModel::XR12,
            "/bus/02/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::XR12,
            "/bus/03/mix/fader"
        ));

        // Matrices
        assert!(is_path_valid_for_model(
            MixerModel::X32,
            "/mtx/06/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::XR18,
            "/mtx/01/mix/fader"
        ));
    }

    #[test]
    fn test_args_parsing() {
        let args = Args::try_parse_from([
            "x32_set_scene",
            "-i",
            "192.168.1.100",
            "-M",
            "Wing",
            "-d",
            "5",
        ])
        .unwrap();

        assert_eq!(args.ip, "192.168.1.100");
        assert_eq!(args.model, MixerModel::Wing);
        assert_eq!(args.delay, 5);
    }
}
