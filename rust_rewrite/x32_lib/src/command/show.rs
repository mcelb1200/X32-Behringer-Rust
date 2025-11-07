//! This module provides the command definitions for the X32 show channels.
use osc_lib::OscArg;

/// Sets the name of a show.
///
/// # Arguments
///
/// * `name` - The new name for the show.
///
/// ```
/// use x32_lib::command::show;
///
/// let (address, args) = show::set_show_name("Test");
/// assert_eq!(address, "/-show/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_show_name(name: &str) -> (String, Vec<OscArg>) {
    let address = "/-show/name".to_string();
    (address, vec![OscArg::String(name.to_string())])
}

/// Sets the name of a cue.
///
/// # Arguments
///
/// * `cue_num` - The cue number (0-99).
/// * `name` - The new name for the cue.
///
/// ```
/// use x32_lib::command::show;
///
/// let (address, args) = show::set_cue_name(1, "Test");
/// assert_eq!(address, "/-show/cue/001/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_cue_name(cue_num: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/-show/cue/{:03}/name", cue_num);
    (address, vec![OscArg::String(name.to_string())])
}
