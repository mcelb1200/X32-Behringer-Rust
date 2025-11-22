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
use osc_lib::{OscArg, OscMessage};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::UdpSocket;
use std::path::PathBuf;
use x32_lib::{create_socket, error::X32Error};

/// A utility to copy FX parameters on the Behringer X32/X-Air consoles.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32/X-Air console
    #[arg(short, long)]
    ip: String,

    /// Path to a custom FX defaults file
    #[arg(long)]
    defaults_file: Option<PathBuf>,

    /// Source FX slot (1-8)
    #[arg(long)]
    from: u8,

    /// Destination FX slot (1-8)
    #[arg(long, default_value_t = 0)]
    to: u8,

    /// Copy action to perform
    #[command(subcommand)]
    action: Action,
}

/// Subcommands for the `x32_copy_fx` tool.
#[derive(clap::Subcommand, Debug)]
enum Action {
    /// Reset the source FX slot to its default values
    Reset,
    /// Copy from source FX slot to destination FX slot
    Copy {
        /// Include master parameters in the copy
        #[arg(long)]
        master: bool,
    },
    /// Copy from side A to side B of the source FX slot
    AtoB {
        /// Include master parameters in the copy
        #[arg(long)]
        master: bool,
    },
    /// Copy from side B to side A of the source FX slot
    BtoA {
        /// Include master parameters in the copy
        #[arg(long)]
        master: bool,
    },
}

/// The main entry point.
fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    if args.from < 1 || args.from > 8 {
        return Err(X32Error::from(
            "Source FX slot must be between 1 and 8.".to_string(),
        ));
    }
    if args.to > 8 {
        return Err(X32Error::from(
            "Destination FX slot must be between 1 and 8.".to_string(),
        ));
    }

    let socket = create_socket(&args.ip, 200)?;

    match args.action {
        Action::Reset => reset_fx(&socket, args.from, args.defaults_file),
        Action::Copy { master } => copy_fx(&socket, args.from, args.to, master),
        Action::AtoB { master } => copy_a_to_b(&socket, args.from, master),
        Action::BtoA { master } => copy_b_to_a(&socket, args.from, master),
    }
}

/// Loads user-defined FX defaults from a file.
///
/// The file format is expected to be pairs of lines:
/// 1. FX Name (e.g., "HALL")
/// 2. Space-separated list of parameter values.
///
/// # Arguments
///
/// * `path` - Path to the defaults file.
///
/// # Returns
///
/// A `Result` containing a HashMap of FX names to parameter strings.
fn load_user_defaults(path: PathBuf) -> Result<HashMap<String, String>, X32Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut user_defaults = HashMap::new();
    let mut lines = reader.lines();

    while let Some(Ok(name_line)) = lines.next() {
        if let Some(Ok(params_line)) = lines.next() {
            user_defaults.insert(name_line.trim().to_string(), params_line.trim().to_string());
        }
    }

    Ok(user_defaults)
}

/// Resets an FX slot to its default values.
///
/// It first queries the mixer to determine the type of effect currently loaded in the
/// slot. Then it looks up the default parameters for that effect (either from the
/// built-in defaults or a user-provided file) and sends them to the mixer.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the mixer.
/// * `from` - The FX slot to reset.
/// * `defaults_file` - An optional path to a user defaults file.
fn reset_fx(socket: &UdpSocket, from: u8, defaults_file: Option<PathBuf>) -> Result<(), X32Error> {
    println!("Resetting FX slot {}.", from);

    let user_defaults: Option<HashMap<String, String>> = if let Some(path) = defaults_file {
        Some(load_user_defaults(path)?)
    } else {
        None
    };

    let fx_type_addr = format!("/fx/{}/type", from);
    let mut msg = OscMessage::new(fx_type_addr, vec![]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    let (len, _) = socket.recv_from(&mut buf)?;
    let received_msg = OscMessage::from_bytes(&buf[..len])?;

    if let Some(OscArg::Int(fx_type_id)) = received_msg.args.first() {
        let fx_name = get_fx_name_from_id(*fx_type_id)?;

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
            msg = OscMessage::new(par_addr, args);
            socket.send(&msg.to_bytes()?)?;
            Ok(())
        } else {
            Err(X32Error::from(format!(
                "Defaults not found for FX type: {}",
                fx_name
            )))
        }
    } else {
        Err(X32Error::from("Could not determine FX type.".to_string()))
    }
}

/// Maps an internal FX type ID to its string name.
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

/// Copies a single parameter from one address to another.
fn copy_param(
    socket: &UdpSocket,
    from_fx: u8,
    from_param: u8,
    to_fx: u8,
    to_param: u8,
) -> Result<(), X32Error> {
    let source_addr = format!("/fx/{}/par/{:02}", from_fx, from_param);
    let mut buf = [0; 512];
    let mut msg = OscMessage::new(source_addr, vec![]);
    socket.send(&msg.to_bytes()?)?;
    let (len, _) = socket.recv_from(&mut buf)?;
    let received_msg = OscMessage::from_bytes(&buf[..len])?;

    let dest_addr = format!("/fx/{}/par/{:02}", to_fx, to_param);
    msg = OscMessage::new(dest_addr, received_msg.args.clone());
    socket.send(&msg.to_bytes()?)?;
    Ok(())
}

/// Copies all parameters from one FX slot to another.
fn copy_fx(socket: &UdpSocket, from: u8, to: u8, master: bool) -> Result<(), X32Error> {
    if to == 0 {
        return Err(X32Error::from(
            "Destination FX slot must be provided for copy action.".to_string(),
        ));
    }
    println!("Copying FX from slot {} to {}.", from, to);
    for i in 1..32 {
        copy_param(socket, from, i, to, i)?;
        copy_param(socket, from, i + 32, to, i + 32)?;
    }
    if master {
        copy_param(socket, from, 32, to, 32)?;
        copy_param(socket, from, 64, to, 64)?;
    }
    Ok(())
}

/// Copies parameters from side A to side B within the same FX slot.
fn copy_a_to_b(socket: &UdpSocket, from: u8, master: bool) -> Result<(), X32Error> {
    println!("Copying from side A to side B of FX slot {}.", from);
    for i in 1..32 {
        copy_param(socket, from, i, from, i + 32)?;
    }
    if master {
        copy_param(socket, from, 32, from, 64)?;
    }
    Ok(())
}

/// Copies parameters from side B to side A within the same FX slot.
fn copy_b_to_a(socket: &UdpSocket, from: u8, master: bool) -> Result<(), X32Error> {
    println!("Copying from side B to side A of FX slot {}.", from);
    for i in 33..64 {
        copy_param(socket, from, i, from, i - 32)?;
    }
    if master {
        copy_param(socket, from, 64, from, 32)?;
    }
    Ok(())
}
