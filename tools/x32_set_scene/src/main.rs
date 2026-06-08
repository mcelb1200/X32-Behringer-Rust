//! `x32_set_scene` is a command-line tool for sending a sequence of OSC commands to a Behringer X32 mixer.
//!
//! It reads OSC commands from standard input (one per line) and sends them to the mixer.
//! This is typically used to restore a scene or apply a batch of settings.

use clap::Parser;
use osc_lib::OscMessage;
use std::io::{self, BufRead, Read};
use std::str::FromStr;
use std::time::Duration;
use x32_lib::{MixerClient, error::Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    #[arg(short, long, default_value_t = 1)]
    delay: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let (client, _transport) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut parser = x32_lib::scene_parse::SceneParser::new();
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
            let mut messages = parser.parse_scene_line(line);

            if messages.is_empty() {
                match OscMessage::from_str(line) {
                    Ok(msg) => messages.push(msg),
                    Err(e) => eprintln!("Error parsing line: {} - {}", line, e),
                }
            }

            for msg in messages {
                client.send_message(&msg.path, msg.args).await?;
                if args.delay > 0 {
                    tokio::time::sleep(Duration::from_millis(args.delay)).await;
                }
            }
        }
    }

    Ok(())
}
