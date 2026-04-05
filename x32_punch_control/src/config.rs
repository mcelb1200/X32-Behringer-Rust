use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub xip_str: String,
    pub xscene_str: String,
    pub xusemidi: bool,
    pub xmidiinport: i32,
    pub xmidioutport: i32,
    pub xmidi_play_str: String,
    pub xmidi_pchin_str: String,
    pub xmidi_pchout_str: String,
    pub xmidi_stop_str: String,
    pub xmidi_pause_str: String,
    pub xmidi_resume_str: String,
    pub xmidi_reset_str: String,
    pub xmidi_bot_str: String,
    pub xmidi_eot_str: String,
    pub xmidi_record_str: String,
    pub xusebank: bool,
    pub xbank: char,
    pub xbbcolor: i32,
    pub xkeepontop: bool,
    pub xrecordplay: bool,
    pub xmtcon: bool,
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
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = match File::open(path.as_ref()) {
            Ok(f) => f,
            Err(_) => return Ok(Config::default()), // If no config, return default
        };

        if file.metadata()?.len() > 1024 * 1024 {
            anyhow::bail!("Config file too large to load (max 1MB)");
        }

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
