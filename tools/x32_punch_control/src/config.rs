//! Configuration loading for `x32_punch_control`.

use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, Read};
use std::path::Path;

/// Configuration loaded from `.X32PunchControl.ini` or defaults.
#[derive(Debug, Clone)]
pub struct Config {
    /// Target X32 IP address.
    pub xip_str: String,
    /// Target scene string.
    pub xscene_str: String,
    /// Whether MIDI control is enabled.
    pub xusemidi: bool,
    /// MIDI input port index.
    pub xmidiinport: i32,
    /// MIDI output port index.
    pub xmidioutport: i32,
    /// MIDI string for play.
    pub xmidi_play_str: String,
    /// MIDI string for punch-in.
    pub xmidi_pchin_str: String,
    /// MIDI string for punch-out.
    pub xmidi_pchout_str: String,
    /// MIDI string for stop.
    pub xmidi_stop_str: String,
    /// MIDI string for pause.
    pub xmidi_pause_str: String,
    /// MIDI string for resume.
    pub xmidi_resume_str: String,
    /// MIDI string for reset.
    pub xmidi_reset_str: String,
    /// MIDI string for beginning of tape.
    pub xmidi_bot_str: String,
    /// MIDI string for end of tape.
    pub xmidi_eot_str: String,
    /// MIDI string for record.
    pub xmidi_record_str: String,
    /// Whether User Bank buttons are used.
    pub xusebank: bool,
    /// User Bank identifier (`A`, `B`, or `C`).
    pub xbank: char,
    /// Button backlight color index.
    pub xbbcolor: i32,
    /// Whether the window stays on top.
    pub xkeepontop: bool,
    /// Whether record-play mode is enabled.
    pub xrecordplay: bool,
    /// Whether MIDI Time Code (MTC) is enabled.
    pub xmtcon: bool,
    /// Catch-up delay in milliseconds.
    pub xcatchdelay: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            xip_str: "192.168.0.64".to_string(),
            xscene_str: "1".to_string(),
            xusemidi: false,
            xmidiinport: 0,
            xmidioutport: 0,
            xmidi_play_str: "".to_string(),
            xmidi_pchin_str: "".to_string(),
            xmidi_pchout_str: "".to_string(),
            xmidi_stop_str: "".to_string(),
            xmidi_pause_str: "".to_string(),
            xmidi_resume_str: "".to_string(),
            xmidi_reset_str: "".to_string(),
            xmidi_bot_str: "".to_string(),
            xmidi_eot_str: "".to_string(),
            xmidi_record_str: "".to_string(),
            xusebank: false,
            xbank: 'A',
            xbbcolor: 0,
            xkeepontop: false,
            xrecordplay: false,
            xmtcon: false,
            xcatchdelay: 10,
        }
    }
}

impl Config {
    /// Loads configuration from `path`, returning defaults if the file cannot be opened.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = match File::open(path.as_ref()) {
            Ok(f) => f,
            Err(_) => return Ok(Config::default()), // If no config, return default
        };

        if file.metadata()?.len() > 1024 * 1024 {
            anyhow::bail!("Config file too large to load (max 1MB)");
        }

        let mut content = String::new();
        file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
        if content.len() > 1024 * 1024 {
            anyhow::bail!("Config file too large to load (max 1MB)");
        }
        let reader = std::io::Cursor::new(content);
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
                    "Xip_str" => config.xip_str = value.to_string(),
                    "Xscene_str" => config.xscene_str = value.to_string(),
                    "Xusemidi" => {
                        config.xusemidi = value == "1"
                            || value.eq_ignore_ascii_case("true")
                            || value.eq_ignore_ascii_case("y")
                    }
                    "Xmidiinport" => config.xmidiinport = value.parse().unwrap_or(0),
                    "Xmidioutport" => config.xmidioutport = value.parse().unwrap_or(0),
                    "Xmidi_play_str" => config.xmidi_play_str = value.to_string(),
                    "Xmidi_pchin_str" => config.xmidi_pchin_str = value.to_string(),
                    "Xmidi_pchout_str" => config.xmidi_pchout_str = value.to_string(),
                    "Xmidi_stop_str" => config.xmidi_stop_str = value.to_string(),
                    "Xmidi_pause_str" => config.xmidi_pause_str = value.to_string(),
                    "Xmidi_resume_str" => config.xmidi_resume_str = value.to_string(),
                    "Xmidi_reset_str" => config.xmidi_reset_str = value.to_string(),
                    "Xmidi_bot_str" => config.xmidi_bot_str = value.to_string(),
                    "Xmidi_eot_str" => config.xmidi_eot_str = value.to_string(),
                    "Xmidi_record_str" => config.xmidi_record_str = value.to_string(),
                    "Xusebank" => {
                        config.xusebank = value == "1"
                            || value.eq_ignore_ascii_case("true")
                            || value.eq_ignore_ascii_case("y")
                    }
                    "Xbank" => config.xbank = value.chars().next().unwrap_or('A'),
                    "Xbbcolor" => config.xbbcolor = value.parse().unwrap_or(0),
                    "Xkeepontop" => {
                        config.xkeepontop = value == "1"
                            || value.eq_ignore_ascii_case("true")
                            || value.eq_ignore_ascii_case("y")
                    }
                    "Xrecordplay" => {
                        config.xrecordplay = value == "1"
                            || value.eq_ignore_ascii_case("true")
                            || value.eq_ignore_ascii_case("y")
                    }
                    "XMTCon" => {
                        config.xmtcon = value == "1"
                            || value.eq_ignore_ascii_case("true")
                            || value.eq_ignore_ascii_case("y")
                    }
                    "Xcatchdelay" => config.xcatchdelay = value.parse().unwrap_or(10),
                    _ => {}
                }
            }
        }

        Ok(config)
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
        writeln!(file, "Xusebank=1").unwrap();
        writeln!(file, "Xbank=B").unwrap();
        writeln!(file, "Xcatchdelay=50").unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(config.xip_str, "10.0.0.1");
        assert!(config.xusebank);
        assert_eq!(config.xbank, 'B');
        assert_eq!(config.xcatchdelay, 50);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.xip_str, "192.168.0.64");
        assert!(!config.xusebank);
        assert_eq!(config.xbank, 'A');
        assert_eq!(config.xcatchdelay, 10);
    }
}
