use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub ip_str: String,
    pub midi_in_port: i32,
    pub midi_out_port: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip_str: "192.168.0.64".to_string(),
            midi_in_port: 0,
            midi_out_port: 0,
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = match File::open(path.as_ref()) {
            Ok(f) => f,
            Err(_) => return Ok(Config::default()), // If no config, return default
        };
        let reader = BufReader::new(file);
        let mut config = Config::default();

        for line_result in reader.lines() {
            let line = line_result?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Xip_str" => config.ip_str = value.to_string(),
                    "Xmidiinport" => config.midi_in_port = value.parse().unwrap_or(0),
                    "Xmidioutport" => config.midi_out_port = value.parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

        Ok(config)
    }
}

/// Represents a mapping between a MIDI command and an OSC command.
#[derive(Debug, Clone, PartialEq)]
pub struct MidiOscRule {
    /// Combined command representation ((m_d1 << 8) | (m_mc | ((m_ch - 1) & 0xF)))
    pub command_id: u32,
    /// Message code (e.g., 0xB0 for Control Change)
    pub mc: i32,
    /// Channel (1-16)
    pub ch: i32,
    /// Data byte 1
    pub d1: i32,
    /// Data byte 2
    pub d2: i32,
    /// OSC template string
    pub osc_template: String,
}

impl MidiOscRule {
    /// Parses a command rule from a string line.
    /// Expected format: `[hex_mc] [dec_ch] [dec_d1] [dec_d2] | [osc_template]`
    /// e.g. `B0 1 12 0 | /ch/01/mix/fader ,f [$1 127.0 /]`
    pub fn parse_line(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() != 2 {
            return None;
        }

        let midi_part = parts[0].trim();
        let osc_template = parts[1].trim().to_string();

        let midi_tokens: Vec<&str> = midi_part.split_whitespace().collect();
        if midi_tokens.len() < 4 {
            return None;
        }

        let mc = i32::from_str_radix(midi_tokens[0], 16).ok()?;
        let ch = midi_tokens[1].parse::<i32>().ok()?;
        let d1 = midi_tokens[2].parse::<i32>().ok()?;
        let d2 = midi_tokens[3].parse::<i32>().ok()?;

        // The combined ID replicates the C logic: (Md1 << 8) | (Mmc | ((Mch - 1) & 0xF))
        let command_id = ((d1 << 8) | (mc | ((ch - 1) & 0xF))) as u32;

        Some(Self {
            command_id,
            mc,
            ch,
            d1,
            d2,
            osc_template,
        })
    }

    /// Loads a list of rules from an `.m2o` file.
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Vec<Self>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut rules = Vec::new();

        for line in reader.lines() {
            if let Some(rule) = Self::parse_line(&line?) {
                rules.push(rule);
            }
        }

        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_load() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Xip_str=10.0.0.1").unwrap();
        writeln!(file, "Xmidiinport=2").unwrap();
        writeln!(file, "Xmidioutport=3").unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(config.ip_str, "10.0.0.1");
        assert_eq!(config.midi_in_port, 2);
        assert_eq!(config.midi_out_port, 3);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.ip_str, "192.168.0.64");
        assert_eq!(config.midi_in_port, 0);
        assert_eq!(config.midi_out_port, 0);
    }

    #[test]
    fn test_parse_rule() {
        let line = "B0 1 12 0 | /ch/01/mix/fader ,f [$1 127.0 /]";
        let rule = MidiOscRule::parse_line(line).unwrap();

        assert_eq!(rule.mc, 0xB0); // 176
        assert_eq!(rule.ch, 1);
        assert_eq!(rule.d1, 12);
        assert_eq!(rule.d2, 0);
        assert_eq!(rule.osc_template, "/ch/01/mix/fader ,f [$1 127.0 /]");

        // (12 << 8) | (0xB0 | ((1 - 1) & 0xF)) = 3072 | (176 | 0) = 3248
        assert_eq!(rule.command_id, 3248);
    }

    #[test]
    fn test_parse_rule_invalid() {
        assert!(MidiOscRule::parse_line("# comment line").is_none());
        assert!(MidiOscRule::parse_line("").is_none());
        assert!(MidiOscRule::parse_line("B0 1 12").is_none()); // Missing parts
        assert!(MidiOscRule::parse_line("ZZ 1 12 0 | OSC").is_none()); // Invalid hex
    }

    #[test]
    fn test_load_m2o_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Test file").unwrap();
        writeln!(file, "B0 1 12 0 | /ch/01/mix/fader ,f [$2 127.0 /]").unwrap();
        writeln!(file, "90 2 64 127 | /ch/01/mix/on ,i [$2 0 >]").unwrap();

        let rules = MidiOscRule::load_file(file.path()).unwrap();
        assert_eq!(rules.len(), 2);

        assert_eq!(rules[0].mc, 0xB0);
        assert_eq!(rules[1].mc, 0x90);
    }
}
