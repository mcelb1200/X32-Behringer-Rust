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
use osc_lib::{OscArg, OscMessage};
use std::io::{self, BufRead, Read};
use tokio::time::Duration;
use x32_lib::{MixerClient, error::Result};

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
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = MixerClient::connect(&args.ip, false).await?;

    let scene_name = match args.scene_name {
        Some(name) => name,
        None => {
            println!("Please enter scene name: ");
            let mut name = String::new();
            io::stdin().lock().take(4096).read_line(&mut name)?;
            name.trim().to_string()
        }
    };

    let note = match args.note {
        Some(note) => note,
        None => {
            println!("Please enter note data: ");
            let mut note = String::new();
            io::stdin().lock().take(4096).read_line(&mut note)?;
            note.trim().to_string()
        }
    };

    println!(
        "#2.7# \"{}\" \"{}\" %000000000 1 X32GetScene V1.5 (c)2014 Patrick-Gilles Maillot\n",
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
            // Using query_value for /node might be tricky because we want the whole message or specific handling.
            // But let's see what the original did: it sent /node and expected a response.

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
