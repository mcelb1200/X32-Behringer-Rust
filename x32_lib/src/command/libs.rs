//! This module provides the command definitions for the X32 libs.
use osc_lib::OscArg;

/// Sets the name of a channel preset.
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
    let address = format!("/libs/ch/{:03}/name", preset_num);
    (address, vec![OscArg::String(name.to_string())])
}

/// Sets the name of a routing preset.
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
    let address = format!("/libs/routing/{:03}/name", preset_num);
    (address, vec![OscArg::String(name.to_string())])
}

/// Sets the name of a fx preset.
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
    let address = format!("/libs/fx/{:03}/name", preset_num);
    (address, vec![OscArg::String(name.to_string())])
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
