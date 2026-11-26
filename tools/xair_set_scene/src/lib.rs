//! `xair_set_scene` reads scene lines from stdin and sends them as OSC commands
//! to a Behringer X-Air mixer.

use anyhow::Result;
use clap::Parser;
use osc_lib::OscMessage;
use std::io::{self, BufRead, Read};
use x32_lib::MixerClient;

/// Command-line arguments for `xair_set_scene`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// IP address of the X-Air mixer.
    #[arg(short, long)]
    pub ip: String,

    /// Delay between commands in milliseconds.
    #[arg(short, long, default_value_t = 1)]
    pub delay: u64,
}

/// Connects to the X-Air mixer and sends scene commands read from stdin.
pub async fn run(args: Args) -> Result<()> {
    let ip = x32_lib::model::resolve_target_address(&args.ip, x32_lib::MixerModel::XR18);

    let client = MixerClient::connect(&ip, false).await?;

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut parser = x32_lib::scene_parse::SceneParser::with_model(x32_lib::MixerModel::XR18);

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
            // First try to parse it as a scene line
            let mut messages = parser.parse_scene_line(line);

            // If it returns empty, it might be a fully formed raw OSC line, fall back to from_str
            if messages.is_empty() {
                use std::str::FromStr;
                match OscMessage::from_str(line) {
                    Ok(msg) => messages.push(msg),
                    Err(e) => eprintln!("Error parsing line: {} - {}", line, e),
                }
            }

            for msg in messages {
                if args.delay > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(args.delay)).await;
                }
                client.send_message(&msg.path, msg.args).await?;
            }
        }
    }

    Ok(())
}
