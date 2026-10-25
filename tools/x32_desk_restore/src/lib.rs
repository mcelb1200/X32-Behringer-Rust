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
    MixerClient, MixerModel,
    error::{Result, X32Error},
};

pub mod parse;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "192.168.1.64")]
    pub ip: String,

    #[arg(
        short = 'M',
        long,
        default_value = "X32",
        help = "Mixer model: X32, Wing, XR18, XR16, XR12"
    )]
    pub model: MixerModel,

    #[arg(long, default_value = "auto")]
    pub transport: String,

    #[arg(long, default_value = "")]
    pub usb_port: String,

    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    #[arg(index = 1)]
    pub file: PathBuf,
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

pub async fn run(args: Args) -> Result<()> {
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

    let target_ip = if args.ip.contains(':') {
        args.ip.clone()
    } else {
        let port = match args.model {
            MixerModel::X32 => 10023,
            MixerModel::Wing => 2222,
            MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 10024,
        };
        format!("{}:{}", args.ip, port)
    };

    let (client, _transport) = MixerClient::connect_with_transport(
        &target_ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    )
    .await?;

    println!("Successfully connected to {} at {}", args.model, target_ip);

    send_commands(&client, &commands).await?;

    println!("Successfully restored data from {}", args.file.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_model_parsing() {
        let args_default = Args::parse_from(["x32_desk_restore", "restore.scn"]);
        assert_eq!(args_default.model, MixerModel::X32);

        let args_wing = Args::parse_from(["x32_desk_restore", "-M", "Wing", "restore.scn"]);
        assert_eq!(args_wing.model, MixerModel::Wing);

        let args_xr18 = Args::parse_from(["x32_desk_restore", "--model", "xr18", "restore.scn"]);
        assert_eq!(args_xr18.model, MixerModel::XR18);
    }
}
