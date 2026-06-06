//! `x32_copy_fx` is a command-line utility for managing effects (FX) settings on Behringer X32/M32 mixers.
//!
//! It allows you to:
//! - Reset an FX slot to its default parameters.
//! - Copy settings from one FX slot to another.
//! - Copy settings between the 'A' and 'B' sides of a dual-channel effect within the same slot.
//!
//! It supports reading custom default values from a file, allowing for personalized initial states.

mod fx_defaults;

use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, Read};
use std::path::PathBuf;
use x32_lib::{MixerClient, error::X32Error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    #[arg(long)]
    defaults_file: Option<PathBuf>,

    #[arg(long)]
    from: u8,

    #[arg(long, default_value_t = 0)]
    to: u8,

    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Reset,
    Copy {
        #[arg(long)]
        master: bool,
    },
    AtoB {
        #[arg(long)]
        master: bool,
    },
    BtoA {
        #[arg(long)]
        master: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    if args.from < 1 || args.from > 8 {
        return Err(X32Error::from("Source FX slot must be between 1 and 8.".to_string()));
    }
    if args.to > 8 {
        return Err(X32Error::from("Destination FX slot must be between 1 and 8.".to_string()));
    }

    let (client, _transport) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;

    match args.action {
        Action::Reset => reset_fx(&client, args.from, args.defaults_file).await,
        Action::Copy { master } => copy_fx(&client, args.from, args.to, master).await,
        Action::AtoB { master } => copy_a_to_b(&client, args.from, master).await,
        Action::BtoA { master } => copy_b_to_a(&client, args.from, master).await,
    }
}

fn load_user_defaults(path: PathBuf) -> Result<HashMap<String, String>, X32Error> {
    let file = File::open(path)?;

    if file.metadata()?.len() > 1024 * 1024 {
        return Err(X32Error::from("Defaults file too large to load (max 1MB)".to_string()));
    }

    let mut content = String::new();
    file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
    if content.len() > 1024 * 1024 {
        return Err(X32Error::from("Defaults file too large to load (max 1MB)".to_string()));
    }

    let mut user_defaults = HashMap::new();
    let cursor = std::io::Cursor::new(content);
    let mut lines = cursor.lines();

    while let Some(Ok(name_line)) = lines.next() {
        if let Some(Ok(params_line)) = lines.next() {
            user_defaults.insert(name_line.trim().to_string(), params_line.trim().to_string());
        }
    }

    Ok(user_defaults)
}

async fn reset_fx(client: &MixerClient, from: u8, defaults_file: Option<PathBuf>) -> Result<(), X32Error> {
    println!("Resetting FX slot {}.", from);

    let user_defaults: Option<HashMap<String, String>> = if let Some(path) = defaults_file {
        Some(load_user_defaults(path)?)
    } else {
        None
    };

    let fx_type_addr = format!("/fx/{}/type", from);
    let arg = client.query_value(&fx_type_addr).await?;

    if let OscArg::Int(fx_type_id) = arg {
        let fx_name = get_fx_name_from_id(fx_type_id)?;

        let defaults_str = if let Some(ref defaults) = user_defaults {
            defaults.get(fx_name).map(|s| s.as_str())
        } else {
            None
        }
        .or_else(|| fx_defaults::FX_DEFAULTS.get(fx_name).copied());

        if let Some(defaults) = defaults_str {
            let mut args = Vec::new();
            for val in defaults.split_whitespace() {
                if let Ok(i) = val.parse::<i32>() {
                    args.push(OscArg::Int(i));
                } else if let Ok(f) = val.parse::<f32>() {
                    args.push(OscArg::Float(f));
                } else {
                    args.push(OscArg::String(val.to_string()));
                }
            }
            let par_addr = format!("/fx/{}/par", from);
            client.send_message(&par_addr, args).await?;
            Ok(())
        } else {
            Err(X32Error::from(format!("Defaults not found for FX type: {}", fx_name)))
        }
    } else {
        Err(X32Error::from("Could not determine FX type.".to_string()))
    }
}

fn get_fx_name_from_id(id: i32) -> Result<&'static str, X32Error> {
    let fx_names = [
        "HALL", "AMBI", "RPLT", "ROOM", "CHAM", "PLAT", "VREV", "VRM", "GATE", "RVRS", "DLY",
        "3TAP", "4TAP", "CRS", "FLNG", "PHAS", "DIMC", "FILT", "ROTA", "PAN", "SUB", "D/RV",
        "CR/R", "FL/R", "D/CR", "D/FL", "MODD", "GEQ2", "GEQ", "TEQ2", "TEQ", "DES2", "DES", "P1A",
        "P1A2", "PQ5", "PQ5S", "WAVD", "LIM", "CMB", "CMB2", "FAC", "FAC1M", "FAC2", "LEC", "LEC2",
        "ULC", "ULC2", "ENH2", "ENH", "EXC2", "EXC", "IMG", "EDI", "SON", "AMP2", "AMP", "DRV2",
        "DRV", "PIT2", "PIT",
    ];
    if id >= 0 && id < fx_names.len() as i32 {
        Ok(fx_names[id as usize])
    } else {
        Err(X32Error::from(format!("Invalid FX type ID: {}", id)))
    }
}

async fn copy_param(
    client: &MixerClient,
    from_fx: u8,
    from_param: u8,
    to_fx: u8,
    to_param: u8,
) -> Result<(), X32Error> {
    let source_addr = format!("/fx/{}/par/{:02}", from_fx, from_param);
    let arg = client.query_value(&source_addr).await?;

    let dest_addr = format!("/fx/{}/par/{:02}", to_fx, to_param);
    client.send_message(&dest_addr, vec![arg]).await?;
    Ok(())
}

async fn copy_fx(client: &MixerClient, from: u8, to: u8, master: bool) -> Result<(), X32Error> {
    if to == 0 {
        return Err(X32Error::from("Destination FX slot must be provided for copy action.".to_string()));
    }
    println!("Copying FX from slot {} to {}.", from, to);
    for i in 1..32 {
        copy_param(client, from, i, to, i).await?;
        copy_param(client, from, i + 32, to, i + 32).await?;
    }
    if master {
        copy_param(client, from, 32, to, 32).await?;
        copy_param(client, from, 64, to, 64).await?;
    }
    Ok(())
}

async fn copy_a_to_b(client: &MixerClient, from: u8, master: bool) -> Result<(), X32Error> {
    println!("Copying from side A to side B of FX slot {}.", from);
    for i in 1..32 {
        copy_param(client, from, i, from, i + 32).await?;
    }
    if master {
        copy_param(client, from, 32, from, 64).await?;
    }
    Ok(())
}

async fn copy_b_to_a(client: &MixerClient, from: u8, master: bool) -> Result<(), X32Error> {
    println!("Copying from side B to side A of FX slot {}.", from);
    for i in 33..64 {
        copy_param(client, from, i, from, i - 32).await?;
    }
    if master {
        copy_param(client, from, 64, from, 32).await?;
    }
    Ok(())
}
