//! # Config Module
//!
//! Handles parsing of the `.m2o` configuration file format.

use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};

/// Represents a single MIDI-to-OSC mapping.
#[derive(Debug, Clone)]
pub struct MidiToOsc {
    /// The MIDI command byte (e.g., 0x90 for Note On).
    pub midi_command: u32,
    /// The MIDI channel (1-16, or 0 for any).
    pub midi_channel: u8,
    /// The first MIDI data byte (0-127, or 0 for any).
    pub midi_data1: u8,
    /// The second MIDI data byte (0-127, or 0 for any).
    pub midi_data2: u8,
    /// The OSC command string to be sent.
    pub osc_command: String,
}

/// Parses a `.m2o` configuration file and returns a vector of `MidiToOsc` structs.
///
/// # Arguments
///
/// * `path` - The path to the configuration file.
pub fn parse_config<P: AsRef<Path>>(path: P) -> io::Result<Vec<MidiToOsc>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut commands = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((midi_part, osc_part)) = line.split_once('|') {
            let midi_params: Vec<&str> = midi_part.trim().split_whitespace().collect();
            if midi_params.len() != 4 {
                eprintln!("Skipping invalid MIDI format: {}", midi_part);
                continue;
            }

            let command = u32::from_str_radix(midi_params[0], 16).unwrap_or(0);
            let channel = midi_params[1].parse::<u8>().unwrap_or(0);
            let data1 = midi_params[2].parse::<u8>().unwrap_or(0);
            let data2 = midi_params[3].parse::<u8>().unwrap_or(0);

            commands.push(MidiToOsc {
                midi_command: command,
                midi_channel: channel,
                midi_data1: data1,
                midi_data2: data2,
                osc_command: osc_part.trim().to_string(),
            });
        }
    }

    Ok(commands)
}
