use crate::common::{CommandType, CommandValue, X32Command};

const F_FND: i32 = 0x0008;
const F_XET: i32 = 0x0003;

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

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_channel_commands() {
        let channel_number = 1;
        let commands = generate_channel_commands(channel_number);

        assert_eq!(commands.len(), 65);
        assert_eq!(commands[0].command, "/ch/01");
        assert_eq!(commands[1].command, "/ch/01/config");
        assert_eq!(commands[2].command, "/ch/01/config/name");
        assert_eq!(commands[3].command, "/ch/01/config/icon");
        assert_eq!(commands[15].command, "/ch/01/gate");
        assert_eq!(commands[24].command, "/ch/01/dyn");
        assert_eq!(commands[42].command, "/ch/01/eq/1/q");
        assert_eq!(commands[64].command, "/ch/01/mix/mlevel");
    }
}
