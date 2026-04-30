//! Provides functions for generating OSC commands to control X32/M32 preferences and status.
//!
//! This module covers global settings and status indicators, such as the currently selected
//! screen, solo status, and other preferences.
use osc_lib::OscArg;

// --- Address String Getters ---

/// Returns the OSC address for the screen name.
pub fn name() -> String {
    "/-stat/screen/name".to_string()
}

/// Returns the OSC address for the solo channel.
pub fn solo_channel() -> String {
    "/-stat/solosw/ch".to_string()
}

/// Returns the OSC address for the solo on/off state.
pub fn solo_on_addr() -> String {
    "/-stat/solosw/on".to_string()
}

// --- OSC Message Setters ---

/// Creates an OSC message to set the name of a prefstat channel.
///
/// # Arguments
///
/// * `name` - The new name for the channel.
///
/// ```
/// use x32_lib::command::prefstat;
///
/// let (address, args) = prefstat::set_name("Test");
/// assert_eq!(address, "/-stat/screen/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_name(name: &str) -> (String, Vec<OscArg>) {
    (self::name(), vec![OscArg::String(name.to_string())])
}

/// Creates an OSC message to set the solo channel of a prefstat channel.
///
/// # Arguments
///
/// * `channel_num` - The solo channel number.
///
/// ```
/// use x32_lib::command::prefstat;
///
/// let (address, args) = prefstat::set_solo_channel(1);
/// assert_eq!(address, "/-stat/solosw/ch");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_solo_channel(channel_num: i32) -> (String, Vec<OscArg>) {
    (solo_channel(), vec![OscArg::Int(channel_num)])
}

/// Creates an OSC message to set the solo on state of a prefstat channel.
///
/// # Arguments
///
/// * `on` - The new solo on state for the channel (0 for Off, 1 for On).
///
/// ```
/// use x32_lib::command::prefstat;
///
/// let (address, args) = prefstat::solo_on(1);
/// assert_eq!(address, "/-stat/solosw/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn solo_on(on: i32) -> (String, Vec<OscArg>) {
    (solo_on_addr(), vec![OscArg::Int(on)])
}
