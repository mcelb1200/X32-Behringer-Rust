//! This module provides the command definitions for the X32 misc channels.
use osc_lib::OscArg;

/// Sets the usb path of a misc channel.
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
    let address = "/-usb/path".to_string();
    (address, vec![OscArg::String(path.to_string())])
}

/// Sets the aux insert of a misc channel.
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
    let address = format!("/config/ins/aux/{}", aux_num);
    (address, vec![OscArg::Int(insert)])
}

/// Sets the ha routing of a misc channel.
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
    let address = format!("/config/routing/p/{:02}", ha_num);
    (address, vec![OscArg::Int(routing)])
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
