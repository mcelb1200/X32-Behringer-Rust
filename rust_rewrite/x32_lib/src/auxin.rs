use crate::common::{CommandFlags, CommandFormat, CommandValue, X32Command};

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
        command: "/auxin".to_string(),
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

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

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
