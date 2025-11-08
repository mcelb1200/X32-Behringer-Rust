//! This module provides the command definitions for the X32 prefstat channels.
use osc_lib::OscArg;

/// Sets the name of a prefstat channel.
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
    let address = "/-stat/screen/name".to_string();
    (address, vec![OscArg::String(name.to_string())])
}

/// Sets the solo channel of a prefstat channel.
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
    let address = "/-stat/solosw/ch".to_string();
    (address, vec![OscArg::Int(channel_num)])
}

/// Sets the solo on state of a prefstat channel.
///
/// # Arguments
///
/// * `on` - The new solo on state for the channel (0 or 1).
///
/// ```
/// use x32_lib::command::prefstat;
///
/// let (address, args) = prefstat::solo_on(1);
/// assert_eq!(address, "/-stat/solosw/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn solo_on(on: i32) -> (String, Vec<OscArg>) {
    let address = "/-stat/solosw/on".to_string();
    (address, vec![OscArg::Int(on)])
}
