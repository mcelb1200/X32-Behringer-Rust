#![allow(dead_code)]
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

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

    // Security: Prevent OOM from maliciously large or corrupted configuration files
    if file.metadata()?.len() > 1024 * 1024 {
        return Err(anyhow!("File too large"));
    }

    let reader = BufReader::new(file.take(1024 * 1024));
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

            // ⚡ Bolt: Parse midi_status hex manually instead of using u8::from_str_radix
            // This avoids overhead of slice creation, generic parsing, and utf8 checks,
            // which improves configuration loading speed for a better UX.
            let status_str = parts[0];
            if status_str.len() > 2 || status_str.is_empty() {
                return Err(anyhow!("Invalid midi status hex length"));
            }
            let mut midi_status = 0u8;
            for &b in status_str.as_bytes() {
                let v = match b {
                    b'0'..=b'9' => b - b'0',
                    b'a'..=b'f' => b - b'a' + 10,
                    b'A'..=b'F' => b - b'A' + 10,
                    _ => return Err(anyhow!("Failed to parse midi status hex: invalid char '{}'", b as char)),
                };
                midi_status = (midi_status << 4) | v;
            }

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

#[derive(Debug, Clone)]
pub struct Config {
    pub midi_in_port: i32,
    pub ip_str: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            midi_in_port: 0,
            ip_str: "192.168.0.64".to_string(),
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let file = File::open(path)?;

        // Security: Prevent OOM from maliciously large or corrupted configuration files
        if file.metadata()?.len() > 1024 * 1024 {
            return Err(anyhow!("File too large"));
        }

        let reader = BufReader::new(file.take(1024 * 1024));

        let mut midi_in_port = 0;
        let mut ip_str = "192.168.0.64".to_string();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Typical line parsing, but keeping it robust
            // Try to see if it's the port number
            if trimmed.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(port) = trimmed.parse::<i32>() {
                    midi_in_port = port;
                }
            } else {
                ip_str = trimmed.to_string();
            }
        }

        Ok(Config {
            midi_in_port,
            ip_str,
        })
    }
}
