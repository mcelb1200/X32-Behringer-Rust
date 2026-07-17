use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub verbose: bool,
    pub delay_bank: u64,
    pub delay_generic: u64,
    pub xx_send_mask: i32,
    pub xr_send_mask: i32,
    pub x32_ip: String,
    pub reaper_ip: String,
    pub reaper_send_port: u16,
    pub reaper_recv_port: u16,
    pub transport_on: bool,
    pub ch_bank_on: bool,
    pub marker_btn_on: bool,
    pub bank_c_color: i32,
    #[allow(dead_code)]
    pub eq_ctrl_on: bool,
    pub master_on: bool,
    pub trk_min: i32,
    pub trk_max: i32,
    pub aux_min: i32,
    pub aux_max: i32,
    pub fxr_min: i32,
    pub fxr_max: i32,
    pub bus_min: i32,
    pub bus_max: i32,
    pub dca_min: i32,
    pub dca_max: i32,
    pub track_send_offset: i32,
    pub rdca: Vec<(i32, i32)>,
    pub bank_up: i32,
    pub bank_dn: i32,
    pub marker_btn: i32,
    pub ch_bank_offset: i32,
    pub bank_size: i32,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path).context("Failed to open config file")?;

        if file.metadata()?.len() > 1024 * 1024 {
            anyhow::bail!("Config file too large to load (max 1MB)");
        }

        let mut content = String::new();
        file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
        if content.len() > 1024 * 1024 {
            anyhow::bail!("Config file too large to load (max 1MB)");
        }
        let cursor = std::io::Cursor::new(content);
        let mut lines = cursor.lines();

        // helper to get next line and parse
        let mut next_line = || -> Result<String> {
            lines
                .next()
                .context("Unexpected end of file")??
                .trim()
                .to_string()
                .parse()
                .context("Failed to parse string")
        };

        // Line 1: width height verbose delayb delayg xxsend xrsend
        let line1 = next_line()?;
        let mut parts1 = line1.split_whitespace();
        let _ = parts1.next(); // width
        let _ = parts1.next(); // height
        let verbose = parts1
            .next()
            .context("Missing verbose in line 1")?
            .parse::<i32>()?
            != 0;
        let delay_bank = parts1
            .next()
            .context("Missing delay_bank in line 1")?
            .parse::<u64>()?;
        let delay_generic = parts1
            .next()
            .context("Missing delay_generic in line 1")?
            .parse::<u64>()?;
        let xx_send_mask = parts1
            .next()
            .context("Missing xx_send_mask in line 1")?
            .parse::<i32>()?;
        let xr_send_mask = parts1
            .next()
            .context("Missing xr_send_mask in line 1")?
            .parse::<i32>()?;

        // Line 2: X32 IP
        let x32_ip = next_line()?;

        // Line 3: Reaper Host IP
        let reaper_ip = next_line()?;

        // Line 4: Reaper Send Port
        let reaper_send_port = next_line()?.parse::<u16>()?;

        // Line 5: Reaper Recv Port
        let reaper_recv_port = next_line()?.parse::<u16>()?;

        // Line 6: flags
        let line6 = next_line()?;
        let mut parts6 = line6.split_whitespace();
        let transport_on = parts6
            .next()
            .context("Missing transport_on in line 6")?
            .parse::<i32>()?
            != 0;
        let ch_bank_on = parts6
            .next()
            .context("Missing ch_bank_on in line 6")?
            .parse::<i32>()?
            != 0;
        let marker_btn_on = parts6
            .next()
            .context("Missing marker_btn_on in line 6")?
            .parse::<i32>()?
            != 0;
        let bank_c_color = parts6
            .next()
            .context("Missing bank_c_color in line 6")?
            .parse::<i32>()?;
        let eq_ctrl_on = parts6
            .next()
            .context("Missing eq_ctrl_on in line 6")?
            .parse::<i32>()?
            != 0;
        let master_on = parts6
            .next()
            .context("Missing master_on in line 6")?
            .parse::<i32>()?
            != 0;

        // Line 7: Ranges
        let line7 = next_line()?;
        let mut parts7 = line7.split_whitespace();
        let trk_min = parts7
            .next()
            .context("Missing trk_min in line 7")?
            .parse::<i32>()?;
        let trk_max = parts7
            .next()
            .context("Missing trk_max in line 7")?
            .parse::<i32>()?;
        let aux_min = parts7
            .next()
            .context("Missing aux_min in line 7")?
            .parse::<i32>()?;
        let aux_max = parts7
            .next()
            .context("Missing aux_max in line 7")?
            .parse::<i32>()?;
        let fxr_min = parts7
            .next()
            .context("Missing fxr_min in line 7")?
            .parse::<i32>()?;
        let fxr_max = parts7
            .next()
            .context("Missing fxr_max in line 7")?
            .parse::<i32>()?;
        let bus_min = parts7
            .next()
            .context("Missing bus_min in line 7")?
            .parse::<i32>()?;
        let bus_max = parts7
            .next()
            .context("Missing bus_max in line 7")?
            .parse::<i32>()?;
        let dca_min = parts7
            .next()
            .context("Missing dca_min in line 7")?
            .parse::<i32>()?;
        let dca_max = parts7
            .next()
            .context("Missing dca_max in line 7")?
            .parse::<i32>()?;
        let track_send_offset = parts7
            .next()
            .context("Missing track_send_offset in line 7")?
            .parse::<i32>()?;

        // Next 8 lines: RDCA ranges
        let mut rdca = Vec::new();
        for _ in 0..8 {
            let line = next_line()?;
            let mut parts = line.split_whitespace();
            let p0 = parts
                .next()
                .context("Missing RDCA param 1")?
                .parse::<i32>()?;
            let p1 = parts
                .next()
                .context("Missing RDCA param 2")?
                .parse::<i32>()?;
            rdca.push((p0, p1));
        }

        // Last line: Bank controls
        let line_last = next_line()?;
        let mut parts_last = line_last.split_whitespace();
        let mut bank_up = parts_last
            .next()
            .context("Missing bank_up")?
            .parse::<i32>()?;
        let mut bank_dn = parts_last
            .next()
            .context("Missing bank_dn")?
            .parse::<i32>()?;
        let marker_btn = parts_last
            .next()
            .context("Missing marker_btn")?
            .parse::<i32>()?;
        let ch_bank_offset = parts_last
            .next()
            .context("Missing ch_bank_offset")?
            .parse::<i32>()?;
        let bank_size = parts_last
            .next()
            .context("Missing bank_size")?
            .parse::<i32>()?;

        // If transport_on is OFF, check if there are extra bank buttons in the file?
        if ch_bank_on && !transport_on {
            // Try to read one more line
            if let Some(Ok(line)) = lines.next() {
                let mut parts = line.split_whitespace();
                if let (Some(p0), Some(p1)) = (parts.next(), parts.next()) {
                    bank_up = p0.parse::<i32>()?;
                    bank_dn = p1.parse::<i32>()?;
                }
            }
        }

        Ok(Config {
            verbose,
            delay_bank,
            delay_generic,
            xx_send_mask,
            xr_send_mask,
            x32_ip,
            reaper_ip,
            reaper_send_port,
            reaper_recv_port,
            transport_on,
            ch_bank_on,
            marker_btn_on,
            bank_c_color,
            eq_ctrl_on,
            master_on,
            trk_min,
            trk_max,
            aux_min,
            aux_max,
            fxr_min,
            fxr_max,
            bus_min,
            bus_max,
            dca_min,
            dca_max,
            track_send_offset,
            rdca,
            bank_up,
            bank_dn,
            marker_btn,
            ch_bank_offset,
            bank_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_load_file_not_found() {
        let result = Config::load("non_existent_file.txt");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to open config file"
        );
    }

    #[test]
    fn test_config_load_too_large() {
        let mut file = NamedTempFile::new().unwrap();
        // Create a file larger than 1MB
        let buffer = vec![b'x'; 1024 * 1024 + 10];
        file.write_all(&buffer).unwrap();

        let result = Config::load(file.path());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Config file too large to load (max 1MB)"
        );
    }

    #[test]
    fn test_config_load_invalid_format() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid format").unwrap();

        let result = Config::load(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_valid() {
        let mut file = NamedTempFile::new().unwrap();
        // A valid config
        let valid_config = "
800 600 1 10 20 1 2
192.168.1.100
192.168.1.101
8000
8001
1 1 1 5 1 1
1 32 1 16 1 4 1 16 1 8 0
1 1
1 1
1 1
1 1
1 1
1 1
1 1
1 1
1 2 3 4 5
";
        write!(file, "{}", valid_config.trim()).unwrap();

        let result = Config::load(file.path());
        assert!(
            result.is_ok(),
            "Expected valid config to parse, got {:?}",
            result
        );
        let config = result.unwrap();
        assert_eq!(config.verbose, true);
        assert_eq!(config.x32_ip, "192.168.1.100");
    }
}
