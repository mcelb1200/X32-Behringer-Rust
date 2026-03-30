//! `xair_get_scene` is a command-line tool for retrieving scene data from a Behringer XAir mixer.
//!
//! It reads a list of OSC addresses from standard input (one per line), queries the mixer
//! for their current values using the `/node` command, and prints the results to standard
//! output in a format suitable for saving as a scene file.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Original C code for XAir version:** Ken Mitchell
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]

use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, BufRead, Read};
use x32_lib::{create_socket, error::Result};

/// Command-line arguments for `xair_get_scene`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the XAir console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Scene name.
    #[arg(short, long)]
    scene_name: Option<String>,

    /// Note data.
    #[arg(short, long)]
    note: Option<String>,
}

/// The main entry point for the application.
fn main() -> Result<()> {
    let args = Args::parse();

    // The XAir port is 10024. `x32_lib::create_socket` adds `:10023` if no port is specified.
    // For XAir tools, check if the IP contains a port (`ip.contains(':')`); if not, append `:10024`.
    let ip = if args.ip.contains(':') {
        args.ip.clone()
    } else {
        format!("{}:10024", args.ip)
    };

    let socket = create_socket(&ip, 100)?;

    let scene_name = match args.scene_name {
        Some(name) => name,
        None => "Unsaved".to_string()
    };

    let note = match args.note {
        Some(note) => note,
        None => "".to_string()
    };

    println!(
        "#2.1# \"{}\" \"{}\" %000000000 1 XAirGetScene V1.4 (c)2014 Patrick-Gilles Maillot\n",
        scene_name, note
    );

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
            let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(line.to_string())]);
            socket.send(&msg.to_bytes()?)?;

            let mut buf = [0; 512];
            let len = socket.recv(&mut buf)?;
            let response = OscMessage::from_bytes(&buf[..len])?;

            let mut output = response.path.clone();
            if let Some(OscArg::String(s)) = response.args.first() {
                output.push(' ');
                output.push_str(s);
            }
            println!("{}", output);
        }
    }

    Ok(())
}
