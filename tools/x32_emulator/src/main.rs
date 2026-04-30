//! `x32_emulator` is a command-line tool that simulates the behavior of a Behringer X32 digital mixer.
//!
//! It runs a UDP server that listens for OSC commands and responds as a real X32 console would.
//! This is invaluable for offline testing, development, and verifying the behavior of other
//! X32 control tools without needing access to physical hardware.
//!
//! The emulator maintains an internal state of the mixer's parameters, allowing for
//! stateful interactions (e.g., setting a fader level and then retrieving it).

use anyhow::Result;
use clap::Parser;
use x32_emulator::server;

/// Command-line arguments for the `x32_emulator`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// IP address to bind to.
    #[arg(short, long, default_value_t = String::from("0.0.0.0"))]
    ip: String,

    /// Port number to bind to.
    #[arg(short, long, default_value_t = 10023)]
    port: u16,
}

/// The main entry point for the emulator.
///
/// This function parses command-line arguments and starts the emulator server.
fn main() -> Result<()> {
    let cli = Cli::parse();
    let bind_addr = format!("{}:{}", cli.ip, cli.port);
    server::run(&bind_addr, None, None)
}
