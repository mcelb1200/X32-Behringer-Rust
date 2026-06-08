//! `x32_desk_save` is a command-line utility for saving preferences, scenes, and routing data
//! from a Behringer X32 digital mixer to a file. It is a Rust implementation of the original
//! `X32DeskSave.c` tool by Patrick-Gilles Maillot.

use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;
use tokio::time::{timeout, Duration};
use x32_lib::{MixerClient, error::X32Error};

mod nodes;

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

    #[arg(short = 'd', long)]
    desk_save: bool,

    #[arg(short, long)]
    scene: bool,

    #[arg(short, long)]
    routing: bool,

    #[arg(short, long)]
    pattern_file: Option<PathBuf>,

    #[arg(index = 1)]
    destination_file: PathBuf,
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

#[tokio::main]
async fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    let commands: Vec<String> = if args.desk_save {
        nodes::DS_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.scene {
        nodes::SC_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.routing {
        nodes::RO_NODE.iter().map(|s| s.to_string()).collect()
    } else if let Some(pattern_file) = &args.pattern_file {
        let file = File::open(pattern_file)?;
        let reader = std::io::BufReader::new(file);
        #[allow(clippy::lines_filter_map_ok)]
        reader.lines()
            .filter_map(|l| l.ok())
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect()
    } else {
        return Err(X32Error::Custom("No mode selected".to_string()));
    };

    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);
    println!("Successfully connected to X32 at {}", args.ip);

    let data = get_desk_data(&client, &commands).await?;

    let file = File::create(&args.destination_file)?;
    let mut file = BufWriter::new(file);

    for line in data {
        writeln!(file, "{}", line)?;
    }
    file.flush()?;

    println!("Successfully saved data to {}", args.destination_file.display());

    Ok(())
}
