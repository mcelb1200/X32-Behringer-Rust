//! `xair_get_scene` queries OSC paths from stdin on a Behringer X-Air mixer
//! and prints the scene data to stdout.

use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, BufRead, Read};
use x32_lib::MixerClient;

/// Command-line arguments for `xair_get_scene`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// IP address of the X-Air mixer.
    #[arg(short, long)]
    pub ip: String,

    /// Optional scene name for the header.
    #[arg(short, long)]
    pub scene_name: Option<String>,

    /// Optional scene note for the header.
    #[arg(short, long)]
    pub note: Option<String>,
}

/// Connects to the X-Air mixer and queries OSC paths read from stdin.
pub async fn run(args: Args) -> Result<()> {
    let ip = x32_lib::model::resolve_target_address(&args.ip, x32_lib::MixerModel::XR18);

    let client = MixerClient::connect(&ip, false).await?;

    let scene_name = match args.scene_name {
        Some(name) => name,
        None => "Unsaved".to_string(),
    };

    let note = match args.note {
        Some(note) => note,
        None => "".to_string(),
    };

    println!(
        "#2.1# \"{}\" \"{}\" %000000000 1 XAirGetScene V1.4 (c)2014 Patrick-Gilles Maillot\n",
        scene_name, note
    );

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
                    return Err(anyhow::anyhow!("Input line too long (exceeds max bytes)"));
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
            let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(line.to_string())]);
            client.send_message(&msg.path, msg.args.clone()).await?;

            if let Ok(arg) = client.query_value(line).await {
                let response = OscMessage::new(line.to_string(), vec![arg]);
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
