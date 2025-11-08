//! This module provides the command definitions for the X32 mtx channels.
use osc_lib::OscArg;

/// Sets the name of a mtx channel.
///
/// # Arguments
///
/// * `channel_num` - The mtx channel number (1-6).
/// * `name` - The new name for the channel.
///
/// ```
/// use x32_lib::command::mtx;
///
/// let (address, args) = mtx::set_name(1, "Test");
/// assert_eq!(address, "/mtx/01/config/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_name(channel_num: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/config/name", channel_num);
    (address, vec![OscArg::String(name.to_string())])
}

/// Sets the color of a mtx channel.
///
/// # Arguments
///
/// * `channel_num` - The mtx channel number (1-6).
/// * `color` - The new color for the channel (0-15).
///
/// ```
/// use x32_lib::command::mtx;
///
/// let (address, args) = mtx::set_color(1, 2);
/// assert_eq!(address, "/mtx/01/config/color");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_color(channel_num: u8, color: i32) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/config/color", channel_num);
    (address, vec![OscArg::Int(color)])
}


/// Sets the fader level of a mtx channel.
///
/// # Arguments
///
/// * `channel_num` - The mtx channel number (1-6).
/// * `level` - The new fader level (0.0-1.0).
///
/// ```
/// use x32_lib::command::mtx;
///
/// let (address, args) = mtx::set_fader(1, 0.75);
/// assert_eq!(address, "/mtx/01/mix/fader");
/// assert_eq!(args, vec![osc_lib::OscArg::Float(0.75)]);
/// ```
pub fn set_fader(channel_num: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/mix/fader", channel_num);
    (address, vec![OscArg::Float(level)])
}

/// Sets the on state of a mtx channel.
///
/// # Arguments
///
/// * `channel_num` - The mtx channel number (1-6).
/// * `on` - The new on state for the channel (0 or 1).
///
/// ```
/// use x32_lib::command::mtx;
///
/// let (address, args) = mtx::set_on(1, 1);
/// assert_eq!(address, "/mtx/01/mix/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/mix/on", channel_num);
    (address, vec![OscArg::Int(on)])
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.5);
        assert_eq!(address, "/mtx/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.5)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(2, 1);
        assert_eq!(address, "/mtx/02/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
