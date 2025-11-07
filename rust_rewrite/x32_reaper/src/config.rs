//! # Configuration Module
//!
//! Defines the data structures for the application's configuration.

use serde::{Deserialize, Serialize};

/// Represents a range of values with a minimum and maximum.
#[derive(Serialize, Deserialize, Debug)]
pub struct Range {
    pub min: u8,
    pub max: u8,
}

/// Holds the application's configuration settings.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Enable verbose logging.
    pub verbose: bool,
    /// Delay in milliseconds for channel bank operations.
    pub delay_bank: u64,
    /// Generic delay in milliseconds for OSC messages.
    pub delay_generic: u64,
    /// IP address of the X32/M32 console.
    pub x32_ip: String,
    /// IP address of the Reaper DAW.
    pub reaper_ip: String,
    /// Port to receive OSC messages from Reaper.
    pub reaper_receive_port: u16,
    /// Port to send OSC messages to Reaper.
    pub reaper_send_port: u16,
    /// Enable transport control.
    pub transport_on: bool,
    /// Enable channel banking.
    pub channel_bank_on: bool,
    /// Enable the marker button.
    pub marker_button_on: bool,
    /// Color for user bank C.
    pub bank_c_color: u8,
    /// Enable EQ control UI.
    pub eq_control_on: bool,
    /// Enable master fader control.
    pub master_on: bool,
    /// Mapping for channel tracks.
    pub channel_map: Range,
    /// Mapping for auxiliary input tracks.
    pub aux_map: Range,
    /// Mapping for FX return tracks.
    pub fx_return_map: Range,
    /// Mapping for bus tracks.
    pub bus_map: Range,
    /// Mapping for DCA tracks.
    pub dca_map: Range,
    /// Offset for track send numbers.
    pub track_send_offset: u8,
    /// Mapping for Reaper DCA groups.
    pub reaper_dca_map: [Range; 8],
    /// Button for channel bank up.
    pub bank_up_button: u8,
    /// Button for channel bank down.
    pub bank_down_button: u8,
    /// Button for setting a marker.
    pub marker_button: u8,
    /// Size of the channel banks.
    pub bank_size: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: false,
            delay_bank: 5,
            delay_generic: 2,
            x32_ip: "192.168.1.64".to_string(),
            reaper_ip: "127.0.0.1".to_string(),
            reaper_receive_port: 10024,
            reaper_send_port: 10025,
            transport_on: false,
            channel_bank_on: false,
            marker_button_on: false,
            bank_c_color: 0,
            eq_control_on: false,
            master_on: true,
            channel_map: Range { min: 1, max: 32 },
            aux_map: Range { min: 33, max: 40 },
            fx_return_map: Range { min: 41, max: 48 },
            bus_map: Range { min: 49, max: 64 },
            dca_map: Range { min: 65, max: 72 },
            track_send_offset: 0,
            reaper_dca_map: [
                Range { min: 0, max: 0 },
                Range { min: 0, max: 0 },
                Range { min: 0, max: 0 },
                Range { min: 0, max: 0 },
                Range { min: 0, max: 0 },
                Range { min: 0, max: 0 },
                Range { min: 0, max: 0 },
                Range { min: 0, max: 0 },
            ],
            bank_up_button: 9,
            bank_down_button: 10,
            marker_button: 8,
            bank_size: 32,
        }
    }
}
