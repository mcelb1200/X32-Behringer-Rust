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
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use anyhow::Result;
use clap::{Parser, ValueEnum};
use tokio::time::{sleep, Duration};
use x32_lib::{get_parameter_async, set_parameter_async, verify_fx_type_async, MixerClient};

/// Command-line arguments for the `x32_geq2_cpy` tool.
#[derive(Parser, Debug)]
#[command(author, version, about = "A utility to copy GEQ settings between FX slots on an X32 console.", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    #[arg(long, default_value = "auto")]
    pub transport: String,

    #[arg(long, default_value = "")]
    pub usb_port: String,

    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    /// Source FX slot number (1-8).
    #[arg(short, long, default_value_t = 1)]
    pub from: u8,

    /// Destination FX slot number (1-8).
    #[arg(short, long, default_value_t = 1)]
    pub to: u8,

    /// Copy direction.
    #[arg(short, long, value_enum, default_value_t = Direction::AtoB)]
    pub direction: Direction,

    /// Copy master level.
    #[arg(short, long, default_value_t = true)]
    pub master: bool,

    /// Verbose mode.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Debug mode.
    #[arg(short = 'D', long, default_value_t = false)]
    pub debug: bool,
}

/// Enumerates the possible directions for the copy operation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Direction {
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
pub async fn run(args: Args) -> Result<()> {
    if args.debug {
        println!("Debug mode is on.");
        println!("Arguments: {:?}", args);
    }

    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    )
    .await?;
    let client = std::sync::Arc::new(client);
    if args.verbose {
        println!("Connected to X32 at {}", args.ip);
    }

    // Verify that the source slot has a GEQ2 or TEQ2 effect.
    if !verify_fx_type_async(&client, args.from, "EQ").await? {
        println!("--!!-- No GEQ2/TEQ2 effect at FX slot #{}", args.from);
        return Ok(());
    }

    // If we're copying to another slot, verify the destination slot as well.
    if args.direction == Direction::CopyTo && !verify_fx_type_async(&client, args.to, "EQ").await? {
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
                let value = get_parameter_async(&client, &source_addr).await?;
                set_parameter_async(&client, &dest_addr, value).await?;
                if args.verbose {
                    println!("Copied {} to {}", source_addr, dest_addr);
                }
            }
            if args.master {
                let source_addr = format!("/fx/{}/par/32", args.from);
                let dest_addr = format!("/fx/{}/par/64", args.from);
                let value = get_parameter_async(&client, &source_addr).await?;
                set_parameter_async(&client, &dest_addr, value).await?;
                if args.verbose {
                    println!("Copied master level.");
                }
            }
        }
        Direction::BtoA => {
            for i in 33..64 {
                let source_addr = format!("/fx/{}/par/{:02}", args.from, i);
                let dest_addr = format!("/fx/{}/par/{:02}", args.from, i - 32);
                let value = get_parameter_async(&client, &source_addr).await?;
                set_parameter_async(&client, &dest_addr, value).await?;
                if args.verbose {
                    println!("Copied {} to {}", source_addr, dest_addr);
                }
            }
            if args.master {
                let source_addr = format!("/fx/{}/par/64", args.from);
                let dest_addr = format!("/fx/{}/par/32", args.from);
                let value = get_parameter_async(&client, &source_addr).await?;
                set_parameter_async(&client, &dest_addr, value).await?;
                if args.verbose {
                    println!("Copied master level.");
                }
            }
        }
        Direction::Reset => {
            for i in 1..64 {
                let addr = format!("/fx/{}/par/{:02}", args.from, i);
                set_parameter_async(&client, &addr, 0.5).await?;
                if args.verbose {
                    println!("Reset {}", addr);
                }
                sleep(Duration::from_millis(10)).await;
            }
            if args.master {
                let addr_a = format!("/fx/{}/par/32", args.from);
                let addr_b = format!("/fx/{}/par/64", args.from);
                set_parameter_async(&client, &addr_a, 0.5).await?;
                set_parameter_async(&client, &addr_b, 0.5).await?;
                if args.verbose {
                    println!("Reset master levels.");
                }
            }
        }
        Direction::CopyTo => {
            for i in 1..64 {
                let source_addr = format!("/fx/{}/par/{:02}", args.from, i);
                let dest_addr = format!("/fx/{}/par/{:02}", args.to, i);
                let value = get_parameter_async(&client, &source_addr).await?;
                set_parameter_async(&client, &dest_addr, value).await?;
                if args.verbose {
                    println!("Copied {} to {}", source_addr, dest_addr);
                }
            }
            if args.master {
                let source_addr_a = format!("/fx/{}/par/32", args.from);
                let source_addr_b = format!("/fx/{}/par/64", args.from);
                let dest_addr_a = format!("/fx/{}/par/32", args.to);
                let dest_addr_b = format!("/fx/{}/par/64", args.to);
                let value_a = get_parameter_async(&client, &source_addr_a).await?;
                let value_b = get_parameter_async(&client, &source_addr_b).await?;
                set_parameter_async(&client, &dest_addr_a, value_a).await?;
                set_parameter_async(&client, &dest_addr_b, value_b).await?;
                if args.verbose {
                    println!("Copied master levels.");
                }
            }
        }
    }

    println!("Operation completed successfully.");

    Ok(())
}
