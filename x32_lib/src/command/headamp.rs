//! Provides functions for generating OSC commands to control X32/M32 headamps (preamps).
//!
//! The headamp controls are responsible for the analog gain of the microphone preamplifiers
//! and for switching 48V phantom power on or off.
use osc_lib::OscArg;


// --- Address String Getters ---

/// Returns the OSC address for a headamp channel's gain.
pub fn gain(channel_num: u8) -> String {
    format!("/headamp/{:02}/gain", channel_num)
}

/// Returns the OSC address for a headamp channel's phantom power state.
pub fn phantom(channel_num: u8) -> String {
    format!("/headamp/{:02}/phantom", channel_num)
}


// --- OSC Message Setters ---

/// Creates an OSC message to set the gain of a headamp channel.
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
    (self::gain(channel_num), vec![OscArg::Float(gain)])
}

/// Creates an OSC message to set the phantom power of a headamp channel.
///
/// # Arguments
///
/// * `channel_num` - The headamp channel number (1-32).
/// * `on` - The new phantom power state for the channel (0 for Off, 1 for On).
///
/// ```
/// use x32_lib::command::headamp;
///
/// let (address, args) = headamp::set_phantom(1, 1);
/// assert_eq!(address, "/headamp/01/phantom");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_phantom(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (self::phantom(channel_num), vec![OscArg::Int(on)])
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
