//! # Libraries Module
//!
//! Controls the preset libraries for channels, routing, and effects on the X32/M32.
//!
//! This module provides functions for managing presets within the console's internal libraries.

use osc_lib::OscArg;

/// Sets the name for a channel preset.
///
/// # Arguments
///
/// * `preset_id` - The ID of the preset (1-100).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::libs;
/// use osc_lib::OscArg;
///
/// let (address, args) = libs::set_channel_preset_name(1, "Vocal Preset");
/// assert_eq!(address, "/-libs/ch/001/name");
/// assert_eq!(args, vec![OscArg::String("Vocal Preset".to_string())]);
/// ```
pub fn set_channel_preset_name(preset_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/-libs/ch/{:03}/name", preset_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the name for a routing preset.
///
/// # Arguments
///
/// * `preset_id` - The ID of the preset (1-100).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::libs;
/// use osc_lib::OscArg;
///
/// let (address, args) = libs::set_routing_preset_name(1, "Main Outs");
/// assert_eq!(address, "/-libs/r/001/name");
/// assert_eq!(args, vec![OscArg::String("Main Outs".to_string())]);
/// ```
pub fn set_routing_preset_name(preset_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/-libs/r/{:03}/name", preset_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the name for an FX preset.
///
/// # Arguments
///
/// * `preset_id` - The ID of the preset (1-100).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::libs;
/// use osc_lib::OscArg;
///
/// let (address, args) = libs::set_fx_preset_name(1, "Vocal Reverb");
/// assert_eq!(address, "/-libs/fx/001/name");
/// assert_eq!(args, vec![OscArg::String("Vocal Reverb".to_string())]);
/// ```
pub fn set_fx_preset_name(preset_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/-libs/fx/{:03}/name", preset_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_channel_preset_name() {
        let (address, args) = set_channel_preset_name(1, "My Preset");
        assert_eq!(address, "/-libs/ch/001/name");
        assert_eq!(args, vec![OscArg::String("My Preset".to_string())]);
    }

    #[test]
    fn test_set_routing_preset_name() {
        let (address, args) = set_routing_preset_name(1, "My Preset");
        assert_eq!(address, "/-libs/r/001/name");
        assert_eq!(args, vec![OscArg::String("My Preset".to_string())]);
    }

    #[test]
    fn test_set_fx_preset_name() {
        let (address, args) = set_fx_preset_name(1, "My Preset");
        assert_eq!(address, "/-libs/fx/001/name");
        assert_eq!(args, vec![OscArg::String("My Preset".to_string())]);
    }
}
