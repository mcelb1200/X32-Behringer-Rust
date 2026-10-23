//! `x32_get_scene` is a command-line tool for retrieving scene data from a Behringer X32 mixer.
//!
//! It reads a list of OSC addresses from standard input (one per line), queries the mixer
//! for their current values using the `/node` command, and prints the results to standard
//! output in a format suitable for saving as a scene file.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Original C code for XAir version:** Ken Mitchell
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use clap::Parser;
use osc_lib::OscArg;
use std::io::{self, BufRead, Read};
use tokio::time::Duration;
use x32_lib::{MixerClient, MixerModel, error::Result};

/// Command-line arguments for `x32_get_scene`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The IP address of the console.
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(
        short = 'M',
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

    /// Scene name.
    #[arg(short, long)]
    pub scene_name: Option<String>,

    /// Note data.
    #[arg(short, long)]
    pub note: Option<String>,
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

/// Formats the model-specific scene file header.
pub fn format_scene_header(model: MixerModel, scene_name: &str, note: &str) -> String {
    match model {
        MixerModel::X32 => {
            format!(
                "#2.7# \"{}\" \"{}\" %000000000 1 X32GetScene V1.5 (c)2014 Patrick-Gilles Maillot",
                scene_name, note
            )
        }
        MixerModel::Wing => {
            format!(
                "#3.0# \"{}\" \"{}\" %000000000 1 WingGetScene V1.0 (c)2024 Patrick-Gilles Maillot",
                scene_name, note
            )
        }
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => {
            format!(
                "#2.1# \"{}\" \"{}\" %000000000 1 XAirGetScene V1.4 (c)2014 Patrick-Gilles Maillot",
                scene_name, note
            )
        }
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

/// The main entry point for the application.
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

    let scene_name = match args.scene_name {
        Some(name) => name,
        None => {
            println!("Please enter scene name: ");
            let mut buf = Vec::new();
            io::stdin().lock().take(4096).read_until(b'\n', &mut buf)?;
            std::str::from_utf8(&buf).unwrap_or("").trim().to_string()
        }
    };

    let note = match args.note {
        Some(note) => note,
        None => {
            println!("Please enter note data: ");
            let mut buf = Vec::new();
            io::stdin().lock().take(4096).read_until(b'\n', &mut buf)?;
            std::str::from_utf8(&buf).unwrap_or("").trim().to_string()
        }
    };

    let header = format_scene_header(args.model, &scene_name, &note);
    println!("{}\n", header);

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    loop {
        let mut byte_buf = Vec::new();
        let mut handle = stdin_lock.by_ref().take(4096);
        match handle.read_until(b'\n', &mut byte_buf) {
            Ok(0) => break,                 // EOF
            Err(e) => return Err(e.into()), // Propagate I/O errors properly
            Ok(len) => {
                if len == 4096 && !byte_buf.ends_with(b"\n") {
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
                eprintln!(
                    "Node path '{}' is unsupported/out-of-bounds for model {}, skipping.",
                    line, args.model
                );
                continue;
            }

            let mut rx = client.subscribe();
            client
                .send_message("/node", vec![OscArg::String(line.to_string())])
                .await?;

            if let Ok(Ok(response)) =
                tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
            {
                let mut output = response.path.clone();
                if let Some(OscArg::String(s)) = response.args.first() {
                    output.push(' ');
                    output.push_str(s);
                }
                println!("{}", output);
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

        // Custom port retained
        assert_eq!(
            resolve_target_address("10.0.0.1:10000", MixerModel::X32),
            "10.0.0.1:10000"
        );
    }

    #[test]
    fn test_format_scene_header() {
        assert!(format_scene_header(MixerModel::X32, "Sunday", "Service").contains("X32GetScene"));
        assert!(
            format_scene_header(MixerModel::Wing, "Sunday", "Service").contains("WingGetScene")
        );
        assert!(
            format_scene_header(MixerModel::XR18, "Sunday", "Service").contains("XAirGetScene")
        );
        assert!(
            format_scene_header(MixerModel::XR16, "Sunday", "Service").contains("XAirGetScene")
        );
        assert!(
            format_scene_header(MixerModel::XR12, "Sunday", "Service").contains("XAirGetScene")
        );
    }

    #[test]
    fn test_is_path_valid_for_model_channels() {
        // X32: max 32
        assert!(is_path_valid_for_model(MixerModel::X32, "/ch/01/mix/fader"));
        assert!(is_path_valid_for_model(MixerModel::X32, "/ch/32/mix/fader"));
        assert!(!is_path_valid_for_model(
            MixerModel::X32,
            "/ch/33/mix/fader"
        ));

        // Wing: max 40
        assert!(is_path_valid_for_model(
            MixerModel::Wing,
            "/ch/40/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::Wing,
            "/ch/41/mix/fader"
        ));

        // XR18: max 16
        assert!(is_path_valid_for_model(
            MixerModel::XR18,
            "/ch/16/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::XR18,
            "/ch/17/mix/fader"
        ));

        // XR12: max 12
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
        // X32 buses 16, matrices 6
        assert!(is_path_valid_for_model(
            MixerModel::X32,
            "/bus/16/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::X32,
            "/bus/17/mix/fader"
        ));
        assert!(is_path_valid_for_model(
            MixerModel::X32,
            "/mtx/06/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::X32,
            "/mtx/07/mix/fader"
        ));

        // XR series: matrices unsupported
        assert!(!is_path_valid_for_model(
            MixerModel::XR18,
            "/mtx/01/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::XR12,
            "/mtx/01/mix/fader"
        ));

        // XR12: max 2 buses
        assert!(is_path_valid_for_model(
            MixerModel::XR12,
            "/bus/02/mix/fader"
        ));
        assert!(!is_path_valid_for_model(
            MixerModel::XR12,
            "/bus/03/mix/fader"
        ));
    }

    #[test]
    fn test_cli_args_parsing() {
        let args = Args::parse_from([
            "x32_get_scene",
            "-M",
            "Wing",
            "-i",
            "10.0.0.5",
            "-s",
            "Main",
            "-n",
            "Test",
        ]);
        assert_eq!(args.model, MixerModel::Wing);
        assert_eq!(args.ip, "10.0.0.5");
        assert_eq!(args.scene_name.as_deref(), Some("Main"));
        assert_eq!(args.note.as_deref(), Some("Test"));
    }
}
