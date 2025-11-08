//! This module provides the command definitions for the X32 fxrtn channels.
use osc_lib::OscArg;

/// Sets the name of a fxrtn channel.
///
/// # Arguments
///
/// * `channel_num` - The fxrtn channel number (1-8).
/// * `name` - The new name for the channel.
///
/// ```
/// use x32_lib::command::fxrtn;
///
/// let (address, args) = fxrtn::set_name(1, "Test");
/// assert_eq!(address, "/fxrtn/01/config/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_name(channel_num: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/config/name", channel_num);
    (address, vec![OscArg::String(name.to_string())])
}

/// Sets the color of a fxrtn channel.
///
/// # Arguments
///
/// * `channel_num` - The fxrtn channel number (1-8).
/// * `color` - The new color for the channel (0-15).
///
/// ```
/// use x32_lib::command::fxrtn;
///
/// let (address, args) = fxrtn::set_color(1, 2);
/// assert_eq!(address, "/fxrtn/01/config/color");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_color(channel_num: u8, color: i32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/config/color", channel_num);
    (address, vec![OscArg::Int(color)])
}


/// Sets the fader level of a fxrtn channel.
///
/// # Arguments
///
/// * `channel_num` - The fxrtn channel number (1-8).
/// * `level` - The new fader level (0.0-1.0).
///
/// ```
/// use x32_lib::command::fxrtn;
///
/// let (address, args) = fxrtn::set_fader(1, 0.75);
/// assert_eq!(address, "/fxrtn/01/mix/fader");
/// assert_eq!(args, vec![osc_lib::OscArg::Float(0.75)]);
/// ```
pub fn set_fader(channel_num: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/fader", channel_num);
    (address, vec![OscArg::Float(level)])
}

/// Sets the bus send level of a fxrtn channel.
///
/// # Arguments
///
/// * `channel_num` - The fxrtn channel number (1-8).
/// * `bus` - The bus number (1-16).
/// * `level` - The new bus send level (0.0-1.0).
///
/// ```
/// use x32_lib::command::fxrtn;
///
/// let (address, args) = fxrtn::set_bus_send_level(1, 1, 0.75);
/// assert_eq!(address, "/fxrtn/01/mix/01/level");
/// assert_eq!(args, vec![osc_lib::OscArg::Float(0.75)]);
/// ```
pub fn set_bus_send_level(channel_num: u8, bus: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/{:02}/level", channel_num, bus);
    (address, vec![OscArg::Float(level)])
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_name() {
        let (address, args) = set_name(1, "Test");
        assert_eq!(address, "/fxrtn/01/config/name");
        assert_eq!(args, vec![OscArg::String("Test".to_string())]);
    }

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.5);
        assert_eq!(address, "/fxrtn/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.5)]);
    }

    #[test]
    fn test_set_bus_send_level() {
        let (address, args) = set_bus_send_level(2, 1, 0.25);
        assert_eq!(address, "/fxrtn/02/mix/01/level");
        assert_eq!(args, vec![OscArg::Float(0.25)]);
    }
}
