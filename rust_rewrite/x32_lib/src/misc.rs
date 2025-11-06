//! # Miscellaneous Module
//!
//! Controls miscellaneous settings on the X32/M32.
//!
//! This module provides functions for controlling various settings that don't fit into other categories,
//! such as USB file browsing, undo timing, and headamp routing.

use osc_lib::OscArg;

/// Sets the USB path for the file browser.
///
/// # Arguments
///
/// * `path` - The path to set.
///
/// # Example
///
/// ```
/// use x32_lib::misc;
/// use osc_lib::OscArg;
///
/// let (address, args) = misc::set_usb_path("A:/");
/// assert_eq!(address, "/-usb/path");
/// assert_eq!(args, vec![OscArg::String("A:/".to_string())]);
/// ```
pub fn set_usb_path(path: &str) -> (String, Vec<OscArg>) {
    let address = "/-usb/path".to_string();
    let args = vec![OscArg::String(path.to_string())];
    (address, args)
}

/// Sets the undo time.
///
/// # Arguments
///
/// * `time` - The time string to set.
pub fn set_undo_time(time: &str) -> (String, Vec<OscArg>) {
    let address = "/undo/time".to_string();
    let args = vec![OscArg::String(time.to_string())];
    (address, args)
}

/// Sets the insert for a specific auxiliary input.
///
/// # Arguments
///
/// * `aux_id` - The ID of the auxiliary input (1-6).
/// * `insert` - The insert ID.
///
/// # Example
///
/// ```
/// use x32_lib::misc;
/// use osc_lib::OscArg;
///
/// let (address, args) = misc::set_aux_insert(1, 1);
/// assert_eq!(address, "/insert/aux/1");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_aux_insert(aux_id: u8, insert: u8) -> (String, Vec<OscArg>) {
    let address = format!("/insert/aux/{}", aux_id);
    let args = vec![OscArg::Int(insert as i32)];
    (address, args)
}

/// Sets the routing for a specific headamp.
///
/// # Arguments
///
/// * `ha_id` - The ID of the headamp (0-39).
/// * `index` - The routing index.
///
/// # Example
///
/// ```
/// use x32_lib::misc;
/// use osc_lib::OscArg;
///
/// let (address, args) = misc::set_ha_routing(0, 1);
/// assert_eq!(address, "/-ha/00/index");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_ha_routing(ha_id: u8, index: u8) -> (String, Vec<OscArg>) {
    let address = format!("/-ha/{:02}/index", ha_id);
    let args = vec![OscArg::Int(index as i32)];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_usb_path() {
        let (address, args) = set_usb_path("A:/");
        assert_eq!(address, "/-usb/path");
        assert_eq!(args, vec![OscArg::String("A:/".to_string())]);
    }

    #[test]
    fn test_set_aux_insert() {
        let (address, args) = set_aux_insert(1, 1);
        assert_eq!(address, "/insert/aux/1");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }

    #[test]
    fn test_set_ha_routing() {
        let (address, args) = set_ha_routing(0, 1);
        assert_eq!(address, "/-ha/00/index");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
