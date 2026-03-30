//! `x32_xlive_wav` is a command-line utility for exploding X-Live! multi-channel
//! WAV files into standard (mono-channel) WAV files.
//!
//! It is a Rust rewrite of the original C program `X32Xlive_Wav.c` by Patrick-Gilles Maillot.

use anyhow::{Result, anyhow};
use clap::Parser;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::fs;
use std::path::PathBuf;

/// A utility to explode X-Live! multi-channel WAV files into standard mono WAV files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Source directory containing the XLive recording session
    #[arg(value_name = "SESSION_DIR")]
    pub session_dir: PathBuf,

    /// Destination directory where mono wave files will be saved
    #[arg(short = 'd', long, value_name = "DIR", default_value = "./")]
    pub dest_dir: PathBuf,

    /// Sets or replaces the Session name read from source
    #[arg(short = 'm', long, value_name = "NAME")]
    pub session_name: Option<String>,

    /// Number of channels to explode to mono wave files (1..32)
    #[arg(short = 'n', long, value_name = "CHANNELS")]
    pub num_channels: Option<u8>,

    /// Sample size (8, 16, 24, 32). Default is 24.
    #[arg(short = 'c', long, value_name = "SIZE", default_value_t = 24)]
    pub sample_size: u8,

    /// Optional scene file to parse channel names from
    #[arg(short = 's', long, value_name = "FILE")]
    pub scene_file: Option<PathBuf>,

    /// Channel number followed by respective wave file name (e.g., 3,new_name). Can be specified multiple times.
    #[arg(short = 'w', long, value_name = "CH,NAME", action = clap::ArgAction::Append)]
    pub channel_names: Vec<String>,

    /// Use uppercase (.WAV) rather than lowercase (.wav) in file names
    #[arg(short = 'u', long)]
    pub uppercase: bool,

    /// Prepends number in front of the channel name
    #[arg(short = 'p', long)]
    pub prepend_number: bool,

    /// Run in silent mode (only for non-error messages)
    #[arg(short = 'S', long)]
    pub silent: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    run(&args)
}

fn run(args: &Args) -> Result<()> {
    if let Some(num) = args.num_channels {
        if num == 0 || num > 32 {
            return Err(anyhow!("Number of channels must be between 1 and 32"));
        }
    } else {
        if !args.silent {
            println!("Number of channels not specified. Will attempt to read from session.");
        }
    }

    if ![8, 16, 24, 32].contains(&args.sample_size) {
        return Err(anyhow!("Sample size must be 8, 16, 24, or 32"));
    }

    if !args.silent {
        println!("Session Directory: {:?}", args.session_dir);
        println!("Destination Directory: {:?}", args.dest_dir);
        println!("Sample Size: {}", args.sample_size);
    }

    let chunk_files = get_session_chunks(&args.session_dir, args.uppercase)?;
    if chunk_files.is_empty() {
        return Err(anyhow!(
            "No valid XLive session chunk files found in the directory"
        ));
    }

    if !args.silent {
        println!("Found {} session chunk files.", chunk_files.len());
    }

    // Read first file to get info
    let reader = WavReader::open(&chunk_files[0])?;
    let spec = reader.spec();

    let session_channels = spec.channels as u8;
    let extract_channels = args.num_channels.unwrap_or(session_channels);

    if extract_channels > session_channels {
        return Err(anyhow!(
            "Requested to extract {} channels, but session only has {}.",
            extract_channels,
            session_channels
        ));
    }

    let dest_dir = &args.dest_dir;
    fs::create_dir_all(dest_dir)?;

    let mut writers = Vec::new();
    for i in 0..extract_channels {
        let name = if (i as usize) < args.channel_names.len() {
            let parts: Vec<&str> = args.channel_names[i as usize].split(',').collect();
            if parts.len() == 2 {
                parts[1].to_string()
            } else {
                format!("Xlive_Wav_{}", i + 1)
            }
        } else {
            format!("Xlive_Wav_{}", i + 1)
        };

        let mut final_name = name;
        if args.prepend_number {
            final_name = format!("{:02}_{}", i + 1, final_name);
        }

        let filename = if args.uppercase {
            format!("{}.WAV", final_name)
        } else {
            format!("{}.wav", final_name)
        };

        let file_path = dest_dir.join(filename);

        let out_spec = WavSpec {
            channels: 1,
            sample_rate: spec.sample_rate,
            bits_per_sample: args.sample_size as u16,
            sample_format: if args.sample_size == 32 {
                SampleFormat::Float
            } else {
                SampleFormat::Int
            },
        };

        let writer = WavWriter::create(file_path, out_spec)?;
        writers.push(writer);
    }

    if !args.silent {
        println!("Extracting {} channels...", extract_channels);
    }

    for chunk_file in chunk_files {
        let mut reader = WavReader::open(&chunk_file)?;
        let mut iter = reader.samples::<i32>();

        loop {
            let mut done = false;
            for i in 0..session_channels {
                if let Some(sample_res) = iter.next() {
                    let sample = sample_res?;
                    if i < extract_channels {
                        let writer = &mut writers[i as usize];

                        // Convert sample based on original and target bit depth
                        // X32 is always 24-bit in 32-bit container, or 32-bit PCM/Float
                        // hound returns i32 for 24-bit files.
                        match args.sample_size {
                            8 => {
                                // Downsample 24-bit to 8-bit
                                let val = (sample >> 16) as i8;
                                writer.write_sample(val)?;
                            }
                            16 => {
                                // Downsample 24-bit to 16-bit
                                let val = (sample >> 8) as i16;
                                writer.write_sample(val)?;
                            }
                            24 => {
                                // Keep 24-bit
                                writer.write_sample(sample)?;
                            }
                            32 => {
                                // Upconvert to 32-bit float
                                // sample is essentially 24 bit integer
                                let max_val = 8388608.0; // 2^23
                                let val = (sample as f32) / max_val;
                                writer.write_sample(val)?;
                            }
                            _ => unreachable!(),
                        }
                    }
                } else {
                    done = true;
                    break;
                }
            }
            if done {
                break;
            }
        }
    }

    for writer in writers {
        writer.finalize()?;
    }

    if !args.silent {
        println!("Done!");
    }

    Ok(())
}

fn get_session_chunks(session_dir: &std::path::Path, uppercase: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut i = 1;
    loop {
        let filename = if uppercase {
            format!("{:08x}.WAV", i)
        } else {
            format!("{:08x}.wav", i)
        };
        let file_path = session_dir.join(&filename);
        if file_path.exists() {
            files.push(file_path);
            i += 1;
        } else {
            break;
        }
    }
    Ok(files)
}
