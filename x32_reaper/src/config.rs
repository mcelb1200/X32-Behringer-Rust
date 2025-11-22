use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub x32_ip: String,
    pub reaper_ip: String,
    pub reaper_send_port: u16, // We send to this port (Reaper listens on this)
    pub reaper_recv_port: u16, // We receive from this port (Reaper sends to this)

    pub verbose: bool,
    pub delay_bank_ms: u64,
    pub delay_generic_ms: u64,

    // Masks for allowed commands (default to all allowed if not specified)
    pub x32_send_mask: Option<i32>,
    pub reaper_send_mask: Option<i32>,

    pub flags: Flags,
    pub map: TrackMap,
    pub dca_map: Vec<Range>, // Array of 8 ranges
    pub bank: BankConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Flags {
    pub transport_on: bool,
    pub channel_bank_on: bool,
    pub marker_button_on: bool,
    pub bank_c_color: i32,
    pub eq_control_on: bool,
    pub master_on: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackMap {
    pub track_min: i32,
    pub track_max: i32,
    pub aux_min: i32,
    pub aux_max: i32,
    pub fxr_min: i32,
    pub fxr_max: i32,
    pub bus_min: i32,
    pub bus_max: i32,
    pub dca_min: i32,
    pub dca_max: i32,
    pub track_send_offset: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Range {
    pub min: i32,
    pub max: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BankConfig {
    pub bank_up_button: i32,
    pub bank_down_button: i32,
    pub marker_button: i32,
    pub initial_bank_offset: i32,
    pub bank_size: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            x32_ip: "192.168.0.64".to_string(),
            reaper_ip: "127.0.0.1".to_string(),
            reaper_send_port: 8000,
            reaper_recv_port: 9000,
            verbose: false,
            delay_bank_ms: 20,
            delay_generic_ms: 20,
            x32_send_mask: Some(-1),
            reaper_send_mask: Some(-1),
            flags: Flags {
                transport_on: false,
                channel_bank_on: true,
                marker_button_on: false,
                bank_c_color: 0,
                eq_control_on: false,
                master_on: true,
            },
            map: TrackMap {
                track_min: 1,
                track_max: 32,
                aux_min: 33,
                aux_max: 40,
                fxr_min: 41,
                fxr_max: 48,
                bus_min: 49,
                bus_max: 64,
                dca_min: 65,
                dca_max: 72,
                track_send_offset: 0,
            },
            dca_map: vec![Range { min: 0, max: 0 }; 8],
            bank: BankConfig {
                bank_up_button: 9,
                bank_down_button: 10,
                marker_button: 0,
                initial_bank_offset: 0,
                bank_size: 32,
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config = serde_json::from_str(&content)?;
    Ok(config)
}
