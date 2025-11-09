//! Provides functions for generating OSC commands to control X32/M32 library presets.
//!
//! The X32 allows users to save and recall presets for channels, effects, and routing.
//! This module provides functions for interacting with these presets.
use osc_lib::OscArg;

// --- Address String Getters ---

/// Returns the OSC address for a channel preset's name.
pub fn channel_preset_name(preset_num: u8) -> String {
    format!("/libs/ch/{:03}/name", preset_num)
}

/// Returns the OSC address for a routing preset's name.
pub fn routing_preset_name(preset_num: u8) -> String {
    format!("/libs/routing/{:03}/name", preset_num)
}

/// Returns the OSC address for an fx preset's name.
pub fn fx_preset_name(preset_num: u8) -> String {
    format!("/libs/fx/{:03}/name", preset_num)
}

// --- OSC Message Setters ---

/// Creates an OSC message to set the name of a channel preset.
///
/// # Arguments
///
/// * `preset_num` - The preset number (1-100).
/// * `name` - The new name for the preset.
///
/// ```
/// use x32_lib::command::libs;
///
/// let (address, args) = libs::set_channel_preset_name(1, "Test");
/// assert_eq!(address, "/libs/ch/001/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_channel_preset_name(preset_num: u8, name: &str) -> (String, Vec<OscArg>) {
    (
        channel_preset_name(preset_num),
        vec![OscArg::String(name.to_string())],
    )
}

/// Creates an OSC message to set the name of a routing preset.
///
/// # Arguments
///
/// * `preset_num` - The preset number (1-100).
/// * `name` - The new name for the preset.
///
/// ```
/// use x32_lib::command::libs;
///
/// let (address, args) = libs::set_routing_preset_name(1, "Test");
/// assert_eq!(address, "/libs/routing/001/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_routing_preset_name(preset_num: u8, name: &str) -> (String, Vec<OscArg>) {
    (
        routing_preset_name(preset_num),
        vec![OscArg::String(name.to_string())],
    )
}

/// Creates an OSC message to set the name of a fx preset.
///
/// # Arguments
///
/// * `preset_num` - The preset number (1-100).
/// * `name` - The new name for the preset.
///
/// ```
/// use x32_lib::command::libs;
///
/// let (address, args) = libs::set_fx_preset_name(1, "Test");
/// assert_eq!(address, "/libs/fx/001/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_fx_preset_name(preset_num: u8, name: &str) -> (String, Vec<OscArg>) {
    (
        fx_preset_name(preset_num),
        vec![OscArg::String(name.to_string())],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_channel_preset_name() {
        let (address, args) = set_channel_preset_name(1, "Test");
        assert_eq!(address, "/libs/ch/001/name");
        assert_eq!(args, vec![OscArg::String("Test".to_string())]);
    }

    #[test]
    fn test_set_routing_preset_name() {
        let (address, args) = set_routing_preset_name(1, "Test");
        assert_eq!(address, "/libs/routing/001/name");
        assert_eq!(args, vec![OscArg::String("Test".to_string())]);
    }

    #[test]
    fn test_set_fx_preset_name() {
        let (address, args) = set_fx_preset_name(1, "Test");
        assert_eq!(address, "/libs/fx/001/name");
        assert_eq!(args, vec![OscArg::String("Test".to_string())]);
    }
}
