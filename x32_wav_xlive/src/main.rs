//! `x32_wav_xlive` is a command-line utility for merging multiple mono WAV files
//! into a multi-channel WAV format compatible with the Behringer X-Live! SD card recorder.
//!
//! It is a Rust rewrite of the original C program `X32Wav_Xlive.c` by Patrick-Gilles Maillot.
//!
//! ## Usage
//!
//! ```sh
//! x32_wav_xlive <SESSION_DIR> [SESSION_NAME] [-f <MARKER_FILE>] [-m <MARKER>] [-u] [-S]
//! ```
//!
//! The program takes a directory of mono, 24-bit WAV files (named `ch_1.wav`, `ch_2.wav`, etc.)
//! and creates a new session directory containing one or more multi-channel, 32-bit WAV files
//! and a `SE_LOG.BIN` metadata file.

use anyhow::{Result, anyhow};
use byteorder::{LittleEndian, WriteBytesExt};
use chrono::{Datelike, Timelike, Utc};
use clap::Parser;
use hound::{WavReader, WavSpec, WavWriter};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// A utility to merge mono WAV files into a multi-channel X-Live! session.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the directory containing the mono WAV files (ch_1.wav, ch_2.wav, etc.).
    #[arg(value_name = "SESSION_DIR")]
    session_dir: PathBuf,

    /// Optional name for the X-Live! session.
    #[arg(value_name = "SESSION_NAME")]
    session_name: Option<String>,

    /// Path to a file containing a list of markers (one per line, in seconds).
    #[arg(short = 'f', long, value_name = "MARKER_FILE")]
    marker_file: Option<PathBuf>,

    /// A single marker time (in seconds). Can be specified multiple times.
    #[arg(short = 'm', long, value_name = "MARKER", action = clap::ArgAction::Append)]
    markers: Vec<f32>,

    /// Use uppercase .WAV file extension instead of .wav.
    #[arg(short = 'u', long)]
    uppercase: bool,

    /// Suppress non-error output.
    #[arg(short = 'S', long)]
    silent: bool,
}

/// The main entry point of the application.
fn main() -> Result<()> {
    let args = Args::parse();
    run(&args)
}

/// The main logic of the application.
///
/// This function finds and validates the input WAV files, creates the session directory
/// and metadata file, and then merges the audio data into one or more multi-channel WAV files.
fn run(args: &Args) -> Result<()> {
    let mut input_files = Vec::new();
    let mut first_spec = None;
    let mut first_duration = 0;

    for i in 1..=32 {
        let filename = if args.uppercase {
            format!("CH_{}.WAV", i)
        } else {
            format!("ch_{}.wav", i)
        };
        let path = args.session_dir.join(filename);

        if !path.exists() {
            // Stop searching for files as soon as one is missing.
            break;
        }

        let reader = WavReader::open(&path)?;
        let spec = reader.spec();
        let duration = reader.duration();

        if first_spec.is_none() {
            first_spec = Some(spec);
            first_duration = duration;
        }

        validate_wav_file(
            &path,
            spec,
            duration,
            first_spec.as_ref().unwrap(),
            first_duration,
        )?;
        input_files.push(path);
    }

    if input_files.is_empty() {
        return Err(anyhow!("No WAV files found in the specified directory."));
    }

    if !args.silent {
        println!("Found {} WAV files to process.", input_files.len());
    }

    let session_timestamp = create_session_timestamp()?;
    let session_dir_name = format!("{:08X}", session_timestamp);
    let session_path = args.session_dir.join(&session_dir_name);
    fs::create_dir_all(&session_path)?;

    if !args.silent {
        println!("Created session directory: {}", session_path.display());
    }

    let num_channels = input_files.len() as u32;
    let total_length = first_duration;
    let audio_bytes = total_length as u64 * 4 * num_channels as u64;
    let max_take_size = 4294901760u64; // 4GB - 32KB - 32KB header

    let mut take_sizes = Vec::new();
    let mut remaining_bytes = audio_bytes;
    while remaining_bytes > 0 {
        let take_bytes = std::cmp::min(remaining_bytes, max_take_size);
        take_sizes.push((take_bytes / 4) as u32);
        remaining_bytes -= take_bytes;
    }

    write_se_log_bin(
        &session_path,
        session_timestamp,
        num_channels,
        first_spec.as_ref().unwrap().sample_rate,
        total_length,
        &take_sizes,
        args,
    )?;

    write_wav_takes(
        &session_path,
        &input_files,
        first_spec.as_ref().unwrap(),
        &take_sizes,
        args,
    )?;

    Ok(())
}

/// Merges the audio data from the input files into one or more multi-channel WAV files.
///
/// The output files are split into "takes" to keep their size below the ~4GB limit
/// of the FAT32 file system used by the X-Live! card.
fn write_wav_takes(
    session_path: &Path,
    input_files: &[PathBuf],
    spec: &WavSpec,
    take_sizes: &[u32],
    args: &Args,
) -> Result<()> {
    let num_channels = input_files.len();
    let mut readers: Vec<_> = input_files
        .iter()
        .map(|path| WavReader::open(path).unwrap())
        .collect();

    for (i, take_size_samples) in take_sizes.iter().enumerate() {
        let filename = if args.uppercase {
            format!("{:08X}.WAV", i + 1)
        } else {
            format!("{:08X}.wav", i + 1)
        };
        let path = session_path.join(filename);

        let out_spec = WavSpec {
            channels: num_channels as u16,
            sample_rate: spec.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&path, out_spec)?;

        let samples_to_write = *take_size_samples as usize / num_channels;

        for _ in 0..samples_to_write {
            for reader in &mut readers {
                let sample = reader.samples::<i32>().next().unwrap()?;
                writer.write_sample(sample)?;
            }
        }
        writer.finalize()?;
    }

    Ok(())
}

/// Creates a timestamp for the session directory name, matching the format used by the X32.
fn create_session_timestamp() -> Result<u32> {
    let now = Utc::now();
    let session_timestamp = (((now.year() - 1980) as u32) << 25)
        | (now.month() << 21)
        | (now.day() << 16)
        | (now.hour() << 11)
        | (now.minute() << 5)
        | (now.second() / 2);
    Ok(session_timestamp)
}

/// Writes the `SE_LOG.BIN` metadata file for the session.
///
/// This file contains information about the session, including the number of channels,
/// sample rate, markers, and take sizes.
fn write_se_log_bin(
    session_path: &Path,
    session_timestamp: u32,
    num_channels: u32,
    sample_rate: u32,
    duration_samples: u32,
    take_sizes: &[u32],
    args: &Args,
) -> Result<()> {
    let log_path = session_path.join("SE_LOG.BIN");
    let mut file = File::create(log_path)?;

    let mut markers = args.markers.clone();
    if let Some(marker_file) = &args.marker_file {
        let mut f = File::open(marker_file)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        for line in s.lines() {
            if let Ok(marker) = line.trim().parse::<f32>() {
                markers.push(marker);
            }
        }
    }
    markers.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let num_markers = markers.len() as u32;
    let total_length = duration_samples;
    let nb_takes = take_sizes.len() as u32;

    file.write_u32::<LittleEndian>(session_timestamp)?;
    file.write_u32::<LittleEndian>(num_channels)?;
    file.write_u32::<LittleEndian>(sample_rate)?;
    file.write_u32::<LittleEndian>(session_timestamp)?;
    file.write_u32::<LittleEndian>(nb_takes)?;
    file.write_u32::<LittleEndian>(num_markers)?;
    file.write_u32::<LittleEndian>(total_length)?;

    for &size in take_sizes {
        file.write_u32::<LittleEndian>(size)?;
    }
    let zero_buf = vec![0u8; 4 * (256 - take_sizes.len())];
    file.write_all(&zero_buf)?;

    for marker in &markers {
        file.write_u32::<LittleEndian>((*marker * sample_rate as f32) as u32)?;
    }
    let zero_buf = vec![0u8; 4 * (125 - markers.len())];
    file.write_all(&zero_buf)?;

    let session_name_bytes = args.session_name.as_deref().unwrap_or("").as_bytes();
    let mut session_name_padded = vec![0u8; 16];
    let copy_len = std::cmp::min(session_name_bytes.len(), 16);
    session_name_padded[..copy_len].copy_from_slice(&session_name_bytes[..copy_len]);
    file.write_all(&session_name_padded)?;

    // Fill out the rest of the 2k with zeros
    let header_size = 28 + (4 * 256) + (4 * 125) + 16;
    let zero_fill_size = 2048 - header_size;
    let zero_buf = vec![0u8; zero_fill_size];
    file.write_all(&zero_buf)?;

    Ok(())
}

/// Validates that a WAV file meets the requirements for processing.
///
/// The file must be a mono, 24-bit PCM WAV file with a sample rate of 44100 or 48000 Hz.
/// It must also have the same sample rate and duration as the first file in the sequence.
fn validate_wav_file(
    path: &Path,
    spec: WavSpec,
    duration: u32,
    first_spec: &WavSpec,
    first_duration: u32,
) -> Result<()> {
    if spec.channels != 1 {
        return Err(anyhow!("File {} is not a mono WAV file.", path.display()));
    }
    if spec.bits_per_sample != 24 {
        return Err(anyhow!("File {} is not a 24-bit WAV file.", path.display()));
    }
    if spec.sample_format != hound::SampleFormat::Int {
        return Err(anyhow!("File {} is not a PCM WAV file.", path.display()));
    }
    if spec.sample_rate != 44100 && spec.sample_rate != 48000 {
        return Err(anyhow!(
            "File {} has an unsupported sample rate: {}. Only 44100 or 48000 are supported.",
            path.display(),
            spec.sample_rate
        ));
    }
    if spec.sample_rate != first_spec.sample_rate {
        return Err(anyhow!(
            "File {} has a different sample rate than the first file.",
            path.display()
        ));
    }
    if duration != first_duration {
        return Err(anyhow!(
            "File {} has a different duration than the first file.",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{WavSpec, WavWriter};
    use tempfile::tempdir;

    fn create_test_wav(dir: &Path, name: &str, spec: WavSpec, duration_ms: u32) {
        let path = dir.join(name);
        let mut writer = WavWriter::create(&path, spec).unwrap();
        let num_samples = (spec.sample_rate as u32 * duration_ms) / 1000;
        for _ in 0..num_samples {
            // Write a 24-bit sample
            writer.write_sample(0i32).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn test_valid_wav_files() {
        let dir = tempdir().unwrap();
        let spec = WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        };
        create_test_wav(dir.path(), "ch_1.wav", spec, 100);

        let args = Args {
            session_dir: dir.path().to_path_buf(),
            session_name: None,
            marker_file: None,
            markers: vec![],
            uppercase: false,
            silent: true,
        };
        assert!(run(&args).is_ok());
    }

    #[test]
    fn test_invalid_bit_depth() {
        let dir = tempdir().unwrap();
        let spec = WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        create_test_wav(dir.path(), "ch_1.wav", spec, 100);

        let args = Args {
            session_dir: dir.path().to_path_buf(),
            session_name: None,
            marker_file: None,
            markers: vec![],
            uppercase: false,
            silent: true,
        };
        let result = run(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("is not a 24-bit WAV file")
        );
    }

    #[test]
    fn test_se_log_bin_creation() {
        let dir = tempdir().unwrap();
        let spec = WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        };
        create_test_wav(dir.path(), "ch_1.wav", spec, 1000); // 1 second duration

        let args = Args {
            session_dir: dir.path().to_path_buf(),
            session_name: Some("Test Session".to_string()),
            marker_file: None,
            markers: vec![0.5],
            uppercase: false,
            silent: true,
        };
        run(&args).unwrap();

        // Find the created session directory
        let session_dir = fs::read_dir(dir.path())
            .unwrap()
            .find(|entry| entry.as_ref().unwrap().path().is_dir())
            .expect("No session directory found in temp dir")
            .unwrap()
            .path();

        let log_path = session_dir.join("SE_LOG.BIN");
        assert!(log_path.exists(), "SE_LOG.BIN was not created");

        let mut file = File::open(log_path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 2048, "SE_LOG.BIN is not 2048 bytes long");
    }

    #[test]
    fn test_wav_take_creation() {
        let dir = tempdir().unwrap();
        let spec = WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        };
        create_test_wav(dir.path(), "ch_1.wav", spec, 100);
        create_test_wav(dir.path(), "ch_2.wav", spec, 100);

        let args = Args {
            session_dir: dir.path().to_path_buf(),
            session_name: None,
            marker_file: None,
            markers: vec![],
            uppercase: false,
            silent: true,
        };
        run(&args).unwrap();

        let session_dir = fs::read_dir(dir.path())
            .unwrap()
            .find(|entry| entry.as_ref().unwrap().path().is_dir())
            .expect("No session directory found")
            .unwrap()
            .path();

        let wav_files: Vec<_> = fs::read_dir(&session_dir)
            .unwrap()
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                if path.extension().map_or(false, |ext| ext == "wav") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(wav_files.len(), 1);

        let reader = WavReader::open(&wav_files[0]).unwrap();
        let output_spec = reader.spec();
        assert_eq!(output_spec.channels, 2);
        assert_eq!(output_spec.sample_rate, 48000);
        assert_eq!(output_spec.bits_per_sample, 32);
    }
}
