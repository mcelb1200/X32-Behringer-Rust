//! # Channel Module
//!
//! Controls the 32 input channels on the X32/M32.
//!
//! This module provides functions for controlling various aspects of the input channels,
//! including configuration, preamp, gate, dynamics, EQ, and mix settings.

use crate::common::{Color, EqType, InsertPosition, InsertSelection, On, CommandFlags, CommandFormat, CommandValue, X32Command};
use osc_lib::OscArg;

// Config
/// Sets the name for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_name(1, "Vocal");
/// assert_eq!(address, "/ch/01/config/name");
/// assert_eq!(args, vec![OscArg::String("Vocal".to_string())]);
/// ```
pub fn set_name(channel_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/config/name", channel_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_color(1, Color::Blue);
/// assert_eq!(address, "/ch/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(4)]);
/// ```
pub fn set_color(channel_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/config/color", channel_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

// Preamp
/// Sets the input source for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `source` - The input source ID.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_input_source(1, 0);
/// assert_eq!(address, "/ch/01/config/insrc");
/// assert_eq!(args, vec![OscArg::Int(0)]);
/// ```
pub fn set_input_source(channel_id: u8, source: u8) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/config/insrc", channel_id);
    let args = vec![OscArg::Int(source as i32)];
    (address, args)
}

// Delay
/// Sets the delay on or off for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `on` - The delay state (`On::On` or `On::Off`).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_delay_on(1, On::On);
/// assert_eq!(address, "/ch/01/delay/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_delay_on(channel_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/delay/on", channel_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the delay time for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `time` - The delay time in milliseconds (0.0 to 500.0).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_delay_time(1, 100.0);
/// assert_eq!(address, "/ch/01/delay/time");
/// assert_eq!(args, vec![OscArg::Float(100.0)]);
/// ```
pub fn set_delay_time(channel_id: u8, time: f32) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/delay/time", channel_id);
    let args = vec![OscArg::Float(time)];
    (address, args)
}

// Mix
/// Sets the fader level for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `level` - The fader level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_fader(1, 0.75);
/// assert_eq!(address, "/ch/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(channel_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/mix/fader", channel_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_on(1, On::On);
/// assert_eq!(address, "/ch/01/mix/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(channel_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/mix/on", channel_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

// Insert
/// Sets the insert on or off for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `on` - The insert state (`On::On` or `On::Off`).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_on(1, On::On);
/// assert_eq!(address, "/ch/01/insert/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_insert_on(channel_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/insert/on", channel_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the insert position for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `pos` - The insert position (`InsertPosition::Pre` or `InsertPosition::Post`).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::InsertPosition;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_pos(1, InsertPosition::Pre);
/// assert_eq!(address, "/ch/01/insert/pos");
/// assert_eq!(args, vec![OscArg::Int(0)]);
/// ```
pub fn set_insert_pos(channel_id: u8, pos: InsertPosition) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/insert/pos", channel_id);
    let args = vec![OscArg::Int(pos as i32)];
    (address, args)
}

/// Sets the insert selection for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `sel` - The insert selection.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::InsertSelection;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_sel(1, InsertSelection::Fx1L);
/// assert_eq!(address, "/ch/01/insert/sel");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_insert_sel(channel_id: u8, sel: InsertSelection) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/insert/sel", channel_id);
    let args = vec![OscArg::Int(sel as i32)];
    (address, args)
}

// EQ
/// Sets the EQ band type for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `band` - The EQ band (1-4).
/// * `eq_type` - The EQ type.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::EqType;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_eq_band_type(1, 1, EqType::Peq);
/// assert_eq!(address, "/ch/01/eq/1/type");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_eq_band_type(channel_id: u8, band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/eq/{}/type", channel_id, band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

pub fn get_channel_commands(channel_num: u8) -> Result<Vec<X32Command<String>>, String> {
    if !(1..=32).contains(&channel_num) {
        return Err(format!(
            "Invalid channel number: {}. Must be between 1 and 32.",
            channel_num
        ));
    }
    let mut commands = Vec::new();
    let base = format!("/ch/{:02}", channel_num);

    commands.push(X32Command {
        command: "/ch".to_string(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });

    // Config
    let config_base = format!("{}/config", base);
    commands.push(X32Command {
        command: config_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/name", config_base),
        format: CommandFormat::String,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/icon", config_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/color", config_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/source", config_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    // Preamp
    let preamp_base = format!("{}/preamp", base);
    commands.push(X32Command {
        command: preamp_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/trim", preamp_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/invert", preamp_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    // Gate
    let gate_base = format!("{}/gate", base);
    commands.push(X32Command {
        command: gate_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/on", gate_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/mode", gate_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/thr", gate_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/range", gate_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/attack", gate_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/hold", gate_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/release", gate_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/keysrc", gate_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    let filter_base = format!("{}/filter", gate_base);
    commands.push(X32Command {
        command: filter_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/on", filter_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/type", filter_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/f", filter_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    // Dynamics
    let dyn_base = format!("{}/dyn", base);
    commands.push(X32Command {
        command: dyn_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/on", dyn_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/mode", dyn_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/det", dyn_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/env", dyn_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/thr", dyn_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/ratio", dyn_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/knee", dyn_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/mgain", dyn_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/attack", dyn_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/hold", dyn_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/release", dyn_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/pos", dyn_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/keysrc", dyn_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/mix", dyn_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/auto", dyn_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    let dyn_filter_base = format!("{}/filter", dyn_base);
    commands.push(X32Command {
        command: dyn_filter_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/on", dyn_filter_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/type", dyn_filter_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/f", dyn_filter_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    // Insert
    let insert_base = format!("{}/insert", base);
    commands.push(X32Command {
        command: insert_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/on", insert_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/pos", insert_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/sel", insert_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    // EQ
    let eq_base = format!("{}/eq", base);
    commands.push(X32Command {
        command: eq_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/on", eq_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    for i in 1..=4 {
        let band_base = format!("{}/{}", eq_base, i);
        commands.push(X32Command {
            command: band_base.clone(),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: format!("{}/type", band_base),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: format!("{}/f", band_base),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: format!("{}/g", band_base),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: format!("{}/q", band_base),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
    }

    // Mix
    let mix_base = format!("{}/mix", base);
    commands.push(X32Command {
        command: mix_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/on", mix_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/fader", mix_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/st", mix_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/pan", mix_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/mono", mix_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/mlevel", mix_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    for i in 1..=16 {
        let send_base = format!("{}/{:02}", mix_base, i);
        commands.push(X32Command {
            command: send_base.clone(),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: format!("{}/on", send_base),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        commands.push(X32Command {
            command: format!("{}/level", send_base),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        });
        // Odd numbered mixes also have pan, type, and panFollow parameters
        if i % 2 != 0 {
            commands.push(X32Command {
                command: format!("{}/pan", send_base),
                format: CommandFormat::Float,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            });
            commands.push(X32Command {
                command: format!("{}/type", send_base),
                format: CommandFormat::Int,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            });
            commands.push(X32Command {
                command: format!("{}/panFollow", send_base),
                format: CommandFormat::Int,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            });
        }
    }

    // Group
    let grp_base = format!("{}/grp", base);
    commands.push(X32Command {
        command: grp_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/dca", grp_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/mute", grp_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    // Automix
    let amix_base = format!("{}/automix", base);
    commands.push(X32Command {
        command: amix_base.clone(),
        format: CommandFormat::StringList(&[]),
        flags: CommandFlags::F_FND,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/group", amix_base),
        format: CommandFormat::Int,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });
    commands.push(X32Command {
        command: format!("{}/weight", amix_base),
        format: CommandFormat::Float,
        flags: CommandFlags::F_XET,
        value: CommandValue::None,
    });

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/ch/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.75)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(1, On::On);
        assert_eq!(address, "/ch/01/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
