//! `x32_get_scene` is a command-line tool for retrieving scene data from a Behringer X32 mixer.
//!
//! It reads a list of OSC addresses from standard input (one per line), queries the mixer
//! for their current values using the `/node` command, and prints the results to standard
//! output in a format suitable for saving as a scene file.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]

use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, BufRead};
use x32_lib::{create_socket, error::Result};

/// Command-line arguments for `x32_get_scene`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
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
    let socket = create_socket(&args.ip, 100)?;

    let scene_name = match args.scene_name {
        Some(name) => name,
        None => {
            println!("Please enter scene name: ");
            let mut name = String::new();
            io::stdin().read_line(&mut name)?;
            name.trim().to_string()
        }
    };

    let note = match args.note {
        Some(note) => note,
        None => {
            println!("Please enter note data: ");
            let mut note = String::new();
            io::stdin().read_line(&mut note)?;
            note.trim().to_string()
        }
    };

    println!(
        "#2.7# \"{}\" \"{}\" %000000000 1 X32GetScene V1.5 (c)2014 Patrick-Gilles Maillot\n",
        scene_name, note
    );

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.starts_with('/') {
            let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(line)]);
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
