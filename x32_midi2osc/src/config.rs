#![allow(dead_code)]
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, PartialEq)]
pub struct MidiOscCommand {
    pub midi_status: u8,
    pub midi_channel: u8,
    pub data1: i32,
    pub data2: i32,
    pub osc_command: String,
}

impl MidiOscCommand {
    /// Calculate the matching value for (dwParam1 & 0xFFFF)
    /// based on the logic in the original C code:
    /// XMCommand = (Md1 << 8) | (Mmc | ((Mch - 1) & 0xF))
    pub fn get_match_key(&self) -> u32 {
        ((self.data1 as u32) << 8)
            | ((self.midi_status as u32) | ((self.midi_channel as u32 - 1) & 0xF))
    }
}

pub fn parse_file(path: &str) -> Result<Vec<MidiOscCommand>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut commands = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Format is typically: Mmc Mch Md1 Md2 | /osc/command/string
        // Example: B0 1 7 127 | /ch/01/mix/fader ,f [$2 127 /]
        if let Some((midi_part, osc_part)) = line.split_once('|') {
            let osc_command = osc_part.trim().to_string();

            let parts: Vec<&str> = midi_part.split_whitespace().collect();
            if parts.len() < 4 {
                return Err(anyhow!("Invalid MIDI command format: {}", midi_part));
            }

            let midi_status = u8::from_str_radix(parts[0], 16)
                .map_err(|e| anyhow!("Failed to parse midi status hex: {}", e))?;

            let midi_channel: u8 = parts[1]
                .parse()
                .map_err(|e| anyhow!("Failed to parse midi channel: {}", e))?;

            let data1: i32 = parts[2]
                .parse()
                .map_err(|e| anyhow!("Failed to parse data1: {}", e))?;

            let data2: i32 = parts[3]
                .parse()
                .map_err(|e| anyhow!("Failed to parse data2: {}", e))?;

            commands.push(MidiOscCommand {
                midi_status,
                midi_channel,
                data1,
                data2,
                osc_command,
            });
        }
    }

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "B0 1 7 127 | /ch/01/mix/fader ,f [$2 127 /]").unwrap();
        writeln!(file, "90 1 60 127 | /ch/01/mix/on ,i [$2 0 >]").unwrap();

        let path = file.path().to_str().unwrap();
        let commands = parse_file(path).unwrap();

        assert_eq!(commands.len(), 2);

        let cmd1 = &commands[0];
        assert_eq!(cmd1.midi_status, 0xB0);
        assert_eq!(cmd1.midi_channel, 1);
        assert_eq!(cmd1.data1, 7);
        assert_eq!(cmd1.data2, 127);
        assert_eq!(cmd1.osc_command, "/ch/01/mix/fader ,f [$2 127 /]");
        assert_eq!(cmd1.get_match_key(), (7 << 8) | (0xB0 | 0));

        let cmd2 = &commands[1];
        assert_eq!(cmd2.midi_status, 0x90);
        assert_eq!(cmd2.midi_channel, 1);
        assert_eq!(cmd2.data1, 60);
        assert_eq!(cmd2.data2, 127);
        assert_eq!(cmd2.osc_command, "/ch/01/mix/on ,i [$2 0 >]");
    }
}
