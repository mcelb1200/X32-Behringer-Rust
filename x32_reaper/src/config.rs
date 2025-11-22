use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
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
        let mut reader = BufReader::new(file);
        let mut lines = reader.lines();

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
        let parts1: Vec<&str> = line1.split_whitespace().collect();
        if parts1.len() < 7 {
            anyhow::bail!("Invalid format in line 1");
        }
        let verbose = parts1[2].parse::<i32>()? != 0;
        let delay_bank = parts1[3].parse::<u64>()?;
        let delay_generic = parts1[4].parse::<u64>()?;
        let xx_send_mask = parts1[5].parse::<i32>()?;
        let xr_send_mask = parts1[6].parse::<i32>()?;

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
        let parts6: Vec<&str> = line6.split_whitespace().collect();
        if parts6.len() < 6 {
            anyhow::bail!("Invalid format in line 6");
        }
        let transport_on = parts6[0].parse::<i32>()? != 0;
        let ch_bank_on = parts6[1].parse::<i32>()? != 0;
        let marker_btn_on = parts6[2].parse::<i32>()? != 0;
        let bank_c_color = parts6[3].parse::<i32>()?;
        let eq_ctrl_on = parts6[4].parse::<i32>()? != 0;
        let master_on = parts6[5].parse::<i32>()? != 0;

        // Line 7: Ranges
        let line7 = next_line()?;
        let parts7: Vec<&str> = line7.split_whitespace().collect();
        if parts7.len() < 11 {
            anyhow::bail!("Invalid format in line 7");
        }
        let trk_min = parts7[0].parse::<i32>()?;
        let trk_max = parts7[1].parse::<i32>()?;
        let aux_min = parts7[2].parse::<i32>()?;
        let aux_max = parts7[3].parse::<i32>()?;
        let fxr_min = parts7[4].parse::<i32>()?;
        let fxr_max = parts7[5].parse::<i32>()?;
        let bus_min = parts7[6].parse::<i32>()?;
        let bus_max = parts7[7].parse::<i32>()?;
        let dca_min = parts7[8].parse::<i32>()?;
        let dca_max = parts7[9].parse::<i32>()?;
        let track_send_offset = parts7[10].parse::<i32>()?;

        // Next 8 lines: RDCA ranges
        let mut rdca = Vec::new();
        for _ in 0..8 {
            let line = next_line()?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                anyhow::bail!("Invalid format in RDCA line");
            }
            rdca.push((parts[0].parse::<i32>()?, parts[1].parse::<i32>()?));
        }

        // Last line: Bank controls
        let line_last = next_line()?;
        let parts_last: Vec<&str> = line_last.split_whitespace().collect();
        if parts_last.len() < 5 {
            anyhow::bail!("Invalid format in last line");
        }
        let mut bank_up = parts_last[0].parse::<i32>()?;
        let mut bank_dn = parts_last[1].parse::<i32>()?;
        let marker_btn = parts_last[2].parse::<i32>()?;
        let ch_bank_offset = parts_last[3].parse::<i32>()?;
        let bank_size = parts_last[4].parse::<i32>()?;

        // If transport_on is OFF, check if there are extra bank buttons in the file?
        // The C code does:
        // fscanf(..., &XMbankup, &XMbankdn, &XMkerbtn, &Xchbkof, &bkchsz);
        // if (!Xtransport_on) { fscanf(..., &XMbankup, &XMbankdn); }

        // It seems if transport_on is FALSE, there might be an EXTRA line with bank up/down?
        // Wait, let's check C code again.
        // fscanf(res_file, "%d %d %d %d %d\n", &XMbankup, &XMbankdn, &XMkerbtn, &Xchbkof, &bkchsz);
        // if (Xchbank_on) { ... if (!Xtransport_on) { fscanf(res_file, "%d %d\n", &XMbankup, &XMbankdn); } }

        if ch_bank_on && !transport_on {
             // Try to read one more line
             if let Some(Ok(line)) = lines.next() {
                 let parts: Vec<&str> = line.split_whitespace().collect();
                 if parts.len() >= 2 {
                     bank_up = parts[0].parse::<i32>()?;
                     bank_dn = parts[1].parse::<i32>()?;
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
