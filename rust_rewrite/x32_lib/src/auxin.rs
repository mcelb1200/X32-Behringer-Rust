
//! Generates OSC command paths for X32 auxin channels.
//!
//! The X32 mixer has 8 auxin channels, each with a set of configurable parameters.
//! This module provides a function to generate the OSC command paths for a given
//! auxin channel, which can be used to control the mixer.

/// Generates a vector of OSC command paths for a specific auxin channel.
///
/// # Arguments
///
/// * `channel_num` - The auxin channel number (1-8).
///
/// # Returns
///
/// A `Result` containing a `Vec<String>` of OSC command paths on success,
/// or a `String` error message if the channel number is invalid.
///
pub fn get_auxin_commands(channel_num: u8) -> Result<Vec<String>, String> {
    if !(1..=8).contains(&channel_num) {
        return Err(format!(
            "Invalid auxin channel number: {}. Must be between 1 and 8.",
            channel_num
        ));
    }

    let mut commands = Vec::new();
    let base = format!("/auxin/{:02}", channel_num);

    commands.push("/auxin".to_string());
    commands.push(base.clone());

    // Config
    let config_base = format!("{}/config", base);
    commands.push(config_base.clone());
    commands.push(format!("{}/name", config_base));
    commands.push(format!("{}/icon", config_base));
    commands.push(format!("{}/color", config_base));
    commands.push(format!("{}/source", config_base));

    // Preamp
    let preamp_base = format!("{}/preamp", base);
    commands.push(preamp_base.clone());
    commands.push(format!("{}/trim", preamp_base));
    commands.push(format!("{}/invert", preamp_base));

    // EQ
    let eq_base = format!("{}/eq", base);
    commands.push(eq_base.clone());
    commands.push(format!("{}/on", eq_base));
    for i in 1..=4 {
        let band_base = format!("{}/{}", eq_base, i);
        commands.push(band_base.clone());
        commands.push(format!("{}/type", band_base));
        commands.push(format!("{}/f", band_base));
        commands.push(format!("{}/g", band_base));
        commands.push(format!("{}/q", band_base));
    }

    // Mix
    let mix_base = format!("{}/mix", base);
    commands.push(mix_base.clone());
    commands.push(format!("{}/on", mix_base));
    commands.push(format!("{}/fader", mix_base));
    commands.push(format!("{}/st", mix_base));
    commands.push(format!("{}/pan", mix_base));
    commands.push(format!("{}/mono", mix_base));
    commands.push(format!("{}/mlevel", mix_base));

    for i in 1..=16 {
        let send_base = format!("{}/{:02}", mix_base, i);
        commands.push(send_base.clone());
        commands.push(format!("{}/on", send_base));
        commands.push(format!("{}/level", send_base));
        // Odd numbered mixes also have pan, type, and panFollow parameters
        if i % 2 != 0 {
            commands.push(format!("{}/pan", send_base));
            commands.push(format!("{}/type", send_base));
            commands.push(format!("{}/panFollow", send_base));
        }
    }

    // Group
    let grp_base = format!("{}/grp", base);
    commands.push(grp_base.clone());
    commands.push(format!("{}/dca", grp_base));
    commands.push(format!("{}/mute", grp_base));

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_auxin_commands_valid_channel() {
        let commands = get_auxin_commands(1).unwrap();
        assert!(commands.contains(&"/auxin/01/config/name".to_string()));
        assert!(commands.contains(&"/auxin/01/mix/01/pan".to_string()));
        assert!(!commands.contains(&"/auxin/01/mix/02/pan".to_string()));
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
        // Expected count:
        // 1 for /auxin
        // 1 for /auxin/01
        // 5 for config
        // 3 for preamp
        // 2 + 4*4 = 18 for eq
        // 7 + 16*3 + 8*2 = 71 for mix
        // 3 for grp
        // Total = 1 + 1 + 5 + 3 + 18 + 71 + 3 = 102
        // Actually, mix is 7 + 16 (on/level) + 8 (pan/type/panFollow) = 7 + 32 + 24 = 63
        // Total = 1 + 1 + 5 + 3 + 18 + 63 + 3 = 94
        // The logic for mix sends is: 16 * 2 (on, level) + 8 * 3 (pan, type, panFollow) = 32 + 24 = 56
        // Mix base is 7. Total mix is 7 + 56 = 63.
        // Let's re-calculate:
        // 1 ("/auxin")
        // 1 ("/auxin/01")
        // 1 ("/auxin/01/config") + 4 ("/auxin/01/config/*") = 5
        // 1 ("/auxin/01/preamp") + 2 ("/auxin/01/preamp/*") = 3
        // 1 ("/auxin/01/eq") + 1 ("/auxin/01/eq/on") + 4 * (1 + 4) = 2 + 20 = 22
        // Wait, 4 bands, each with type, f, g, q = 4 * 4 = 16 commands. Plus eq/on = 17. Plus eq/1, eq/2, eq/3, eq/4 = 4. total = 21. plus /eq = 22.
        // Let's re-read the code.
        // eq_base (1) + eq/on (1) + 4 * (band_base (1) + 3) = 2 + 4*4=18
        // This is correct.
        //
        // mix_base (1) + on (1) + fader (1) + st (1) + pan (1) + mono (1) + mlevel (1) = 7
        // 16 sends. Each send has a base (1), on (1), level (1) = 3.  16*3 = 48.
        // 8 of those sends have pan(1), type(1), panFollow(1) = 3. 8*3 = 24.
        // Total mix = 7 + 48 + 24 = 79.
        //
        // grp_base (1) + dca (1) + mute (1) = 3
        //
        // Total = 2 (base) + 5 (config) + 3 (preamp) + 22 (eq) + 79 (mix) + 3 (grp) = 114
        assert_eq!(commands.len(), 114);
    }
}
