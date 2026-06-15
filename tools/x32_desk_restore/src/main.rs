//! `x32_desk_restore` is a command-line utility for restoring preferences, scenes, and routing data
//! to a Behringer X32 digital mixer from a file. It is a Rust implementation of the original
//! `X32DeskRestore.c` tool by Patrick-Gilles Maillot.

use clap::Parser;
use osc_lib::OscMessage;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;
use x32_lib::{
    MixerClient,
    error::{Result, X32Error},
};

mod parse;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.1.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    #[arg(index = 1)]
    file: PathBuf,
}

async fn send_commands(client: &MixerClient, commands: &[String]) -> Result<()> {
    let mut rx = client.subscribe();

    for cmd_str in commands {
        let msg = if cmd_str.starts_with("/-") {
            if let Some(parsed_msg) = parse::parse_node_line(cmd_str) {
                parsed_msg
            } else {
                continue;
            }
        } else {
            let command = if cmd_str.starts_with('/') {
                cmd_str.to_string()
            } else {
                format!("/{}", cmd_str)
            };

            match OscMessage::from_str(&command) {
                Ok(msg) => msg,
                Err(_) => OscMessage::new(command, vec![]),
            }
        };

        client.send_message(&msg.path, msg.args).await?;

        // Wait a small amount for any response (equivalent to `socket.recv`)
        let _ = timeout(Duration::from_millis(50), rx.recv()).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let file = File::open(&args.file)?;

    let metadata = file.metadata()?;
    if metadata.len() > 1024 * 1024 {
        return Err(X32Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too large",
        )));
    }

    let mut content = String::new();
    file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
    if content.len() > 1024 * 1024 {
        return Err(X32Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too large",
        )));
    }

    let commands: Vec<String> = content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect();

    if commands.is_empty() {
        eprintln!("No commands to send from file: {}", args.file.display());
        return Ok(());
    }

    let (client, _transport) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    )
    .await?;

    println!("Successfully connected to X32 at {}", args.ip);

    send_commands(&client, &commands).await?;

    println!("Successfully restored data from {}", args.file.display());

    Ok(())
}
