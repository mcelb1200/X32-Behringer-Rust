//! `x32_set_preset` is a command-line tool for loading channel, effect, and routing presets
//! to a Behringer X32 or Midas M32 digital mixing console.

use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::UdpSocket;
use std::path::Path;
use osc_lib::{OscArg, OscMessage, tokenize};

/// A command-line tool for loading presets to an X32/M32 mixer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 mixer
    #[arg(long)]
    ip: String,

    /// Port of the X32 mixer
    #[arg(long, default_value_t = 10023)]
    port: u16,

    /// Preset file to load
    #[arg(short, long)]
    file: String,

    /// Destination slot for the preset
    #[arg(short, long, default_value_t = 1)]
    slot: u8,

    /// Master levels safe
    #[arg(long)]
    master_safe: bool,

    /// Preamp settings safe
    #[arg(long)]
    preamp_safe: bool,

    /// Channel config safe
    #[arg(long)]
    config_safe: bool,

    /// Gate settings safe
    #[arg(long)]
    gate_safe: bool,

    /// Dynamics settings safe
    #[arg(long)]
    dynamics_safe: bool,

    /// EQ settings safe
    #[arg(long)]
    eq_safe: bool,

    /// Sends settings safe
    #[arg(long)]
    sends_safe: bool,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,
}

/// The type of preset file.
enum PresetType {
    Channel,
    Effect,
    Routing,
}

/// The main entry point for the `x32_set_preset` utility.
///
/// This function parses command-line arguments, determines the preset type,
/// reads the preset file, and sends the appropriate OSC messages to the mixer.
fn main() -> Result<()> {
    let args = Args::parse();

    let remote_addr = format!("{}:{}", args.ip, args.port);
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(remote_addr)?;

    if args.master_safe {
        let msg = OscMessage::new("/main/st/mix/fader".to_string(), vec![OscArg::Float(0.0)]);
        socket.send(&msg.to_bytes()?)?;
        let msg = OscMessage::new("/main/m/mix/fader".to_string(), vec![OscArg::Float(0.0)]);
        socket.send(&msg.to_bytes()?)?;
    }

    let path = Path::new(&args.file);
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let preset_type = match extension {
        "chn" => PresetType::Channel,
        "efx" => PresetType::Effect,
        "rou" => PresetType::Routing,
        _ => return Err(anyhow!("Unsupported preset file type")),
    };

    let file = File::open(&args.file)?;
    let reader = BufReader::new(file);

    let base_path = get_base_path(args.slot, &preset_type)?;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') {
            continue;
        }

        let tokens = tokenize(&line)?;
        if tokens.is_empty() {
            continue;
        }

        let osc_path = &tokens[0];

        if should_skip(&osc_path, &args, &preset_type) {
            continue;
        }

        let full_path = match preset_type {
            PresetType::Channel | PresetType::Effect => format!("{}{}", base_path, osc_path),
            PresetType::Routing => osc_path.to_string(),
        };

        let parsed_args = parse_args(&tokens[1..])?;
        let msg = OscMessage::new(full_path, parsed_args);

        if args.verbose {
            println!("Sending: {}", msg.to_string());
        }
        socket.send(&msg.to_bytes()?)?;
    }

    Ok(())
}

/// Parses the arguments from a line in a preset file.
///
/// This function handles both explicit and inferred argument types.
fn parse_args(parts: &[String]) -> Result<Vec<OscArg>> {
    let mut args = Vec::new();
    let mut type_tags: Option<&str> = None;
    let mut values: Vec<&String> = Vec::new();

    for part in parts {
        if part.starts_with(',') {
            if type_tags.is_some() {
                return Err(anyhow!("Multiple type tag strings found in arguments: {:?}", parts));
            }
            type_tags = Some(&part[1..]);
        } else {
            values.push(part);
        }
    }

    if let Some(tags) = type_tags {
        if tags.len() != values.len() {
            return Err(anyhow!("Mismatch between number of type tags ({}) and values ({}).", tags.len(), values.len()));
        }
        for (i, tag) in tags.chars().enumerate() {
            let val_str = values[i];
            let arg = match tag {
                'i' => OscArg::Int(val_str.parse()?),
                'f' => OscArg::Float(val_str.parse()?),
                's' => OscArg::String(val_str.trim_matches('"').to_string()),
                _ => return Err(anyhow!("Unsupported OSC type tag: {}", tag)),
            };
            args.push(arg);
        }
    } else {
        // No type tags, infer types
        for val_str in values {
            let arg = if let Ok(i) = val_str.parse::<i32>() {
                OscArg::Int(i)
            } else if let Ok(f) = val_str.parse::<f32>() {
                OscArg::Float(f)
            } else {
                OscArg::String(val_str.trim_matches('"').to_string())
            };
            args.push(arg);
        }
    }

    Ok(args)
}

/// Gets the base OSC path for a given preset type and slot.
fn get_base_path(slot: u8, preset_type: &PresetType) -> Result<String> {
    match preset_type {
        PresetType::Channel => {
            match slot {
                1..=32 => Ok(format!("/ch/{:02}", slot)),
                33..=40 => Ok(format!("/auxin/{:02}", slot - 32)),
                41..=48 => Ok(format!("/fxrtn/{:02}", slot - 40)),
                49..=64 => Ok(format!("/bus/{:02}", slot - 48)),
                65..=70 => Ok(format!("/mtx/{:02}", slot - 64)),
                71 => Ok("/main/st".to_string()),
                72 => Ok("/main/m".to_string()),
                _ => Err(anyhow!("Invalid destination slot for channel preset")),
            }
        },
        PresetType::Effect => {
            if (1..=8).contains(&slot) {
                Ok(format!("/fx/{}/", slot))
            } else {
                Err(anyhow!("Invalid destination slot for effect preset"))
            }
        },
        PresetType::Routing => Ok("".to_string()),
    }
}

/// Determines whether an OSC message should be skipped based on the "safe" flags.
fn should_skip(path: &str, args: &Args, preset_type: &PresetType) -> bool {
    match preset_type {
        PresetType::Channel => {
            (args.preamp_safe && path.starts_with("/preamp")) ||
            (args.preamp_safe && path.starts_with("/headamp")) ||
            (args.config_safe && path.starts_with("/config")) ||
            (args.config_safe && path.starts_with("/delay")) ||
            (args.gate_safe && path.starts_with("/gate")) ||
            (args.dynamics_safe && path.starts_with("/dyn")) ||
            (args.eq_safe && path.starts_with("/eq")) ||
            (args.sends_safe && path.starts_with("/mix"))
        },
        _ => false
    }
}
