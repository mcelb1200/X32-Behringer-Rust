//! `x32_set_lib` is a command-line tool for uploading library presets to a Behringer X32/M32 mixer.
//!
//! It reads preset files (Channel, Effects, or Routing) and sends them to the specified library slots on the mixer.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use x32_lib::MixerClient;
use tokio::time::{timeout, Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.1.62")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    #[arg(short, long, default_value_t = 1)]
    start_index: usize,

    #[arg(required = true)]
    files: Vec<PathBuf>,

    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);

    println!("Connected to X32 at {}", args.ip);

    let mut current_index = args.start_index;

    for file_path in &args.files {
        if !file_path.exists() {
            eprintln!("Warning: File not found: {:?}", file_path);
            continue;
        }

        if current_index > 100 {
            eprintln!("Warning: Reached maximum library slot index (100). Stopping.");
            break;
        }

        if args.verbose {
            println!("Processing {:?} -> Slot {}", file_path, current_index);
        }

        process_file(&client, file_path, current_index, args.verbose).await?;
        current_index += 1;
    }

    Ok(())
}

async fn process_file(
    client: &MixerClient,
    path: &Path,
    index: usize,
    verbose: bool,
) -> Result<()> {
    let file = File::open(path)?;

    // Sentinel: Prevent OOM from maliciously large files
    if file.metadata()?.len() > 1024 * 1024 {
        return Err(anyhow::anyhow!("File too large to load (max 1MB)"));
    }

    let mut content = String::new();
    file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
    if content.len() > 1024 * 1024 {
        return Err(anyhow::anyhow!("File too large to load (max 1MB)"));
    }

    let mut lines = content.lines();

    let header = lines.next().ok_or_else(|| anyhow::anyhow!("Empty file"))?;
    if !header.starts_with("#2.1#") {
        return Err(anyhow::anyhow!("Invalid file format: missing #2.1# header"));
    }

    let name = header
        .split('"')
        .nth(1)
        .unwrap_or(&format!("Preset_{:03}", index))
        .to_string();

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let _lib_type = match ext.as_str() {
        "chn" => "libchan",
        "efx" => "libfx",
        "rou" => "librout",
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown preset file extension: {}. Expected .chn, .efx, or .rou",
                ext
            ))
        }
    };

    let mut param_index = 0;

    let mut rx = client.subscribe();

    for line in lines {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let address = match ext.as_str() {
            "chn" => {
                if param_index == 25 {
                    "headamp/000".to_string()
                } else if param_index == 0 {
                    "ch/01/config".to_string()
                } else if param_index == 1 {
                    "ch/01/delay".to_string()
                } else if param_index == 2 {
                    "ch/01/preamp".to_string()
                } else if param_index == 3 {
                    "ch/01/gate".to_string()
                } else if param_index == 4 {
                    "ch/01/gate/filter".to_string()
                } else if param_index == 5 {
                    "ch/01/dyn".to_string()
                } else if param_index == 6 {
                    "ch/01/dyn/filter".to_string()
                } else if (7..=11).contains(&param_index) {
                    let eq_idx = param_index - 7;
                    if eq_idx == 0 {
                        "ch/01/eq".to_string()
                    } else {
                        format!("ch/01/eq/{}", eq_idx)
                    }
                } else if param_index == 12 {
                    "ch/01/mix".to_string()
                } else if (13..=28).contains(&param_index) {
                    format!("ch/01/mix/{:02}", param_index - 12)
                } else {
                    param_index += 1;
                    continue;
                }
            }
            "efx" => {
                if param_index == 0 {
                    "fx/1/type".to_string()
                } else if param_index == 1 {
                    "fx/1/source".to_string()
                } else if param_index == 2 {
                    "fx/1/par".to_string()
                } else {
                    param_index += 1;
                    continue;
                }
            }
            "rou" => {
                if param_index == 0 {
                    "config/routing/IN".to_string()
                } else if param_index == 1 {
                    "config/routing/AES50A".to_string()
                } else if param_index == 2 {
                    "config/routing/AES50B".to_string()
                } else if param_index == 3 {
                    "config/routing/CARD".to_string()
                } else if param_index == 4 {
                    "config/routing/OUT".to_string()
                } else if param_index == 5 {
                    "config/routing/PLAY".to_string()
                } else if (6..=37).contains(&param_index) {
                    let idx = param_index - 6;
                    let out_idx = (idx / 2) + 1;
                    if idx % 2 == 0 {
                        format!("outputs/main/{:02}", out_idx)
                    } else {
                        format!("outputs/main/{:02}/delay", out_idx)
                    }
                } else if (38..=43).contains(&param_index) {
                    format!("outputs/aux/{:02}", param_index - 37)
                } else if (44..=75).contains(&param_index) {
                    let idx = param_index - 44;
                    let p16_idx = (idx / 2) + 1;
                    if idx % 2 == 0 {
                        format!("outputs/p16/{:02}", p16_idx)
                    } else {
                        format!("outputs/p16/{:02}/iQ", p16_idx)
                    }
                } else if (76..=77).contains(&param_index) {
                    format!("outputs/aes/{:02}", param_index - 75)
                } else {
                    param_index += 1;
                    continue;
                }
            }
            _ => {
                param_index += 1;
                continue;
            }
        };

        let node_arg = format!("{} {}", address, line.trim_end());
        let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(node_arg)]);
        client.send_message(&msg.path, msg.args).await?;
        let _ = timeout(Duration::from_millis(10), rx.recv()).await;

        param_index += 1;
    }

    let save_args = match ext.as_str() {
        "chn" => vec![
            OscArg::String("libchan".to_string()),
            OscArg::Int(index as i32 - 1),
            OscArg::String(name),
            OscArg::Int(0),
        ],
        "efx" => vec![
            OscArg::String("libfx".to_string()),
            OscArg::Int(index as i32 - 1),
            OscArg::String(name),
        ],
        "rou" => vec![
            OscArg::String("librout".to_string()),
            OscArg::Int(index as i32 - 1),
            OscArg::String(name),
        ],
        _ => unreachable!(),
    };

    let msg = OscMessage::new("/save".to_string(), save_args);
    client.send_message(&msg.path, msg.args).await?;

    if verbose {
        println!("  Saved to slot {}", index);
    }

    Ok(())
}
