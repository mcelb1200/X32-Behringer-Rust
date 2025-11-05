
//
// X32Channel.h
//
// Created on: 4 févr. 2015
// Author: patrick
//

//! # X32 Channel Command Generation
//!
//! This module provides the structures and functions necessary to programmatically
//! generate a complete list of OSC (Open Sound Control) commands for a specific
//! channel on a Behringer X32 digital mixer. It is a Rust reimplementation of
//! the original `X32Channel.h` C header file.
//!
//! The primary function, `generate_channel_commands`, creates a vector of `X32Command`
//! structs, each representing a specific OSC command path and its properties. This
//! approach avoids the large, repetitive static arrays found in the original C code,
//! leading to a more maintainable and idiomatic Rust implementation.

/// Represents the various data types for OSC commands.
#[derive(Debug, Clone)]
pub enum CommandType {
    /// No data type.
    Nil,
    /// 32-bit integer.
    I32,
    F32,
    S32,
    B32,
    E32,
    P32,
    Fx32,
    OffOn,
    Cmono,
    Csolo,
    Ctalk,
    Ctalkab,
    Cosc,
    Croutsw,
    Croutin,
    Croutac,
    Croutot,
    Croutplay,
    Cctrl,
    Cenc,
    Ctape,
    Cmix,
    Chco,
    Chde,
    Chpr,
    Chga,
    Chgf,
    Chdy,
    Chdf,
    Chin,
    Cheq,
    Chmx,
    Chmo,
    Chme,
    Chgrp,
    Chamix,
    Axpr,
    Bsco,
    Mxpr,
    Mxdy,
    Msmx,
    Fxtyp1,
    Fxsrc,
    Fxpar1,
    Fxtyp2,
    Fxpar2,
    Omain,
    Omain2,
    Op16,
    Omaind,
    Hamp,
    Prefs,
    Pir,
    Piq,
    Pcard,
    Prta,
    Pip,
    Paddr,
    Pmask,
    Pgway,
    Stat,
    Sscreen,
    Scha,
    Smet,
    Srou,
    Sset,
    Slib,
    Sfx,
    Smon,
    Susb,
    Ssce,
    Sass,
    Ssolosw,
    Saes,
    Stape,
    Sosc,
    Stalk,
    Usb,
    Snam,
    Scue,
    Sscn,
    Ssnp,
    Ha,
    Action,
    Urec,
    Slibs,
    D48,
    D48a,
    D48g,
    Urouo,
    Uroui,
    Pkey,
}

/// Represents the value of an OSC command argument.
#[derive(Debug, Clone)]
pub enum CommandValue {
    /// An integer value.
    Int(i32),
    /// A floating-point value.
    Float(f32),
    /// A string value.
    String(String),
    /// A blob of binary data.
    Data(Vec<u8>),
}

/// Represents a single X32 OSC command.
#[derive(Debug, Clone)]
pub struct X32Command {
    /// The OSC command path (e.g., `/ch/01/mix/fader`).
    pub command: String,
    /// The data type of the command's argument.
    pub format: CommandType,
    /// Flags that define the command's behavior (e.g., `F_FND`, `F_XET`).
    pub flags: i32,
    /// The value of the command's argument.
    pub value: CommandValue,
    /// Optional list of node strings associated with the command.
    pub node: Option<Vec<String>>,
}

const F_FND: i32 = 0x0008;
const F_XET: i32 = 0x0003;

/// Generates a list of all OSC commands for a specific channel.
///
/// This function programmatically creates a `Vec<X32Command>` containing all
/// the OSC commands for a given channel number. This includes commands for
/// configuration, preamp, gate, dynamics, EQ, and mix sends.
///
/// # Arguments
///
/// * `channel_number` - The channel number (1-32) for which to generate commands.
///
/// # Returns
///
/// A `Vec<X32Command>` containing all OSC commands for the specified channel.
///
/// # Example
///
/// ```
/// use x32_lib::channel::{generate_channel_commands, X32Command, CommandType};
///
/// let channel_commands = generate_channel_commands(1);
///
/// assert_eq!(channel_commands.len(), 155);
/// assert_eq!(channel_commands[0].command, "/ch/01");
/// ```
pub fn generate_channel_commands(channel_number: u8) -> Vec<X32Command> {
    let mut commands = Vec::new();

    commands.push(X32Command {
        command: format!("/ch/{:02}", channel_number),
        format: CommandType::Chco,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/config", channel_number),
        format: CommandType::Chco,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/config/name", channel_number),
        format: CommandType::S32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/config/icon", channel_number),
        format: CommandType::I32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/config/color", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/config/source", channel_number),
        format: CommandType::I32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/grp", channel_number),
        format: CommandType::Chgrp,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/grp/dca", channel_number),
        format: CommandType::P32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/grp/mute", channel_number),
        format: CommandType::P32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/preamp", channel_number),
        format: CommandType::Chpr,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/preamp/trim", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/preamp/invert", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/preamp/hpon", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/preamp/hpslope", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/preamp/hpf", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });

    // Delay
    commands.push(X32Command {
        command: format!("/ch/{:02}/delay", channel_number),
        format: CommandType::Chde,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/delay/on", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/delay/time", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });

    // Insert
    commands.push(X32Command {
        command: format!("/ch/{:02}/insert", channel_number),
        format: CommandType::Chin,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/insert/on", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/insert/pos", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/insert/sel", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });

    // Gate
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate", channel_number),
        format: CommandType::Chga,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/on", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/mode", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/thr", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/range", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/attack", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/hold", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/release", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/keysrc", channel_number),
        format: CommandType::I32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });

    // Gate Filter
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/filter", channel_number),
        format: CommandType::Chgf,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/filter/on", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/filter/type", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/gate/filter/f", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });

    // Dyn
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn", channel_number),
        format: CommandType::Chdy,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/on", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/mode", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/det", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/env", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/thr", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/ratio", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/knee", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/mgain", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/attack", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/hold", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/release", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/pos", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/keysrc", channel_number),
        format: CommandType::I32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });

    // Dyn Filter
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/filter", channel_number),
        format: CommandType::Chdf,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/filter/on", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/filter/type", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/dyn/filter/f", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });


    // EQ
    for i in 1..=4 {
        commands.push(X32Command {
            command: format!("/ch/{:02}/eq/{}", channel_number, i),
            format: CommandType::Cheq,
            flags: F_FND,
            value: CommandValue::Int(0),
            node: None,
        });
        commands.push(X32Command {
            command: format!("/ch/{:02}/eq/{}/type", channel_number, i),
            format: CommandType::E32,
            flags: F_XET,
            value: CommandValue::Int(0),
            node: None,
        });
        commands.push(X32Command {
            command: format!("/ch/{:02}/eq/{}/f", channel_number, i),
            format: CommandType::F32,
            flags: F_XET,
            value: CommandValue::Float(0.0),
            node: None,
        });
        commands.push(X32Command {
            command: format!("/ch/{:02}/eq/{}/g", channel_number, i),
            format: CommandType::F32,
            flags: F_XET,
            value: CommandValue::Float(0.0),
            node: None,
        });
        commands.push(X32Command {
            command: format!("/ch/{:02}/eq/{}/q", channel_number, i),
            format: CommandType::F32,
            flags: F_XET,
            value: CommandValue::Float(0.0),
            node: None,
        });
    }

    // Mix
    commands.push(X32Command {
        command: format!("/ch/{:02}/mix", channel_number),
        format: CommandType::Chmx,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/mix/on", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/mix/fader", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/mix/st", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/mix/pan", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/mix/mono", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/mix/mlevel", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });

    for i in 1..=16 {
        if i % 2 != 0 {
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}", channel_number, i),
                format: CommandType::Chmo,
                flags: F_FND,
                value: CommandValue::Int(0),
                node: None,
            });
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}/on", channel_number, i),
                format: CommandType::E32,
                flags: F_XET,
                value: CommandValue::Int(0),
                node: None,
            });
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}/level", channel_number, i),
                format: CommandType::F32,
                flags: F_XET,
                value: CommandValue::Float(0.0),
                node: None,
            });
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}/pan", channel_number, i),
                format: CommandType::F32,
                flags: F_XET,
                value: CommandValue::Float(0.0),
                node: None,
            });
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}/type", channel_number, i),
                format: CommandType::E32,
                flags: F_XET,
                value: CommandValue::Int(0),
                node: None,
            });
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}/panFollow", channel_number, i),
                format: CommandType::E32,
                flags: F_XET,
                value: CommandValue::Int(0),
                node: None,
            });
        } else {
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}", channel_number, i),
                format: CommandType::Chme,
                flags: F_FND,
                value: CommandValue::Int(0),
                node: None,
            });
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}/on", channel_number, i),
                format: CommandType::E32,
                flags: F_XET,
                value: CommandValue::Int(0),
                node: None,
            });
            commands.push(X32Command {
                command: format!("/ch/{:02}/mix/{:02}/level", channel_number, i),
                format: CommandType::F32,
                flags: F_XET,
                value: CommandValue::Float(0.0),
                node: None,
            });
        }
    }

    // Automix
    commands.push(X32Command {
        command: format!("/ch/{:02}/automix", channel_number),
        format: CommandType::Chamix,
        flags: F_FND,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/automix/group", channel_number),
        format: CommandType::E32,
        flags: F_XET,
        value: CommandValue::Int(0),
        node: None,
    });
    commands.push(X32Command {
        command: format!("/ch/{:02}/automix/weight", channel_number),
        format: CommandType::F32,
        flags: F_XET,
        value: CommandValue::Float(0.0),
        node: None,
    });

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_channel_commands() {
        let channel_number = 1;
        let commands = generate_channel_commands(channel_number);

        assert_eq!(commands.len(), 155);
        assert_eq!(commands[0].command, "/ch/01");
        assert_eq!(commands[17].command, "/ch/01/delay/time");
        assert_eq!(commands[21].command, "/ch/01/insert/sel");
        assert_eq!(commands[30].command, "/ch/01/gate/keysrc");
        assert_eq!(commands[34].command, "/ch/01/gate/filter/f");
        assert_eq!(commands[48].command, "/ch/01/dyn/keysrc");
        assert_eq!(commands[52].command, "/ch/01/dyn/filter/f");
        assert_eq!(commands[72].command, "/ch/01/eq/4/q");
        assert_eq!(commands[79].command, "/ch/01/mix/mlevel");
        assert_eq!(commands[154].command, "/ch/01/automix/weight");
    }
}
