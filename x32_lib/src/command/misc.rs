//! Provides functions for generating OSC commands for miscellaneous X32/M32 settings.
//!
//! This module covers a range of settings that don't fit into the other categories,
//! such as USB playback routing, auxiliary input inserts, and headamp routing.
use osc_lib::OscArg;


// --- Address String Getters ---

/// Returns the OSC address for the USB player path.
pub fn usb_path() -> String {
    "/-usb/path".to_string()
}

/// Returns the OSC address for an auxiliary input's insert.
pub fn aux_insert(aux_num: u8) -> String {
    format!("/config/ins/aux/{}", aux_num)
}

/// Returns the OSC address for a headamp's routing.
pub fn ha_routing(ha_num: u8) -> String {
    format!("/config/routing/p/{:02}", ha_num)
}


// --- OSC Message Setters ---

/// Creates an OSC message to set the usb path of a misc channel.
///
/// # Arguments
///
/// * `path` - The new path for the channel.
///
/// ```
/// use x32_lib::command::misc;
///
/// let (address, args) = misc::set_usb_path("Test");
/// assert_eq!(address, "/-usb/path");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_usb_path(path: &str) -> (String, Vec<OscArg>) {
    (usb_path(), vec![OscArg::String(path.to_string())])
}

/// Creates an OSC message to set the aux insert of a misc channel.
///
/// # Arguments
///
/// * `aux_num` - The aux number (1-6).
/// * `insert` - The new insert for the channel (0-18).
///
/// ```
/// use x32_lib::command::misc;
///
/// let (address, args) = misc::set_aux_insert(1, 2);
/// assert_eq!(address, "/config/ins/aux/1");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_aux_insert(aux_num: u8, insert: i32) -> (String, Vec<OscArg>) {
    (aux_insert(aux_num), vec![OscArg::Int(insert)])
}

/// Creates an OSC message to set the ha routing of a misc channel.
///
/// # Arguments
///
/// * `ha_num` - The ha number (0-31).
/// * `routing` - The new routing for the channel (0-7).
///
/// ```
/// use x32_lib::command::misc;
///
/// let (address, args) = misc::set_ha_routing(1, 2);
/// assert_eq!(address, "/config/routing/p/01");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_ha_routing(ha_num: u8, routing: i32) -> (String, Vec<OscArg>) {
    (ha_routing(ha_num), vec![OscArg::Int(routing)])
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_usb_path() {
        let (address, args) = set_usb_path("Test");
        assert_eq!(address, "/-usb/path");
        assert_eq!(args, vec![OscArg::String("Test".to_string())]);
    }

    #[test]
    fn test_set_aux_insert() {
        let (address, args) = set_aux_insert(1, 2);
        assert_eq!(address, "/config/ins/aux/1");
        assert_eq!(args, vec![OscArg::Int(2)]);
    }

    #[test]
    fn test_set_ha_routing() {
        let (address, args) = set_ha_routing(1, 2);
        assert_eq!(address, "/config/routing/p/01");
        assert_eq!(args, vec![OscArg::Int(2)]);
    }
}
