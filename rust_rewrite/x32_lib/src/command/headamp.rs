//! This module provides the command definitions for the X32 headamp channels.
use osc_lib::OscArg;

/// Sets the gain of a headamp channel.
///
/// # Arguments
///
/// * `channel_num` - The headamp channel number (1-32).
/// * `gain` - The new gain for the channel (0.0-1.0).
///
/// ```
/// use x32_lib::command::headamp;
///
/// let (address, args) = headamp::set_gain(1, 0.75);
/// assert_eq!(address, "/headamp/01/gain");
/// assert_eq!(args, vec![osc_lib::OscArg::Float(0.75)]);
/// ```
pub fn set_gain(channel_num: u8, gain: f32) -> (String, Vec<OscArg>) {
    let address = format!("/headamp/{:02}/gain", channel_num);
    (address, vec![OscArg::Float(gain)])
}

/// Sets the phantom power of a headamp channel.
///
/// # Arguments
///
/// * `channel_num` - The headamp channel number (1-32).
/// * `on` - The new phantom power state for the channel (0 or 1).
///
/// ```
/// use x32_lib::command::headamp;
///
/// let (address, args) = headamp::set_phantom(1, 1);
/// assert_eq!(address, "/headamp/01/phantom");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_phantom(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    let address = format!("/headamp/{:02}/phantom", channel_num);
    (address, vec![OscArg::Int(on)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_gain() {
        let (address, args) = set_gain(1, 0.5);
        assert_eq!(address, "/headamp/01/gain");
        assert_eq!(args, vec![OscArg::Float(0.5)]);
    }

    #[test]
    fn test_set_phantom() {
        let (address, args) = set_phantom(2, 1);
        assert_eq!(address, "/headamp/02/phantom");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
