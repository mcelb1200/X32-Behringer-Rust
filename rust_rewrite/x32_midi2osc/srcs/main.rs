use clap::Parser;
use std::path::PathBuf;

mod config;
mod midi;

/// A utility to translate MIDI commands to OSC messages for the Behringer X32/M32 consoles.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    ip: String,

    /// MIDI input port number
    #[arg(long)]
    midi_in: u8,

    /// MIDI output port number for thru functionality
    #[arg(long)]
    midi_out: Option<u8>,

    /// Path to the configuration file
    #[arg(short, long)]
    config: PathBuf,

    /// Enable debug mode to print MIDI messages
    #[arg(long)]
    debug: bool,

    /// List available MIDI ports and exit
    #[arg(long)]
    list_midi: bool,
}

fn main() {
    let args = Args::parse();

    if args.list_midi {
        midi::list_midi_ports();
        return;
    }

    println!("X32Midi2OSC started with the following configuration:");
    println!("  X32 IP: {}", args.ip);
    println!("  MIDI In Port: {}", args.midi_in);
    if let Some(midi_out) = args.midi_out {
        println!("  MIDI Out Port: {}", midi_out);
    }
    println!("  Config File: {}", args.config.display());
    println!("  Debug Mode: {}", if args.debug { "On" } else { "Off" });

    match config::parse_config(&args.config) {
        Ok(commands) => {
            println!("Successfully parsed {} MIDI to OSC commands.", commands.len());
        }
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            std::process::exit(1);
        }
    }

    midi::run_midi(args.midi_in, args.midi_out, args.debug);
}
