//! `x32_geq2_cpy` is a command-line utility for copying Graphic EQ (GEQ) settings on an
//! X32/M32 digital mixer.
//!
//! It specifically targets the Dual GEQ (GEQ2) and Dual TruEQ (TEQ2) effects, allowing
//! users to:
//! - Copy settings from side A to side B (or vice versa) within the same FX slot.
//! - Reset the EQ curves to flat.
//! - Copy the entire EQ configuration from one FX slot to another.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]

use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::thread;
use std::time::Duration;
use x32_lib::{create_socket, get_parameter, set_parameter, verify_fx_type};

/// Command-line arguments for the `x32_geq2_cpy` tool.
#[derive(Parser, Debug)]
#[command(author, version, about = "A utility to copy GEQ settings between FX slots on an X32 console.", long_about = None)]
struct Args {
    /// IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Source FX slot number (1-8).
    #[arg(short, long, default_value_t = 1)]
    from: u8,

    /// Destination FX slot number (1-8).
    #[arg(short, long, default_value_t = 1)]
    to: u8,

    /// Copy direction.
    #[arg(short, long, value_enum, default_value_t = Direction::AtoB)]
    direction: Direction,

    /// Copy master level.
    #[arg(short, long, default_value_t = true)]
    master: bool,

    /// Verbose mode.
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Debug mode.
    #[arg(short = 'D', long, default_value_t = false)]
    debug: bool,
}

/// Enumerates the possible directions for the copy operation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Direction {
    /// Copy from side A to side B of the source slot.
    AtoB,
    /// Copy from side B to side A of the source slot.
    BtoA,
    /// Reset both sides of the source slot.
    Reset,
    /// Copy the entire source slot to the destination slot.
    CopyTo,
}

/// The main entry point for the application.
fn main() -> Result<()> {
    let args = Args::parse();

    if args.debug {
        println!("Debug mode is on.");
        println!("Arguments: {:?}", args);
    }

    let socket = create_socket(&args.ip, 1000)?;
    if args.verbose {
        println!("Connected to X32 at {}", args.ip);
    }

    // Verify that the source slot has a GEQ2 or TEQ2 effect.
    if !verify_fx_type(&socket, args.from, "EQ")? {
        println!("--!!-- No GEQ2/TEQ2 effect at FX slot #{}", args.from);
        return Ok(());
    }

    // If we're copying to another slot, verify the destination slot as well.
    if args.direction == Direction::CopyTo && !verify_fx_type(&socket, args.to, "EQ")? {
        println!("--!!-- No GEQ2/TEQ2 effect at FX slot #{}", args.to);
        return Ok(());
    }

    if args.verbose {
        println!("GEQ2/TEQ2 effect verified. Proceeding with operation.");
    }

    match args.direction {
        Direction::AtoB => {
            for i in 1..32 {
                let source_addr = format!("/fx/{}/par/{:02}", args.from, i);
                let dest_addr = format!("/fx/{}/par/{:02}", args.from, i + 32);
                let value = get_parameter(&socket, &source_addr)?;
                set_parameter(&socket, &dest_addr, value)?;
                if args.verbose {
                    println!("Copied {} to {}", source_addr, dest_addr);
                }
            }
            if args.master {
                let source_addr = format!("/fx/{}/par/32", args.from);
                let dest_addr = format!("/fx/{}/par/64", args.from);
                let value = get_parameter(&socket, &source_addr)?;
                set_parameter(&socket, &dest_addr, value)?;
                if args.verbose {
                    println!("Copied master level.");
                }
            }
        }
        Direction::BtoA => {
            for i in 33..64 {
                let source_addr = format!("/fx/{}/par/{:02}", args.from, i);
                let dest_addr = format!("/fx/{}/par/{:02}", args.from, i - 32);
                let value = get_parameter(&socket, &source_addr)?;
                set_parameter(&socket, &dest_addr, value)?;
                if args.verbose {
                    println!("Copied {} to {}", source_addr, dest_addr);
                }
            }
            if args.master {
                let source_addr = format!("/fx/{}/par/64", args.from);
                let dest_addr = format!("/fx/{}/par/32", args.from);
                let value = get_parameter(&socket, &source_addr)?;
                set_parameter(&socket, &dest_addr, value)?;
                if args.verbose {
                    println!("Copied master level.");
                }
            }
        }
        Direction::Reset => {
            for i in 1..64 {
                let addr = format!("/fx/{}/par/{:02}", args.from, i);
                set_parameter(&socket, &addr, 0.5)?;
                if args.verbose {
                    println!("Reset {}", addr);
                }
                thread::sleep(Duration::from_millis(10));
            }
            if args.master {
                let addr_a = format!("/fx/{}/par/32", args.from);
                let addr_b = format!("/fx/{}/par/64", args.from);
                set_parameter(&socket, &addr_a, 0.5)?;
                set_parameter(&socket, &addr_b, 0.5)?;
                if args.verbose {
                    println!("Reset master levels.");
                }
            }
        }
        Direction::CopyTo => {
            for i in 1..64 {
                let source_addr = format!("/fx/{}/par/{:02}", args.from, i);
                let dest_addr = format!("/fx/{}/par/{:02}", args.to, i);
                let value = get_parameter(&socket, &source_addr)?;
                set_parameter(&socket, &dest_addr, value)?;
                if args.verbose {
                    println!("Copied {} to {}", source_addr, dest_addr);
                }
            }
            if args.master {
                let source_addr_a = format!("/fx/{}/par/32", args.from);
                let source_addr_b = format!("/fx/{}/par/64", args.from);
                let dest_addr_a = format!("/fx/{}/par/32", args.to);
                let dest_addr_b = format!("/fx/{}/par/64", args.to);
                let value_a = get_parameter(&socket, &source_addr_a)?;
                let value_b = get_parameter(&socket, &source_addr_b)?;
                set_parameter(&socket, &dest_addr_a, value_a)?;
                set_parameter(&socket, &dest_addr_b, value_b)?;
                if args.verbose {
                    println!("Copied master levels.");
                }
            }
        }
    }

    println!("Operation completed successfully.");

    Ok(())
}
