//! Provides functions for generating OSC commands to control X32/M32 matrix channels (1-6).
//!
//! Matrix channels are used for creating custom mixes of mix buses and main outputs, often
//! for sending to different speaker zones, recording devices, or broadcast feeds.
use osc_lib::OscArg;

// --- Address String Getters ---

/// Returns the OSC address for a matrix channel's name.
pub fn name(channel_num: u8) -> String {
    format!("/mtx/{:02}/config/name", channel_num)
}

/// Returns the OSC address for a matrix channel's color.
pub fn color(channel_num: u8) -> String {
    format!("/mtx/{:02}/config/color", channel_num)
}

/// Returns the OSC address for a matrix channel's fader level.
pub fn fader_level(channel_num: u8) -> String {
    format!("/mtx/{:02}/mix/fader", channel_num)
}

/// Returns the OSC address for a matrix channel's on/off (mute) state.
pub fn on(channel_num: u8) -> String {
    format!("/mtx/{:02}/mix/on", channel_num)
}

// --- OSC Message Setters ---

/// Creates an OSC message to set the name of a mtx channel.
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
    (
        self::name(channel_num),
        vec![OscArg::String(name.to_string())],
    )
}

/// Creates an OSC message to set the color of a mtx channel.
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
    (self::color(channel_num), vec![OscArg::Int(color)])
}

/// Creates an OSC message to set the fader level of a mtx channel.
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
    (fader_level(channel_num), vec![OscArg::Float(level)])
}

/// Creates an OSC message to set the on state of a mtx channel.
///
/// # Arguments
///
/// * `channel_num` - The mtx channel number (1-6).
/// * `on` - The new on state for the channel (0 for Off, 1 for On).
///
/// ```
/// use x32_lib::command::mtx;
///
/// let (address, args) = mtx::set_on(1, 1);
/// assert_eq!(address, "/mtx/01/mix/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (self::on(channel_num), vec![OscArg::Int(on)])
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
