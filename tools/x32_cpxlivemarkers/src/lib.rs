use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about = "Reads markers from an XLive! session/SE_LOG.BIN file", long_about = None)]
pub struct Cli {
    /// The path to the SE_LOG.BIN file
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Output format (reaper or audition)
    #[arg(short, long, value_enum, default_value_t = Format::Reaper)]
    pub format: Format,

    /// Prefix to use for marker names
    #[arg(short, long, default_value = "")]
    pub prefix: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Format {
    Reaper,
    Audition,
}

pub fn format_marker(
    format: Format,
    index: u32,
    marker_pos_samples: u32,
    samprate: u32,
    prefix: &str,
) -> Result<String> {
    if samprate == 0 {
        return Err(anyhow!("Invalid sample rate (0)"));
    }

    let pos = marker_pos_samples as f64 / samprate as f64;

    match format {
        Format::Reaper => Ok(format!("{} {:.6} {}{} 0 -1.0 0", index, pos, prefix, index)),
        Format::Audition => {
            let hh = (pos / 3600.0) as u32;
            let mm = ((pos % 3600.0) / 60.0) as u32;
            let ss = (pos % 60.0) as u32;
            let ff = ((pos - pos.floor()) * 100.0 / 4.0) as u32;
            Ok(format!(
                "{}{}, {:02}:{:02}:{:02}:{:02}, 00:00:00:00, 25fps, Cue, -",
                prefix, index, hh, mm, ss, ff
            ))
        }
    }
}

pub fn process_file<R: Read + Seek>(mut file: R, cli: &Cli) -> Result<Vec<String>> {
    file.seek(SeekFrom::Start(8))?;
    let samprate = file.read_u32::<LittleEndian>()?;

    if samprate == 0 {
        return Err(anyhow!("Invalid sample rate (0)"));
    }

    file.seek(SeekFrom::Start(20))?;
    let mut nbmarker = file.read_u32::<LittleEndian>()?;

    if nbmarker == 0 {
        return Ok(vec![]);
    }

    if nbmarker > 500 {
        nbmarker = 500; // Bound to prevent DoS
    }

    file.seek(SeekFrom::Start(1052))?;

    let mut output = Vec::new();
    for i in 1..=nbmarker {
        let marker = file.read_u32::<LittleEndian>()?;
        let formatted = format_marker(cli.format, i, marker, samprate, &cli.prefix)?;
        output.push(formatted);
    }

    Ok(output)
}

pub fn run(cli: Cli) -> Result<()> {
    let mut file = match File::open(&cli.file) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Failed to open file: {}", cli.file.display());
            process::exit(1);
        }
    };

    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(_) => {
            eprintln!("Failed to open file: {}", cli.file.display());
            process::exit(1);
        }
    };

    if metadata.len() < 2048 {
        eprintln!("File is too small to be a valid SE_LOG.BIN file");
        process::exit(1);
    }

    if metadata.len() > 1024 * 1024 {
        eprintln!("File is too large to be a valid SE_LOG.BIN file");
        process::exit(1);
    }

    let markers = match process_file(&mut file, &cli) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error processing file: {}", e);
            process::exit(1);
        }
    };

    if markers.is_empty() {
        println!("No Markers");
    } else {
        for marker in markers {
            println!("{}", marker);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_reaper() {
        let res = format_marker(Format::Reaper, 1, 48000, 48000, "M").unwrap();
        assert_eq!(res, "1 1.000000 M1 0 -1.0 0");

        let res2 = format_marker(Format::Reaper, 2, 96000, 48000, "M").unwrap();
        assert_eq!(res2, "2 2.000000 M2 0 -1.0 0");
    }

    #[test]
    fn test_format_audition() {
        let res = format_marker(Format::Audition, 1, 48000 * 3661, 48000, "C").unwrap();
        assert_eq!(res, "C1, 01:01:01:00, 00:00:00:00, 25fps, Cue, -");

        let frames = 48000 / 2; // 0.5 seconds = 12.5 frames? No, the math is (pos - pos.floor()) * 100 / 4
                                // pos = 0.5. 0.5 * 100 = 50. 50 / 4 = 12.
        let res2 = format_marker(Format::Audition, 2, frames, 48000, "C").unwrap();
        assert_eq!(res2, "C2, 00:00:00:12, 00:00:00:00, 25fps, Cue, -");
    }

    #[test]
    fn test_zero_samprate() {
        let res = format_marker(Format::Reaper, 1, 48000, 0, "M");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid sample rate (0)");
    }
}
