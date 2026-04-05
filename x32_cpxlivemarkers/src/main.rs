use anyhow::{Context, Result, bail};
use byteorder::{LittleEndian, ReadBytesExt};
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

/// Formats for marker output
#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum Format {
    /// REAPER format
    Reaper,
    /// Audition format
    Audition,
}

/// Reads markers from an X-Live! session/SE_LOG.BIN file and prints data to stdout.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the SE_LOG.BIN file.
    input: PathBuf,

    /// Output format.
    #[arg(short, long, value_enum, default_value_t = Format::Reaper)]
    format: Format,

    /// Marker prefix.
    #[arg(short, long, default_value = "")]
    prefix: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut file = File::open(&args.input)
        .with_context(|| format!("Failed to open file: {}", args.input.display()))?;

    // Check file size to prevent OOM / DoS from reading huge invalid files
    let metadata = file.metadata()?;
    let len = metadata.len();

    // SE_LOG.BIN is expected to be around 2048 bytes
    if len < 2048 {
        bail!("File is too small to be a valid SE_LOG.BIN file");
    }

    if len > 1024 * 1024 {
        bail!("File is too large to be a valid SE_LOG.BIN file");
    }

    // Offset 8: sample rate
    file.seek(SeekFrom::Start(8))?;
    let samprate = file.read_u32::<LittleEndian>()?;

    if samprate == 0 {
        bail!("Invalid sample rate (0) in SE_LOG.BIN file");
    }

    // Offset 20: number of markers
    file.seek(SeekFrom::Start(20))?;
    let nbmarker = file.read_u32::<LittleEndian>()?;

    if nbmarker == 0 {
        println!("No Markers");
        return Ok(());
    }

    // Markers start at offset 1052
    file.seek(SeekFrom::Start(1052))?;

    let prefix = args.prefix.chars().take(5).collect::<String>();

    for i in 1..=nbmarker {
        let marker = file.read_u32::<LittleEndian>()?;
        let xmk = marker as f32 / samprate as f32;

        match args.format {
            Format::Reaper => {
                println!("{} {:.6} {}{} 0 -1.0 0", i, xmk, prefix, i);
            }
            Format::Audition => {
                let xmk_int = xmk as i32;
                let xmkh = xmk_int / 3600;
                let xmkm = (xmk_int / 60) % 60; // Fixed bug from original C code
                let xmks = xmk_int % 60;
                let xmkt = ((xmk - xmk_int as f32) * 100.0 / 4.0) as i32; // conversion for 25fps

                println!(
                    "{}{}, {:02}:{:02}:{:02}:{:02}, 00:00:00:00, 25fps, Cue, -",
                    prefix, i, xmkh, xmkm, xmks, xmkt
                );
            }
        }
    }

    Ok(())
}
