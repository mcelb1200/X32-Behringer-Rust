//! # Bus Module
//!
//! Controls the 16 mix buses on the X32/M32.
//!
//! This module provides functions for controlling various aspects of the mix buses,
//! including configuration, dynamics, EQ, and mix settings.

use crate::common::{Color, EqType, On, CommandFlags, CommandFormat, CommandValue, X32Command, XCOLORS, OFF_ON, XDYMODE, XDYDET, XDYENV, XDYPPOS, XDYFTYP, XISEL, XEQTY1};
use osc_lib::OscArg;

// Config
/// Sets the name for a specific mix bus.
///
/// # Arguments
///
/// * `bus_id` - The ID of the mix bus (1-16).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::bus;
/// use osc_lib::OscArg;
///
/// let (address, args) = bus::set_name(1, "Monitors");
/// assert_eq!(address, "/bus/01/config/name");
/// assert_eq!(args, vec![OscArg::String("Monitors".to_string())]);
/// ```
pub fn set_name(bus_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/bus/{:02}/config/name", bus_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for a specific mix bus.
///
/// # Arguments
///
/// * `bus_id` - The ID of the mix bus (1-16).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::bus;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = bus::set_color(1, Color::Green);
/// assert_eq!(address, "/bus/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_color(bus_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/bus/{:02}/config/color", bus_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

// Dynamics
/// Sets the dynamics on or off for a specific mix bus.
///
/// # Arguments
///
/// * `bus_id` - The ID of the mix bus (1-16).
/// * `on` - The dynamics state (`On::On` or `On::Off`).
///
/// # Example
///
/// ```
/// use x32_lib::bus;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = bus::set_dyn_on(1, On::On);
/// assert_eq!(address, "/bus/01/dyn/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_dyn_on(bus_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/bus/{:02}/dyn/on", bus_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

// EQ
/// Sets the EQ band type for a specific mix bus.
///
/// # Arguments
///
/// * `bus_id` - The ID of the mix bus (1-16).
/// * `band` - The EQ band (1-6).
/// * `eq_type` - The EQ type.
///
/// # Example
///
/// ```
/// use x32_lib::bus;
/// use x32_lib::common::EqType;
/// use osc_lib::OscArg;
///
/// let (address, args) = bus::set_eq_band_type(1, 1, EqType::Peq);
/// assert_eq!(address, "/bus/01/eq/1/type");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_eq_band_type(bus_id: u8, band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/bus/{:02}/eq/{}/type", bus_id, band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

// Mix
/// Sets the fader level for a specific mix bus.
///
/// # Arguments
///
/// * `bus_id` - The ID of the mix bus (1-16).
/// * `level` - The fader level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::bus;
/// use osc_lib::OscArg;
///
/// let (address, args) = bus::set_fader(1, 0.75);
/// assert_eq!(address, "/bus/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(bus_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/bus/{:02}/mix/fader", bus_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for a specific mix bus.
///
/// # Arguments
///
/// * `bus_id` - The ID of the mix bus (1-16).
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
///
/// # Example
///
/// ```
/// use x32_lib::bus;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = bus::set_on(1, On::On);
/// assert_eq!(address, "/bus/01/mix/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(bus_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/bus/{:02}/mix/on", bus_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

pub fn get_bus_commands(bus_number: i32) -> Vec<X32Command> {
    let bus_prefix = format!("/bus/{:02}", bus_number);
    let mut commands = vec![
        X32Command {
            command: "/bus",
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/config", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/config/name", bus_prefix).into_boxed_str()),
            format: CommandFormat::String,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/config/icon", bus_prefix).into_boxed_str()),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/config/color", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&XCOLORS),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/on", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/mode", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&XDYMODE),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/det", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&XDYDET),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/env", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&XDYENV),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/thr", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/ratio", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/knee", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/mgain", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/attack", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/hold", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/release", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/pos", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&XDYPPOS),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/keysrc", bus_prefix).into_boxed_str()),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/mix", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/auto", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/filter", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/filter/on", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/filter/type", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&XDYFTYP),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/dyn/filter/f", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/insert", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/insert/on", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/insert/pos", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&XDYPPOS),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/insert/sel", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&XISEL),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/eq", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::Int(1),
        },
        X32Command {
            command: Box::leak(format!("{}/eq/on", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
    ];

    for i in 1..=6 {
        commands.extend(vec![
            X32Command {
                command: Box::leak(format!("{}/eq/{}", bus_prefix, i).into_boxed_str()),
                format: CommandFormat::StringList(&[]),
                flags: CommandFlags::F_FND,
                value: CommandValue::None,
            },
            X32Command {
                command: Box::leak(format!("{}/eq/{}/type", bus_prefix, i).into_boxed_str()),
                format: CommandFormat::StringList(&XEQTY1),
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            },
            X32Command {
                command: Box::leak(format!("{}/eq/{}/f", bus_prefix, i).into_boxed_str()),
                format: CommandFormat::Float,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            },
            X32Command {
                command: Box::leak(format!("{}/eq/{}/g", bus_prefix, i).into_boxed_str()),
                format: CommandFormat::Float,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            },
            X32Command {
                command: Box::leak(format!("{}/eq/{}/q", bus_prefix, i).into_boxed_str()),
                format: CommandFormat::Float,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            },
        ]);
    }

    commands.extend(vec![
        X32Command {
            command: Box::leak(format!("{}/mix", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/mix/on", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/mix/fader", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/mix/st", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/mix/pan", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/mix/mono", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/mix/mlevel", bus_prefix).into_boxed_str()),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
    ]);

    for i in 1..=6 {
        if i % 2 != 0 {
            commands.extend(vec![
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::StringList(&[]),
                    flags: CommandFlags::F_FND,
                    value: CommandValue::None,
                },
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}/on", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::StringList(&OFF_ON),
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}/level", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::Float,
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}/pan", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::Float,
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}/type", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::StringList(&OFF_ON),
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}/panFollow", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::Int,
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
            ]);
        } else {
            commands.extend(vec![
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::StringList(&[]),
                    flags: CommandFlags::F_FND,
                    value: CommandValue::None,
                },
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}/on", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::StringList(&OFF_ON),
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: Box::leak(format!("{}/mix/{:02}/level", bus_prefix, i).into_boxed_str()),
                    format: CommandFormat::Float,
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
            ]);
        }
    }

    commands.extend(vec![
        X32Command {
            command: Box::leak(format!("{}/grp", bus_prefix).into_boxed_str()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/grp/dca", bus_prefix).into_boxed_str()),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: Box::leak(format!("{}/grp/mute", bus_prefix).into_boxed_str()),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
    ]);

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/bus/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.75)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(1, On::On);
        assert_eq!(address, "/bus/01/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}