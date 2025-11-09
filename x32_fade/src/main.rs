//! `x32_fade` is a command-line utility for controlling and fading faders on a
//! Behringer X32 or Midas M32 digital mixing console.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use x32_lib::{create_socket, get_fader_level};
use osc_lib::{OscMessage, OscArg};
use anyhow::Result;

/// A command-line tool to control and fade X32 faders.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 console.
    #[arg(short, long)]
    ip: Option<String>,

    /// A fader to control. Can be specified multiple times.
    /// E.g., --fader /ch/01/mix/fader
    #[arg(long)]
    fader: Vec<String>,

    /// Fade-in time in seconds.
    #[arg(long)]
    fade_in: Option<f32>,

    /// Fade-out time in seconds.
    #[arg(long)]
    fade_out: Option<f32>,

    /// Number of steps for the fade.
    #[arg(long)]
    steps: Option<u32>,

    /// Path to a config file to load.
    #[arg(long)]
    load_config: Option<PathBuf>,

    /// Path to save the config file to.
    #[arg(long)]
    save_config: Option<PathBuf>,

    /// Verbose mode.
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Debug mode.
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

/// Represents the configuration for a fade operation, serializable to/from JSON.
#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    /// The IP address of the X32 console.
    ip: Option<String>,
    /// A list of OSC fader addresses to control.
    faders: Vec<String>,
    /// The duration of the fade-in in seconds.
    fade_in: Option<f32>,
    /// The duration of the fade-out in seconds.
    fade_out: Option<f32>,
    /// The number of steps to use for the fade.
    steps: Option<u32>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut config = Config::default();

    // Load configuration from a file if specified.
    if let Some(path) = &args.load_config {
        let data = fs::read_to_string(path)?;
        config = serde_json::from_str(&data)?;
        if args.debug {
            println!("Loaded config from {:?}: {:?}", path, config);
        }
    }

    // Command-line arguments override config file values.
    if args.ip.is_some() {
        config.ip = args.ip;
    }
    if !args.fader.is_empty() {
        config.faders = args.fader;
    }
    if args.fade_in.is_some() {
        config.fade_in = args.fade_in;
    }
    if args.fade_out.is_some() {
        config.fade_out = args.fade_out;
    }
    if args.steps.is_some() {
        config.steps = args.steps;
    }

    // If a save path is specified, write the config and exit.
    if let Some(path) = &args.save_config {
        let data = serde_json::to_string_pretty(&config)?;
        fs::write(path, data)?;
        if args.verbose {
            println!("Saved config to {:?}", path);
        }
        return Ok(());
    }

    // Ensure we have an IP address before proceeding.
    if let Some(ip) = &config.ip {
        let socket = create_socket(ip, 1000)?;
        if args.verbose {
            println!("Connected to X32 at {}", ip);
        }

        let steps = config.steps.unwrap_or(32);

        // Perform fade-in if specified.
        if let Some(fade_in_duration) = config.fade_in {
            if fade_in_duration > 0.0 {
                if args.verbose {
                    println!(
                        "Fading in over {} seconds in {} steps.",
                        fade_in_duration, steps
                    );
                }
                fade(&socket, &config.faders, fade_in_duration, steps, true, args.verbose)?;
            }
        }

        // Perform fade-out if specified.
        if let Some(fade_out_duration) = config.fade_out {
            if fade_out_duration > 0.0 {
                if args.verbose {
                    println!(
                        "Fading out over {} seconds in {} steps.",
                        fade_out_duration, steps
                    );
                }
                fade(&socket, &config.faders, fade_out_duration, steps, false, args.verbose)?;
            }
        }
    } else {
        println!("Error: No IP address specified either via --ip or --load-config.");
    }

    Ok(())
}

/// Performs a fade operation on a set of faders.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the X32.
/// * `faders` - A slice of strings representing the OSC addresses of the faders to control.
/// * `duration_s` - The duration of the fade in seconds.
/// * `steps` - The number of steps to use for the fade.
/// * `is_fade_in` - If `true`, performs a fade-in (to 0.75). If `false`, performs a fade-out (to 0.0).
/// * `verbose` - If `true`, prints the OSC messages being sent.
fn fade(
    socket: &std::net::UdpSocket,
    faders: &[String],
    duration_s: f32,
    steps: u32,
    is_fade_in: bool,
    verbose: bool,
) -> Result<()> {
    if steps == 0 || duration_s <= 0.0 {
        return Ok(());
    }

    // Query the initial level of each fader.
    let mut initial_levels = Vec::new();
    for fader_addr in faders {
        initial_levels.push(get_fader_level(socket, fader_addr)?);
    }

    let target_level = if is_fade_in { 0.75 } else { 0.0 };
    let step_interval = Duration::from_secs_f32(duration_s / steps as f32);

    for i in 1..=steps {
        for (fader_addr, &initial_level) in faders.iter().zip(&initial_levels) {
            let current_level = if is_fade_in {
                initial_level + ((target_level - initial_level) / steps as f32) * i as f32
            } else {
                initial_level - ((initial_level - target_level) / steps as f32) * i as f32
            };

            let msg = OscMessage::new(fader_addr.clone(), vec![OscArg::Float(current_level)]);
            let buf = msg.to_bytes()?;
            socket.send(&buf)?;
            if verbose {
                println!("Sent: {} {}", fader_addr, current_level);
            }
        }

        thread::sleep(step_interval);
    }

    // Send the final target level to ensure accuracy.
    for fader_addr in faders {
         let msg = OscMessage::new(fader_addr.clone(), vec![OscArg::Float(target_level)]);
         let buf = msg.to_bytes()?;
         socket.send(&buf)?;
         if verbose {
             println!("Sent final: {} {}", fader_addr, target_level);
         }
    }

    Ok(())
}
