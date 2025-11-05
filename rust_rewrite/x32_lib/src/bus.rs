#![allow(dead_code)]

use crate::common::{CommandFlags, CommandFormat, CommandValue, X32Command};

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
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}", bus_prefix),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/config", bus_prefix),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/config/name", bus_prefix),
            format: CommandFormat::String,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/config/icon", bus_prefix),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/config/color", bus_prefix),
            format: CommandFormat::StringList(&XCOLORS),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn", bus_prefix),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/on", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/mode", bus_prefix),
            format: CommandFormat::StringList(&XDYMODE),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/det", bus_prefix),
            format: CommandFormat::StringList(&XDYDET),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/env", bus_prefix),
            format: CommandFormat::StringList(&XDYENV),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/thr", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/ratio", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/knee", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/mgain", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/attack", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/hold", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/release", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/pos", bus_prefix),
            format: CommandFormat::StringList(&XDYPPOS),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/keysrc", bus_prefix),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/mix", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/auto", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/filter", bus_prefix),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/filter/on", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/filter/type", bus_prefix),
            format: CommandFormat::StringList(&XDYFTYP),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/dyn/filter/f", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/insert", bus_prefix),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/insert/on", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/insert/pos", bus_prefix),
            format: CommandFormat::StringList(&XDYPPOS),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/insert/sel", bus_prefix),
            format: CommandFormat::StringList(&XISEL),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/eq", bus_prefix),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::Int(1),
        },
        X32Command {
            command: format!("{}/eq/on", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
    ];

    for i in 1..=6 {
        commands.extend(vec![
            X32Command {
                command: format!("{}/eq/{}", bus_prefix, i),
                format: CommandFormat::StringList(&[]),
                flags: CommandFlags::F_FND,
                value: CommandValue::None,
            },
            X32Command {
                command: format!("{}/eq/{}/type", bus_prefix, i),
                format: CommandFormat::StringList(&XEQTY1),
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            },
            X32Command {
                command: format!("{}/eq/{}/f", bus_prefix, i),
                format: CommandFormat::Float,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            },
            X32Command {
                command: format!("{}/eq/{}/g", bus_prefix, i),
                format: CommandFormat::Float,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            },
            X32Command {
                command: format!("{}/eq/{}/q", bus_prefix, i),
                format: CommandFormat::Float,
                flags: CommandFlags::F_XET,
                value: CommandValue::None,
            },
        ]);
    }

    commands.extend(vec![
        X32Command {
            command: format!("{}/mix", bus_prefix),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/mix/on", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/mix/fader", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/mix/st", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/mix/pan", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/mix/mono", bus_prefix),
            format: CommandFormat::StringList(&OFF_ON),
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/mix/mlevel", bus_prefix),
            format: CommandFormat::Float,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
    ]);

    for i in 1..=6 {
        if i % 2 != 0 {
            commands.extend(vec![
                X32Command {
                    command: format!("{}/mix/{:02}", bus_prefix, i),
                    format: CommandFormat::StringList(&[]),
                    flags: CommandFlags::F_FND,
                    value: CommandValue::None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/on", bus_prefix, i),
                    format: CommandFormat::StringList(&OFF_ON),
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/level", bus_prefix, i),
                    format: CommandFormat::Float,
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/pan", bus_prefix, i),
                    format: CommandFormat::Float,
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/type", bus_prefix, i),
                    format: CommandFormat::StringList(&OFF_ON),
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/panFollow", bus_prefix, i),
                    format: CommandFormat::Int,
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
            ]);
        } else {
            commands.extend(vec![
                X32Command {
                    command: format!("{}/mix/{:02}", bus_prefix, i),
                    format: CommandFormat::StringList(&[]),
                    flags: CommandFlags::F_FND,
                    value: CommandValue::None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/on", bus_prefix, i),
                    format: CommandFormat::StringList(&OFF_ON),
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
                X32Command {
                    command: format!("{}/mix/{:02}/level", bus_prefix, i),
                    format: CommandFormat::Float,
                    flags: CommandFlags::F_XET,
                    value: CommandValue::None,
                },
            ]);
        }
    }

    commands.extend(vec![
        X32Command {
            command: format!("{}/grp", bus_prefix),
            format: CommandFormat::StringList(&[]),
            flags: CommandFlags::F_FND,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/grp/dca", bus_prefix),
            format: CommandFormat::Int,
            flags: CommandFlags::F_XET,
            value: CommandValue::None,
        },
        X32Command {
            command: format!("{}/grp/mute", bus_prefix),
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
    fn test_get_bus_commands() {
        let bus_commands = get_bus_commands(1);

        assert_eq!(bus_commands.len(), 99);
    }
}
