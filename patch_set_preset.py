import re

with open("tools/x32_set_preset/src/main.rs", "r") as f:
    text = f.read()

out = """//! `x32_set_preset` is a command-line utility for uploading scene and snippet presets
//! to a Behringer X32/M32 digital mixer.
//!
//! It reads `.scn` (scene) or `.snp` (snippet) files and sends their contents to the
//! specified preset slots on the mixer.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use anyhow::{anyhow, Result};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::time::{timeout, Duration};
use x32_lib::MixerClient;

/// Command-line arguments for the `x32_set_preset` tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    /// Starting slot index for uploading.
    #[arg(short, long, default_value_t = 1)]
    start_index: usize,

    /// The `.scn` or `.snp` files to upload.
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Print verbose output during processing.
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

    if args.verbose {
        println!("Connected to X32 at {}", args.ip);
    }

    let mut current_index = args.start_index;

    for file_path in &args.files {
        if !file_path.exists() {
            eprintln!("Warning: File not found: {:?}", file_path);
            continue;
        }

        if current_index > 100 {
            eprintln!("Warning: Reached maximum preset slot index (100). Stopping.");
            break;
        }

        if args.verbose {
            println!("Processing {:?} -> Slot {}", file_path, current_index);
        }

        if let Err(e) = process_file(&client, file_path, current_index, args.verbose).await {
            eprintln!("Error processing {:?}: {}", file_path, e);
        } else {
            current_index += 1;
        }
    }

    Ok(())
}

/// Reads a preset file, sends its commands to the mixer, and saves the preset to a slot.
async fn process_file(
    client: &MixerClient,
    path: &Path,
    index: usize,
    verbose: bool,
) -> Result<()> {
    let mut file = File::open(path)?;

    // Sentinel: Prevent OOM from maliciously large files
    if file.metadata()?.len() > 1024 * 1024 {
        return Err(anyhow!("File too large to load (max 1MB)"));
    }

    let mut content = String::new();
    file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
    if content.len() > 1024 * 1024 {
        return Err(anyhow!("File too large to load (max 1MB)"));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext != "scn" && ext != "snp" {
        return Err(anyhow!(
            "Unsupported file extension: {}. Expected .scn or .snp",
            ext
        ));
    }

    let mut lines = content.lines();

    // Parse header to get name and note
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("Empty file"))?
        .trim();

    // Headers look like: #4.0# "Scene Name" "Scene Note" %...
    let parts: Vec<&str> = header.split('"').collect();
    let name = if parts.len() >= 3 {
        parts[1].to_string()
    } else {
        format!("Preset_{:03}", index)
    };

    let note = if parts.len() >= 5 {
        parts[3].to_string()
    } else {
        "".to_string()
    };

    let mut rx = client.subscribe();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let msg = if trimmed.starts_with("/-") {
            // It's a structured node line (like in snippet files), parse it as a command directly
            let cmd_str = if trimmed.starts_with('/') { trimmed.to_string() } else { format!("/{}", trimmed) };
            if let Ok(m) = OscMessage::from_str(&cmd_str) {
                m
            } else {
                OscMessage::new(cmd_str, vec![])
            }
        } else {
            // It's a standard parameter line, parse it
            match OscMessage::from_str(trimmed) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("  Warning: Failed to parse line '{}': {}", trimmed, e);
                    continue;
                }
            }
        };

        client.send_message(&msg.path, msg.args).await?;
        let _ = timeout(Duration::from_millis(5), rx.recv()).await;
    }

    // Save the preset to the specified slot
    let save_target = if ext == "scn" { "scene" } else { "snippet" };

    let save_args = vec![
        OscArg::String(save_target.to_string()),
        OscArg::Int(index as i32), // Note: scenes/snippets are 1-100, not 0-99 in /save arguments according to C code. Actually wait, C code does `index` directly!
        OscArg::String(name.clone()),
        OscArg::String(note),
    ];

    let save_msg = OscMessage::new("/save".to_string(), save_args);
    client.send_message(&save_msg.path, save_msg.args).await?;

    if verbose {
        println!("  Saved {} to slot {} ({})", ext, index, name);
    }

    Ok(())
}
"""

with open("tools/x32_set_preset/src/main.rs", "w") as f:
    f.write(out)
