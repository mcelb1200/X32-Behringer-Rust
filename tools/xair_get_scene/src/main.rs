use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, BufRead, Read};
use x32_lib::MixerClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: String,

    #[arg(short, long)]
    scene_name: Option<String>,

    #[arg(short, long)]
    note: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let ip = if args.ip.contains(':') {
        args.ip.clone()
    } else {
        format!("{}:10024", args.ip)
    };

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
                    // Line too long, discard remainder
                    let mut discard = Vec::with_capacity(1024);
                    loop {
                        discard.clear();
                        let mut chunk_handle = stdin_lock.by_ref().take(1024);
                        match chunk_handle.read_until(b'\n', &mut discard) {
                            Ok(0) => break,
                            Err(e) => return Err(e.into()),
                            Ok(_) => {
                                if discard.ends_with(b"\n") {
                                    break;
                                }
                            }
                        }
                    }
                    eprintln!("Input line too long, discarded.");
                    continue;
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
