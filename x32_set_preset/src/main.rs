//! `x32_set_preset` is a command-line tool for loading preset files (.chn, .efx, .rou)
//! directly onto a specific target slot on a Behringer X32/M32 mixer.
//!
//! Unlike `x32_set_lib`, which uploads to the mixer's library, this tool applies the
//! preset's parameters directly to a channel strip, effects slot, or the global routing table.
//! It supports safe-loading features to exclude specific parameter groups (like Headamp gain or Config)
//! from being overwritten.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use x32_lib::create_socket;

/// Command-line arguments for `x32_set_preset`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 mixer.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// The preset file to load (.chn, .efx, .rou).
    file: PathBuf,

    /// The target slot (e.g., ch01, aux01, fx1, bus01, mtx01, main_st, main_m).
    #[arg(short, long)]
    target: Option<String>,

    // Safety Flags
    /// Skip loading Headamp/Preamp settings.
    #[arg(long)]
    safe_headamp: bool,
    /// Skip loading Config settings (Color, Icon, Name, Source).
    #[arg(long)]
    safe_config: bool,
    /// Skip loading Gate settings.
    #[arg(long)]
    safe_gate: bool,
    /// Skip loading Dynamics settings.
    #[arg(long)]
    safe_dyn: bool,
    /// Skip loading EQ settings.
    #[arg(long)]
    safe_eq: bool,
    /// Skip loading Mix/Send settings (Fader, Pan, Mute, Sends).
    #[arg(long)]
    safe_send: bool,

    /// Mute Master faders before loading to prevent audio bursts.
    #[arg(long)]
    master_safe: bool,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,
}

/// Types of presets supported by the tool.
#[derive(Debug, PartialEq)]
enum PresetType {
    /// Channel preset (.chn).
    Channel,
    /// Effects preset (.efx).
    Effect,
    /// Routing preset (.rou).
    Routing,
    /// Unknown file type.
    Unknown,
}

/// The main entry point for the application.
fn main() -> Result<()> {
    let args = Args::parse();

    // Detect preset type from extension
    let ext = args.file.extension().and_then(|s| s.to_str()).unwrap_or("");
    let preset_type = match ext {
        "chn" => PresetType::Channel,
        "efx" => PresetType::Effect,
        "rou" => PresetType::Routing,
        _ => PresetType::Unknown,
    };

    if preset_type == PresetType::Unknown {
        return Err(anyhow!(
            "Unknown file extension. Expected .chn, .efx, or .rou"
        ));
    }

    // Validate target for Channel/Effect presets
    let target_prefix = if let Some(t) = &args.target {
        parse_target(t, &preset_type)?
    } else if preset_type == PresetType::Routing {
        String::new() // Routing presets don't need a target prefix usually
    } else {
        return Err(anyhow!(
            "--target is required for Channel and Effect presets."
        ));
    };

    // Connect to X32
    println!("Connecting to X32 at {}...", args.ip);
    let socket = create_socket(&args.ip, 500)?;

    // Master Safe: Mute mains if requested
    if args.master_safe {
        println!("Muting Main L/R and M/C...");
        let msgs = vec![
            OscMessage::new("/main/st/mix/on".to_string(), vec![OscArg::Int(0)]),
            OscMessage::new("/main/m/mix/on".to_string(), vec![OscArg::Int(0)]),
        ];
        for msg in msgs {
            socket.send(&msg.to_bytes()?)?;
        }
    }

    println!("Loading preset: {:?}", args.file);
    let file = File::open(&args.file).context("Failed to open preset file")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Header check
        if line.starts_with('#') {
            if line.contains("#2.7#") || line.contains("#2.1#") || line.contains("#2.0#") {
                // Accepted versions (relaxed check compared to C)
                continue;
            }
            // Some files have comments or other headers
            continue;
        }

        // Parse line into address and arguments
        let (cmd_addr, args_str) = match line.split_once(char::is_whitespace) {
            Some((a, b)) => (a, b),
            None => (line, ""),
        };

        // Apply Safes
        if should_skip(cmd_addr, &args) {
            if args.verbose {
                println!("Skipping (safe): {}", line);
            }
            continue;
        }

        // Remap Address
        let full_address = match preset_type {
            PresetType::Channel => map_channel_address(&target_prefix, cmd_addr),
            PresetType::Effect => map_effect_address(&target_prefix, cmd_addr),
            PresetType::Routing => cmd_addr.to_string(),
            _ => cmd_addr.to_string(),
        };

        if full_address.is_empty() {
            // Mapping returned empty, meaning this command is not applicable to target
            continue;
        }

        // Parse Arguments
        let osc_args = parse_args(args_str);

        let msg = OscMessage::new(full_address, osc_args);
        if args.verbose {
            println!("Sending: {}", msg);
        }
        socket.send(&msg.to_bytes()?)?;
    }

    println!("Done.");
    Ok(())
}

/// Parses the target string into an OSC address prefix.
fn parse_target(target: &str, ptype: &PresetType) -> Result<String> {
    let t = target.to_lowercase();

    match ptype {
        PresetType::Channel => {
            if let Some(suffix) = t.strip_prefix("ch") {
                // ch01 -> /ch/01
                let id: u32 = suffix.parse()?;
                return Ok(format!("/ch/{:02}", id));
            }
            if let Some(suffix) = t.strip_prefix("aux") {
                // aux01 -> /auxin/01
                let id: u32 = suffix.parse()?;
                return Ok(format!("/auxin/{:02}", id));
            }
            if let Some(suffix) = t.strip_prefix("bus") {
                let id: u32 = suffix.parse()?;
                return Ok(format!("/bus/{:02}", id));
            }
            if let Some(suffix) = t.strip_prefix("mtx") {
                let id: u32 = suffix.parse()?;
                return Ok(format!("/mtx/{:02}", id));
            }
            if t == "main_st" || t == "st" {
                return Ok("/main/st".to_string());
            }
            if t == "main_m" || t == "m" {
                return Ok("/main/m".to_string());
            }
            if let Some(suffix) = t.strip_prefix("fxrtn") {
                let id: u32 = suffix.parse()?;
                return Ok(format!("/fxrtn/{:02}", id));
            }
            Err(anyhow!(
                "Invalid target for Channel preset. Use chXX, auxXX, busXX, mtxXX, fxrtnXX, st, or m."
            ))
        }
        PresetType::Effect => {
            if let Some(suffix) = t.strip_prefix("fx") {
                // fx1 -> /fx/1
                let id: u32 = suffix.parse()?;
                if !(1..=8).contains(&id) {
                    return Err(anyhow!("FX slot must be 1-8"));
                }
                return Ok(format!("/fx/{}", id));
            }
            Err(anyhow!("Invalid target for Effect preset. Use fx1..fx8."))
        }
        _ => Ok(String::new()),
    }
}

/// Maps a channel preset command to the target OSC address.
fn map_channel_address(prefix: &str, addr: &str) -> String {
    if addr.starts_with("/headamp") {
        // If target is a channel (ch01..ch32), map to global headamp (HA 1-32).
        // This follows the C code logic assuming 1:1 mapping.
        if prefix.starts_with("/ch/") {
            let ch_str = &prefix[4..6]; // "01"
            let parts: Vec<&str> = addr.split('/').collect();
            if parts.len() >= 4 {
                // "", "headamp", "000", "gain"
                return format!("/headamp/{}/{}", ch_str, parts[3]);
            }
        }
        return String::new(); // Skip headamp for non-physical channels or if parsing fails
    }

    format!("{}{}", prefix, addr)
}

/// Maps an effect preset command to the target OSC address.
fn map_effect_address(prefix: &str, addr: &str) -> String {
    format!("{}{}", prefix, addr)
}

/// Checks if a command should be skipped based on safety flags.
fn should_skip(addr: &str, args: &Args) -> bool {
    if args.safe_config && addr.starts_with("/config") {
        return true;
    }
    if args.safe_headamp && (addr.starts_with("/headamp") || addr.starts_with("/preamp")) {
        return true;
    }
    if args.safe_gate && addr.starts_with("/gate") {
        return true;
    }
    if args.safe_dyn && addr.starts_with("/dyn") {
        return true;
    }
    if args.safe_eq && addr.starts_with("/eq") {
        return true;
    }
    if args.safe_send && addr.starts_with("/mix") {
        return true;
    }
    false
}

/// Parses a string of arguments into `OscArg` values.
fn parse_args(s: &str) -> Vec<OscArg> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quote = false;

    for c in s.chars() {
        if c == '"' {
            in_quote = !in_quote;
            continue;
        }
        if c.is_whitespace() && !in_quote {
            if !current_arg.is_empty() {
                args.push(parse_single_arg(&current_arg));
                current_arg.clear();
            }
        } else {
            current_arg.push(c);
        }
    }
    if !current_arg.is_empty() {
        args.push(parse_single_arg(&current_arg));
    }

    args
}

/// Parses a single argument string into an `OscArg`.
fn parse_single_arg(s: &str) -> OscArg {
    if let Ok(i) = s.parse::<i32>() {
        OscArg::Int(i)
    } else if let Ok(f) = s.parse::<f32>() {
        OscArg::Float(f)
    } else {
        OscArg::String(s.to_string())
    }
}
