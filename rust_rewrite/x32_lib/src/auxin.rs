
//! # Auxin Module
//!
//! Controls the 8 auxiliary inputs on the X32/M32.
//!
//! This module provides functions for controlling various aspects of the auxiliary inputs,
//! including configuration, EQ, and mix settings.

use crate::common::{Color, EqType, On, CommandFlags, CommandFormat, CommandValue, X32Command};
use osc_lib::OscArg;
use std::borrow::Cow;

// Config
/// Sets the name for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_name(1, "CD Player");
/// assert_eq!(address, "/auxin/01/config/name");
/// assert_eq!(args, vec![OscArg::String("CD Player".to_string())]);
/// ```
pub fn set_name(auxin_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/config/name", auxin_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_color(1, Color::Red);
/// assert_eq!(address, "/auxin/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_color(auxin_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/config/color", auxin_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

// EQ
/// Sets the EQ band type for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `band` - The EQ band (1-4).
/// * `eq_type` - The EQ type.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::EqType;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_eq_band_type(1, 1, EqType::Peq);
/// assert_eq!(address, "/auxin/01/eq/1/type");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_eq_band_type(auxin_id: u8, band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/eq/{}/type", auxin_id, band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

// Mix
/// Sets the fader level for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `level` - The fader level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_fader(1, 0.75);
/// assert_eq!(address, "/auxin/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(auxin_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/mix/fader", auxin_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_on(1, On::On);
/// assert_eq!(address, "/auxin/01/mix/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(auxin_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/mix/on", auxin_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

pub fn get_auxin_commands(channel_num: u8) -> Result<Vec<X32Command>, String> {
    if !(1..=8).contains(&channel_num) {
        return Err(format!(
            "Invalid auxin channel number: {}. Must be between 1 and 8.",
            channel_num
        ));
    }
    let mut commands = Vec::new();
    let base = format!("/auxin/{:02}", channel_num);

    commands.push(X32Command {
        command: Cow::from("/auxin"),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(base.clone()),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });

    // Config
    let config_base = format!("{}/config", base);
    commands.push(X32Command {
        command: Cow::from(config_base.clone()),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/name", config_base)),
        format: CommandFormat::String,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/icon", config_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/color", config_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/source", config_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    // Preamp
    let preamp_base = format!("{}/preamp", base);
    commands.push(X32Command {
        command: Cow::from(preamp_base.clone()),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/trim", preamp_base)),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/invert", preamp_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    // EQ
    let eq_base = format!("{}/eq", base);
    commands.push(X32Command {
        command: Cow::from(eq_base.clone()),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/on", eq_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    for i in 1..=4 {
        let band_base = format!("{}/{}", eq_base, i);
        commands.push(X32Command {
            command: Cow::from(band_base.clone()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: Cow::from(format!("{}/type", band_base)),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: Cow::from(format!("{}/f", band_base)),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: Cow::from(format!("{}/g", band_base)),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: Cow::from(format!("{}/q", band_base)),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
    }

    // Mix
    let mix_base = format!("{}/mix", base);
    commands.push(X32Command {
        command: Cow::from(mix_base.clone()),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/on", mix_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/fader", mix_base)),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/st", mix_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/pan", mix_base)),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/mono", mix_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/mlevel", mix_base)),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    for i in 1..=16 {
        let send_base = format!("{}/{:02}", mix_base, i);
        commands.push(X32Command {
            command: Cow::from(send_base.clone()),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: Cow::from(format!("{}/on", send_base)),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: Cow::from(format!("{}/level", send_base)),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        // Odd numbered mixes also have pan, type, and panFollow parameters
        if i % 2 != 0 {
            commands.push(X32Command {
                command: Cow::from(format!("{}/pan", send_base)),
                format: CommandFormat::Float,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            });
            commands.push(X32Command {
                command: Cow::from(format!("{}/type", send_base)),
                format: CommandFormat::Int,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            });
            commands.push(X32Command {
                command: Cow::from(format!("{}/panFollow", send_base)),
                format: CommandFormat::Int,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            });
        }
    }

    // Group
    let grp_base = format!("{}/grp", base);
    commands.push(X32Command {
        command: Cow::from(grp_base.clone()),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/dca", grp_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: Cow::from(format!("{}/mute", grp_base)),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use osc_lib::OscArg;
    use crate::common::On;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/auxin/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.75)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(1, On::On);
        assert_eq!(address, "/auxin/01/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }

    #[test]
    fn test_get_auxin_commands_valid_channel() {
        let commands = get_auxin_commands(1).unwrap();
        assert!(commands.iter().any(|c| c.command == "/auxin/01/config/name"));
        assert!(commands.iter().any(|c| c.command == "/auxin/01/mix/01/pan"));
        assert!(!commands.iter().any(|c| c.command == "/auxin/01/mix/02/pan"));
    }

    #[test]
    fn test_get_auxin_commands_invalid_channel() {
        let result = get_auxin_commands(0);
        assert!(result.is_err());

        let result = get_auxin_commands(9);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_auxin_commands_command_count() {
        let commands = get_auxin_commands(1).unwrap();
        assert_eq!(commands.len(), 114);
    }
}
