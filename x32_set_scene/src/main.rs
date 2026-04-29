//! `x32_set_scene` is a command-line tool for sending a sequence of OSC commands to a Behringer X32 mixer.
//!
//! It reads OSC commands from standard input (one per line) and sends them to the mixer.
//! This is typically used to restore a scene or apply a batch of settings.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Original C code for XAir version:** Ken Mitchell
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]

use clap::Parser;
use osc_lib::OscMessage;
use std::io::{self, BufRead, Read};
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use x32_lib::{create_socket, error::Result};

/// Command-line arguments for `x32_set_scene`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Delay between commands in milliseconds.
    #[arg(short, long, default_value_t = 1)]
    delay: u64,
}

/// The main entry point for the application.
fn main() -> Result<()> {
    let args = Args::parse();
    let socket = create_socket(&args.ip, 100)?;

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    loop {
        let mut line = String::new();
        let len = stdin_lock.by_ref().take(4096).read_line(&mut line)?;
        if len == 0 {
            break;
        }
        let line = line.trim();
        if line.starts_with('/') {
            // First try to parse it as a scene line
            let mut messages = x32_lib::scene_parse::parse_scene_line(line);

            // If it returns empty, it might be a fully formed raw OSC line, fall back to from_str
            if messages.is_empty() {
                match OscMessage::from_str(line) {
                    Ok(msg) => messages.push(msg),
                    Err(e) => eprintln!("Error parsing line: {} - {}", line, e),
                }
            }

            for msg in messages {
                if let Ok(bytes) = msg.to_bytes() {
                    socket.send(&bytes)?;
                    if args.delay > 0 {
                        thread::sleep(Duration::from_millis(args.delay));
                    }
                }
            }
        }
    }

    Ok(())
}
