//! `x32_desk_save` is a command-line utility for saving preferences, scenes, and routing data
//! from a Behringer X32 digital mixer to a file. It is a Rust implementation of the original
//! `X32DeskSave.c` tool by Patrick-Gilles Maillot.

use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;
use tokio::time::{timeout, Duration};
use x32_lib::{error::X32Error, MixerClient, MixerModel};

/// Predefined OSC node lists for desk, scene, and routing saves.
pub mod nodes;

/// Command-line arguments for `x32_desk_save`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Target mixer IP address.
    #[arg(short, long, default_value = "192.168.1.64")]
    pub ip: String,

    /// Target mixer model.
    #[arg(
        short = 'M',
        long,
        default_value = "X32",
        help = "Mixer model: X32, Wing, XR18, XR16, XR12"
    )]
    pub model: MixerModel,

    /// Transport type (`auto`, `osc`, `usb`, `aes50`).
    #[arg(long, default_value = "auto")]
    pub transport: String,

    /// USB serial port name.
    #[arg(long, default_value = "")]
    pub usb_port: String,

    /// AES50 bridge IP address.
    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    /// Save desk preferences and status nodes.
    #[arg(short = 'd', long)]
    pub desk_save: bool,

    /// Save scene nodes.
    #[arg(short, long)]
    pub scene: bool,

    /// Save routing nodes.
    #[arg(short, long)]
    pub routing: bool,

    /// Optional pattern file listing OSC nodes to save.
    #[arg(short, long)]
    pub pattern_file: Option<PathBuf>,

    /// Output file path.
    #[arg(index = 1)]
    pub destination_file: PathBuf,
}

async fn get_desk_data(client: &MixerClient, commands: &[String]) -> Result<Vec<String>, X32Error> {
    let mut results = Vec::new();
    let mut rx = client.subscribe();

    for cmd in commands {
        let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(cmd.to_string())]);
        client.send_message(&msg.path, msg.args).await?;

        if let Ok(Ok(response)) = timeout(Duration::from_millis(500), rx.recv()).await {
            results.push(response.to_string());
        }
    }

    Ok(results)
}

/// Runs `x32_desk_save` with the given arguments.
pub async fn run(args: Args) -> Result<(), X32Error> {
    let commands: Vec<String> = if args.desk_save {
        nodes::DS_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.scene {
        nodes::SC_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.routing {
        nodes::RO_NODE.iter().map(|s| s.to_string()).collect()
    } else if let Some(pattern_file) = &args.pattern_file {
        let file = File::open(pattern_file)?;
        let mut file_take = std::io::Read::take(file, 1024 * 1024 + 1);
        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut file_take, &mut buffer)?;
        if buffer.len() > 1024 * 1024 {
            return Err(X32Error::Custom("File too large".to_string()));
        }
        let reader = std::io::BufReader::new(std::io::Cursor::new(buffer.into_bytes()));
        reader
            .lines()
            .map_while(Result::ok)
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect()
    } else {
        return Err(X32Error::Custom("No mode selected".to_string()));
    };

    let target_ip = if args.ip.contains(':') {
        args.ip.clone()
    } else {
        format!(
            "{}:{}",
            args.ip,
            x32_lib::model::default_port_for_model(args.model)
        )
    };

    let (client, _) = MixerClient::connect_with_transport(
        &target_ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    )
    .await?;
    let client = std::sync::Arc::new(client);
    println!("Successfully connected to {} at {}", args.model, target_ip);

    let data = get_desk_data(&client, &commands).await?;

    let file = File::create(&args.destination_file)?;
    let mut file = BufWriter::new(file);

    for line in data {
        writeln!(file, "{}", line)?;
    }
    file.flush()?;

    println!(
        "Successfully saved data to {}",
        args.destination_file.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_model_parsing() {
        let args_default = Args::parse_from(["x32_desk_save", "-d", "out.node"]);
        assert_eq!(args_default.model, MixerModel::X32);

        let args_wing = Args::parse_from(["x32_desk_save", "-M", "Wing", "-d", "out.node"]);
        assert_eq!(args_wing.model, MixerModel::Wing);

        let args_xr12 = Args::parse_from(["x32_desk_save", "--model", "xr12", "-s", "out.node"]);
        assert_eq!(args_xr12.model, MixerModel::XR12);
    }
}
