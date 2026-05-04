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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = MixerClient::connect(&args.ip, true).await?;

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        let len = stdin_lock.by_ref().take(4096).read_line(&mut line)?;
        if len == 0 {
            break;
        }
        let line = line.trim();
        if line.starts_with('/') {
            let mut parts = line.split_whitespace();
            let path = parts.next().unwrap();
            let mut args = Vec::new();

            for part in parts {
                if let Ok(i) = part.parse::<i32>() {
                    args.push(OscArg::Int(i));
                } else if let Ok(f) = part.parse::<f32>() {
                    args.push(OscArg::Float(f));
                } else {
                    args.push(OscArg::String(part.to_string()));
                }
            }

            let msg = OscMessage::new(path.to_string(), args);
            client.send_message(&msg.path, msg.args).await?;
        }
    }

    Ok(())
}
