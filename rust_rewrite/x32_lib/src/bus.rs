#![allow(dead_code)]

// Command flags
pub const F_GET: i32 = 0x0001;
pub const F_SET: i32 = 0x0002;
pub const F_XET: i32 = F_GET | F_SET;
pub const F_NPR: i32 = 0x0004;
pub const F_FND: i32 = 0x0008;

// Enum for command types
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    Bsco,
    S32,
    I32,
    E32,
    Chdy,
    F32,
    Chdf,
    Chin,
    Offon,
    Cheq,
    Chmx,
    Chmo,
    Chme,
    Chgrp,
    P32,
}

// Enum for command format
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandFormat {
    Type(CommandType),
    Str(&'static str),
}

// Enum for command value
#[derive(Debug, PartialEq, Clone)]
pub enum CommandValue {
    Int(i32),
    Float(f32),
    String(String),
    None,
}

// The Command structure describes ALL X32 commands
#[derive(Debug, PartialEq, Clone)]
pub struct X32Command {
    pub command: String,
    pub format: CommandFormat,
    pub flags: i32,
    pub value: CommandValue,
    pub node: Option<&'static [&'static str]>,
}

// Node string arrays
pub const OFF_ON: &[&str] = &[" OFF", " ON", ""];
pub const XCOLORS: &[&str] = &[
    " OFF", " RD", " GN", " YE", " BL", " MG", " CY", " WH", " OFFi", " RDi", " GNi", " YEi",
    " BLi", " MGi", " CYi", " WHi", "",
];
pub const XDYMODE: &[&str] = &[" COMP", " EXP", ""];
pub const XDYDET: &[&str] = &[" PEAK", " RMS", ""];
pub const XDYENV: &[&str] = &[" LIN", " LOG", ""];
pub const XDYPPOS: &[&str] = &[" PRE", " POST", ""];
pub const XDYFTYP: &[&str] = &[
    " LC6", " LC12", " HC6", " HC12", " 1.0", " 2.0", " 3.0", " 5.0", " 10.0", "",
];
pub const XISEL: &[&str] = &[
    " OFF", " FX1L", " FX1R", " FX2L", " FX2R", " FX3L", " FX3R", " FX4L", " FX4R", " FX5L",
    " FX5R", " FX6L", " FX6R", " FX7L", " FX7R", " FX8L", " FX8R", " AUX1", " AUX2", " AUX3",
    " AUX4", " AUX5", " AUX6", "",
];
pub const XEQTY1: &[&str] = &[" LCut", " LShv", " PEQ", " VEQ", " HShv", " HCut", ""];


pub fn get_bus_commands(bus_number: i32) -> Vec<X32Command> {
    let bus_prefix = format!("/bus/{:02}", bus_number);
    let mut commands = vec![
        X32Command {
            command: "/bus".to_string(),
            format: CommandFormat::Type(CommandType::Bsco),
            flags: F_FND,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}", bus_prefix),
            format: CommandFormat::Type(CommandType::Bsco),
            flags: F_FND,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/config", bus_prefix),
            format: CommandFormat::Type(CommandType::Bsco),
            flags: F_FND,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/config/name", bus_prefix),
            format: CommandFormat::Type(CommandType::S32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/config/icon", bus_prefix),
            format: CommandFormat::Type(CommandType::I32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/config/color", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&XCOLORS),
        },
        X32Command {
            command: format!("{}/dyn", bus_prefix),
            format: CommandFormat::Type(CommandType::Chdy),
            flags: F_FND,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/on", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/dyn/mode", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&XDYMODE),
        },
        X32Command {
            command: format!("{}/dyn/det", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&XDYDET),
        },
        X32Command {
            command: format!("{}/dyn/env", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&XDYENV),
        },
        X32Command {
            command: format!("{}/dyn/thr", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/ratio", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/dyn/knee", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/mgain", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/attack", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/hold", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/release", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/pos", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&XDYPPOS),
        },
        X32Command {
            command: format!("{}/dyn/keysrc", bus_prefix),
            format: CommandFormat::Type(CommandType::I32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/mix", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/dyn/auto", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/dyn/filter", bus_prefix),
            format: CommandFormat::Type(CommandType::Chdf),
            flags: F_FND,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/dyn/filter/on", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/dyn/filter/type", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&XDYFTYP),
        },
        X32Command {
            command: format!("{}/dyn/filter/f", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/insert", bus_prefix),
            format: CommandFormat::Type(CommandType::Chin),
            flags: F_FND,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/insert/on", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/insert/pos", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&XDYPPOS),
        },
        X32Command {
            command: format!("{}/insert/sel", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&XISEL),
        },
        X32Command {
            command: format!("{}/eq", bus_prefix),
            format: CommandFormat::Type(CommandType::Offon),
            flags: F_FND,
            value: CommandValue::Int(1),
            node: None,
        },
        X32Command {
            command: format!("{}/eq/on", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
    ];

    for i in 1..=6 {
        commands.extend(vec![
            X32Command {
                command: format!("{}/eq/{}", bus_prefix, i),
                format: CommandFormat::Type(CommandType::Cheq),
                flags: F_FND,
                value: CommandValue::None,
                node: None,
            },
            X32Command {
                command: format!("{}/eq/{}/type", bus_prefix, i),
                format: CommandFormat::Type(CommandType::E32),
                flags: F_XET,
                value: CommandValue::None,
                node: Some(&XEQTY1),
            },
            X32Command {
                command: format!("{}/eq/{}/f", bus_prefix, i),
                format: CommandFormat::Type(CommandType::F32),
                flags: F_XET,
                value: CommandValue::None,
                node: None,
            },
            X32Command {
                command: format!("{}/eq/{}/g", bus_prefix, i),
                format: CommandFormat::Type(CommandType::F32),
                flags: F_XET,
                value: CommandValue::None,
                node: None,
            },
            X32Command {
                command: format!("{}/eq/{}/q", bus_prefix, i),
                format: CommandFormat::Type(CommandType::F32),
                flags: F_XET,
                value: CommandValue::None,
                node: None,
            },
        ]);
    }

    commands.extend(vec![
        X32Command {
            command: format!("{}/mix", bus_prefix),
            format: CommandFormat::Type(CommandType::Chmx),
            flags: F_FND,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/mix/on", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/mix/fader", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/mix/st", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/mix/pan", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/mix/mono", bus_prefix),
            format: CommandFormat::Type(CommandType::E32),
            flags: F_XET,
            value: CommandValue::None,
            node: Some(&OFF_ON),
        },
        X32Command {
            command: format!("{}/mix/mlevel", bus_prefix),
            format: CommandFormat::Type(CommandType::F32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
    ]);

    for i in 1..=6 {
        if i % 2 != 0 {
            commands.extend(vec![
                X32Command {
                    command: format!("{}/mix/{:02}", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::Chmo),
                    flags: F_FND,
                    value: CommandValue::None,
                    node: None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/on", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::E32),
                    flags: F_XET,
                    value: CommandValue::None,
                    node: Some(&OFF_ON),
                },
                X32Command {
                    command: format!("{}/mix/{:02}/level", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::F32),
                    flags: F_XET,
                    value: CommandValue::None,
                    node: None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/pan", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::F32),
                    flags: F_XET,
                    value: CommandValue::None,
                    node: None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/type", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::E32),
                    flags: F_XET,
                    value: CommandValue::None,
                    node: Some(&OFF_ON),
                },
                X32Command {
                    command: format!("{}/mix/{:02}/panFollow", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::E32),
                    flags: F_XET,
                    value: CommandValue::None,
                    node: None,
                },
            ]);
        } else {
            commands.extend(vec![
                X32Command {
                    command: format!("{}/mix/{:02}", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::Chme),
                    flags: F_FND,
                    value: CommandValue::None,
                    node: None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/on", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::E32),
                    flags: F_XET,
                    value: CommandValue::None,
                    node: Some(&OFF_ON),
                },
                X32Command {
                    command: format!("{}/mix/{:02}/level", bus_prefix, i),
                    format: CommandFormat::Type(CommandType::F32),
                    flags: F_XET,
                    value: CommandValue::None,
                    node: None,
                },
            ]);
        }
    }

    commands.extend(vec![
        X32Command {
            command: format!("{}/grp", bus_prefix),
            format: CommandFormat::Type(CommandType::Chgrp),
            flags: F_FND,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/grp/dca", bus_prefix),
            format: CommandFormat::Type(CommandType::P32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
        X32Command {
            command: format!("{}/grp/mute", bus_prefix),
            format: CommandFormat::Type(CommandType::P32),
            flags: F_XET,
            value: CommandValue::None,
            node: None,
        },
    ]);

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bus_commands() {
        let bus_commands = get_bus_commands(1);

        assert_eq!(bus_commands.len(), 99);
    }
}
