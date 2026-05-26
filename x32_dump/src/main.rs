//! `x32_dump` is a command-line utility for dumping binary OSC payloads to a human-readable format.
//!
//! It reads raw OSC data from a file or standard input, parses it, and prints the formatted messages.
//! This tool essentially serves as a standalone equivalent to the `Xfdump` and `Xdump` functions
//! found in the original C library.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Rust implementation by:** mcelb1200

use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscMessage;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

/// Command-line arguments for the `x32_dump` tool.
#[derive(Parser, Debug)]
#[command(author, version, about = "A utility to dump binary OSC messages to text", long_about = None)]
struct Args {
    /// The file containing raw OSC data to read. If not provided, reads from standard input.
    #[arg(name = "FILE")]
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut buffer = Vec::new();

    if let Some(file_path) = args.file {
        let file = File::open(&file_path)
            .with_context(|| format!("Failed to open file: {:?}", file_path))?;
        // Read file contents, capped to prevent memory exhaustion (max 10MB)
        file.take(10 * 1024 * 1024).read_to_end(&mut buffer)?;
    } else {
        let stdin = io::stdin();
        let handle = stdin.lock();
        // Read from stdin, capped to prevent memory exhaustion
        handle.take(10 * 1024 * 1024).read_to_end(&mut buffer)?;
    }

    if buffer.is_empty() {
        return Ok(());
    }

    // Try to parse the buffer as an OSC message.
    // In the future, this could be expanded to support bundles.
    match OscMessage::from_bytes(&buffer) {
        Ok(msg) => {
            println!("{}", msg);
        }
        Err(e) => {
            eprintln!("Failed to parse OSC message: {}", e);
            // Optionally, print a raw hex dump if parsing fails
            let mut hex_str = String::new();
            static HEX: &[u8; 16] = b"0123456789abcdef";
            for &byte in &buffer {
                hex_str.push(HEX[(byte >> 4) as usize] as char);
                hex_str.push(HEX[(byte & 0x0f) as usize] as char);
                hex_str.push(' ');
            }
            eprintln!("Raw hex: {}", hex_str);
        }
    }

    Ok(())
}
