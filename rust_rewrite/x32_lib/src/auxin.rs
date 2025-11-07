//! # Auxin Module
//!
//! Controls the 8 Auxiliary input channels on the X32/M32.
//!
//! This module provides functions for setting various parameters of the Aux channels,
//! including name, color, fader level, and mute status.

use crate::common::{
    Color, EqType, On,
};
use osc_lib::OscArg;

/// Returns a list of all OSC commands for a specific Aux input channel.
///
/// This function generates commands for config, preamp, EQ, mix, and automix settings.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the Aux input channel (1-8).
///
/// # Returns
///
/// A `Vec<String>` containing all the OSC commands.
pub fn get_auxin_commands(auxin_id: u8) -> Result<Vec<String>, &'static str> {
    if !(1..=8).contains(&auxin_id) {
        return Err("Invalid Auxin ID. Must be between 1 and 8.");
    }

    let mut commands = Vec::new();
    let id_str = format!("{:02}", auxin_id);

    // Config and Preamp
    commands.push(format!("/auxin/{}/config/name", id_str));
    commands.push(format!("/auxin/{}/config/icon", id_str));
    commands.push(format!("/auxin/{}/config/color", id_str));
    commands.push(format!("/auxin/{}/config/source", id_str));
    commands.push(format!("/auxin/{}/preamp/trim", id_str));
    commands.push(format!("/auxin/{}/preamp/invert", id_str));

    // EQ
    commands.push(format!("/auxin/{}/eq/on", id_str));
    commands.push(format!("/auxin/{}/eq/reset", id_str));
    for i in 1..=4 {
        commands.push(format!("/auxin/{}/eq/{}/type", id_str, i));
        commands.push(format!("/auxin/{}/eq/{}/freq", id_str, i));
        commands.push(format!("/auxin/{}/eq/{}/gain", id_str, i));
        commands.push(format!("/auxin/{}/eq/{}/q", id_str, i));
    }

    // Mix
    commands.push(format!("/auxin/{}/mix/on", id_str));
    commands.push(format!("/auxin/{}/mix/fader", id_str));
    commands.push(format!("/auxin/{}/mix/st", id_str));
    commands.push(format!("/auxin/{}/mix/pan", id_str));
    commands.push(format!("/auxin/{}/mix/mono", id_str));
    commands.push(format!("/auxin/{}/mix/mlevel", id_str));

    for i in 1..=16 {
        commands.push(format!("/auxin/{}/mix/{:02}/on", id_str, i));
        commands.push(format!("/auxin/{}/mix/{:02}/level", id_str, i));
        commands.push(format!("/auxin/{}/mix/{:02}/pan", id_str, i));
        commands.push(format!("/auxin/{}/mix/{:02}/type", id_str, i));
    }

    // Automix
    commands.push(format!("/auxin/{}/automix/group", id_str));
    commands.push(format!("/auxin/{}/automix/weight", id_str));

    Ok(commands)
}

/// Sets the name of a specific Aux input channel.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the Aux input channel (1-8).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_name(1, "AUX 1");
/// assert_eq!(address, "/auxin/01/config/name");
/// assert_eq!(args, vec![OscArg::String("AUX 1".to_string())]);
/// ```
pub fn set_name(auxin_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/config/name", auxin_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color of a specific Aux input channel.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the Aux input channel (1-8).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_color(1, Color::Red);
/// assert_eq!(address, "/auxin/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_color(auxin_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/config/color", auxin_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

/// Sets the EQ band type for a specific Aux input channel.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the Aux input channel (1-8).
/// * `band_id` - The ID of the EQ band (1-4).
/// * `eq_type` - The EQ type to set.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::EqType;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_eq_band_type(1, 1, EqType::Lcut);
/// assert_eq!(address, "/auxin/01/eq/1/type");
/// assert_eq!(args, vec![OscArg::Int(0)]);
/// ```
pub fn set_eq_band_type(auxin_id: u8, band_id: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/eq/{}/type", auxin_id, band_id);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

/// Sets the fader level of a specific Aux input channel.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the Aux input channel (1-8).
/// * `level` - The fader level to set (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_fader(1, 0.75);
/// assert_eq!(address, "/auxin/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(auxin_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/mix/fader", auxin_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute status of a specific Aux input channel.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the Aux input channel (1-8).
/// * `on` - The mute status to set.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_on(1, On::On);
/// assert_eq!(address, "/auxin/01/mix/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(auxin_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/mix/on", auxin_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_auxin_commands_valid_channel() {
        let result = get_auxin_commands(1);
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert!(commands.contains(&"/auxin/01/config/name".to_string()));
        assert!(commands.contains(&"/auxin/01/mix/fader".to_string()));
        assert!(commands.contains(&"/auxin/01/eq/4/q".to_string()));
    }

    #[test]
    fn test_get_auxin_commands_invalid_channel() {
        let result = get_auxin_commands(0);
        assert!(result.is_err());

        let result = get_auxin_commands(9);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_auxin_commands_command_count() {
        let result = get_auxin_commands(1);
        assert!(result.is_ok());
        let commands = result.unwrap();
        // 6 (config/preamp) + 2 (eq on/reset) + 4*4 (eq bands) + 6 (mix) + 16*4 (mix sends) + 2 (automix) = 96
        assert_eq!(commands.len(), 96);
    }

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/auxin/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.75f32)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(1, On::On);
        assert_eq!(address, "/auxin/01/mix/on");
        assert_eq!(args, vec![OscArg::Int(1i32)]);
    }
}
