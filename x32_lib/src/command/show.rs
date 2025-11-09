//! Provides functions for generating OSC commands to control X32/M32 show management.
//!
//! This module covers the commands related to loading and managing shows, scenes, cues,
//! and snippets.
use osc_lib::OscArg;


// --- Address String Getters ---

/// Returns the OSC address for the show name.
pub fn show_name() -> String {
    "/-show/name".to_string()
}

/// Returns the OSC address for a cue's name.
pub fn cue_name(cue_num: u8) -> String {
    format!("/-show/cue/{:03}/name", cue_num)
}


// --- OSC Message Setters ---

/// Creates an OSC message to set the name of a show.
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
    (show_name(), vec![OscArg::String(name.to_string())])
}

/// Creates an OSC message to set the name of a cue.
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
    (cue_name(cue_num), vec![OscArg::String(name.to_string())])
}
