//! Provides functions for generating OSC commands to control X32/M32 input channels (1-32).
//!
//! This module covers a wide range of channel parameters, including configuration (name, color),
//! preamp settings, gate, dynamics, EQ, and mix bus sends.
//!
//! There are two primary ways to interact with channel commands:
//!
//! 1.  **Helper Functions:** Use `set_*` functions like `set_fader` or `set_name` to generate
//!     a tuple containing the OSC address string and a vector of `OscArg` arguments, ready
//!     to be packaged into an `OscMessage`. Use the corresponding address getter functions
//!     (e.g., `fader_level`) to get just the OSC address for querying a value.
//!
//! 2.  **Exhaustive Command List:** Use `get_channel_commands` to retrieve a complete, programmatically
//!     generated list of all available commands for a specific channel. This is useful for
//!     applications that need to discover or map all possible controls for a channel.
//!
//! # Example: Set and Verify a Channel's Fader Level
//!
//! ```no_run
//! use x32_lib::{create_socket, get_fader_level, OscMessage};
//! use x32_lib::command::channel;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let socket = create_socket("192.168.1.64", 1000)?;
//!
//!     // Set the channel fader level
//!     let (address, args) = channel::set_fader(1, 0.75);
//!     let msg = OscMessage::new(address, args);
//!     socket.send(&msg.to_bytes()?)?;
//!
//!     // Get the fader level to verify it was set
//!     let level = get_fader_level(&socket, &channel::fader_level(1))?;
//!     println!("Channel 1 fader level is now: {}", level);
//!
//!     Ok(())
//! }
//! ```
use super::{
    Command, CommandFlags, CommandType, OFF_ON, XAMXGRP, XCOLORS, XDYDET, XDYENV, XDYFTYP, XDYMODE,
    XDYPPOS, XDYRAT, XEQTY1, XGMODE, XHSLOP, XISEL, XMTYPE,
};
use lazy_static::lazy_static;
use osc_lib::OscArg;

// --- Address String Getters ---

/// Returns the OSC address for a channel's name.
pub fn name(channel_num: u8) -> String {
    format!("/ch/{:02}/config/name", channel_num)
}

/// Returns the OSC address for a channel's color.
pub fn color(channel_num: u8) -> String {
    format!("/ch/{:02}/config/color", channel_num)
}

/// Returns the OSC address for a channel's input source.
pub fn input_source(channel_num: u8) -> String {
    format!("/ch/{:02}/config/source", channel_num)
}

/// Returns the OSC address for a channel's delay on/off state.
pub fn delay_on(channel_num: u8) -> String {
    format!("/ch/{:02}/delay/on", channel_num)
}

/// Returns the OSC address for a channel's delay time.
pub fn delay_time(channel_num: u8) -> String {
    format!("/ch/{:02}/delay/time", channel_num)
}

/// Returns the OSC address for a channel's fader level.
pub fn fader_level(channel_num: u8) -> String {
    format!("/ch/{:02}/mix/fader", channel_num)
}

/// Returns the OSC address for a channel's on/off (mute) state.
pub fn on(channel_num: u8) -> String {
    format!("/ch/{:02}/mix/on", channel_num)
}

/// Returns the OSC address for a channel's insert on/off state.
pub fn insert_on(channel_num: u8) -> String {
    format!("/ch/{:02}/insert/on", channel_num)
}

/// Returns the OSC address for a channel's insert position (pre/post).
pub fn insert_pos(channel_num: u8) -> String {
    format!("/ch/{:02}/insert/pos", channel_num)
}

/// Returns the OSC address for a channel's insert selection.
pub fn insert_sel(channel_num: u8) -> String {
    format!("/ch/{:02}/insert/sel", channel_num)
}

/// Returns the OSC address for a channel's EQ band type.
pub fn eq_band_type(channel_num: u8, band: u8) -> String {
    format!("/ch/{:02}/eq/{}/type", channel_num, band)
}

// --- OSC Message Setters ---

/// Creates an OSC message to set the name of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `name` - The new name for the channel.
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_name(1, "Test");
/// assert_eq!(address, "/ch/01/config/name");
/// assert_eq!(args, vec![OscArg::String("Test".to_string())]);
/// ```
pub fn set_name(channel_num: u8, name: &str) -> (String, Vec<OscArg>) {
    (
        self::name(channel_num),
        vec![OscArg::String(name.to_string())],
    )
}

/// Creates an OSC message to set the color of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `color` - The new color for the channel (0-15). See `XCOLORS` for values.
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_color(1, 2);
/// assert_eq!(address, "/ch/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_color(channel_num: u8, color: i32) -> (String, Vec<OscArg>) {
    (self::color(channel_num), vec![OscArg::Int(color)])
}

/// Creates an OSC message to set the input source of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `source` - The new input source for the channel.
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_input_source(1, 2);
/// assert_eq!(address, "/ch/01/config/source");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_input_source(channel_num: u8, source: i32) -> (String, Vec<OscArg>) {
    (self::input_source(channel_num), vec![OscArg::Int(source)])
}

/// Creates an OSC message to set the delay on state of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `on` - The new delay on state for the channel (0 for Off, 1 for On).
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_delay_on(1, 1);
/// assert_eq!(address, "/ch/01/delay/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_delay_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (self::delay_on(channel_num), vec![OscArg::Int(on)])
}

/// Creates an OSC message to set the delay time of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `time` - The new delay time for the channel (0.0 to 1.0, representing 0-500ms).
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_delay_time(1, 0.5);
/// assert_eq!(address, "/ch/01/delay/time");
/// assert_eq!(args, vec![OscArg::Float(0.5)]);
/// ```
pub fn set_delay_time(channel_num: u8, time: f32) -> (String, Vec<OscArg>) {
    (self::delay_time(channel_num), vec![OscArg::Float(time)])
}

/// Creates an OSC message to set the fader level of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `level` - The new fader level (0.0 to 1.0).
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_fader(1, 0.75);
/// assert_eq!(address, "/ch/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(channel_num: u8, level: f32) -> (String, Vec<OscArg>) {
    (fader_level(channel_num), vec![OscArg::Float(level)])
}

/// Creates an OSC message to set the on state (mute) of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `on` - The new on state for the channel (0 for Muted, 1 for On).
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_on(1, 1);
/// assert_eq!(address, "/ch/01/mix/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (self::on(channel_num), vec![OscArg::Int(on)])
}

/// Creates an OSC message to set the insert on state of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `on` - The new insert on state for the channel (0 for Off, 1 for On).
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_on(1, 1);
/// assert_eq!(address, "/ch/01/insert/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_insert_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (self::insert_on(channel_num), vec![OscArg::Int(on)])
}

/// Creates an OSC message to set the insert position of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `pos` - The new insert position for the channel (0 for Pre, 1 for Post).
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_pos(1, 1);
/// assert_eq!(address, "/ch/01/insert/pos");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_insert_pos(channel_num: u8, pos: i32) -> (String, Vec<OscArg>) {
    (self::insert_pos(channel_num), vec![OscArg::Int(pos)])
}

/// Creates an OSC message to set the insert selection of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `sel` - The new insert selection for the channel (0-18). See `XISEL` for values.
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_sel(1, 2);
/// assert_eq!(address, "/ch/01/insert/sel");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_insert_sel(channel_num: u8, sel: i32) -> (String, Vec<OscArg>) {
    (self::insert_sel(channel_num), vec![OscArg::Int(sel)])
}

/// Creates an OSC message to set the eq band type of a channel.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
/// * `band` - The eq band (1-4).
/// * `eq_type` - The new eq type for the channel (0-5). See `XEQTY1` for values.
///
/// ```
/// use x32_lib::command::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_eq_band_type(1, 1, 2);
/// assert_eq!(address, "/ch/01/eq/1/type");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_eq_band_type(channel_num: u8, band: u8, eq_type: i32) -> (String, Vec<OscArg>) {
    (
        self::eq_band_type(channel_num, band),
        vec![OscArg::Int(eq_type)],
    )
}

/// Programmatically generates a vector of all available OSC commands for a single channel.
///
/// This function is useful for applications that need to dynamically discover and map all
/// possible controls for a channel, such as a full remote control surface or a scene parser.
///
/// # Arguments
///
/// * `channel_num` - The channel number (1-32).
///
/// # Panics
///
/// Panics if `channel_num` is not in the range 1-32.
pub fn get_channel_commands(channel_num: u8) -> Vec<Command<'static>> {
    assert!((1..=32).contains(&channel_num));
    let ch = format!("{:02}", channel_num);

    let mut commands = vec![];
    let flags = CommandFlags::GET.union(CommandFlags::SET);

    let mut add =
        |path: String, command_type: CommandType, nodes: Option<&'static [&'static str]>| {
            commands.push(Command {
                path: Box::leak(path.into_boxed_str()),
                command_type,
                flags,
                nodes,
            });
        };

    // Config
    add(format!("/ch/{}/config/name", ch), CommandType::String, None);
    add(format!("/ch/{}/config/icon", ch), CommandType::Int, None);
    add(
        format!("/ch/{}/config/color", ch),
        CommandType::Enum,
        Some(XCOLORS),
    );
    add(format!("/ch/{}/config/source", ch), CommandType::Int, None);

    // Group
    add(format!("/ch/{}/grp/dca", ch), CommandType::Bitmask, None);
    add(format!("/ch/{}/grp/mute", ch), CommandType::Bitmask, None);

    // Preamp
    add(format!("/ch/{}/preamp/trim", ch), CommandType::Float, None);
    add(
        format!("/ch/{}/preamp/invert", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(
        format!("/ch/{}/preamp/hpon", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(
        format!("/ch/{}/preamp/hpslope", ch),
        CommandType::Enum,
        Some(XHSLOP),
    );
    add(format!("/ch/{}/preamp/hpf", ch), CommandType::Float, None);

    // Delay
    add(
        format!("/ch/{}/delay/on", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(format!("/ch/{}/delay/time", ch), CommandType::Float, None);

    // Insert
    add(
        format!("/ch/{}/insert/on", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(
        format!("/ch/{}/insert/pos", ch),
        CommandType::Enum,
        Some(XDYPPOS),
    );
    add(
        format!("/ch/{}/insert/sel", ch),
        CommandType::Enum,
        Some(XISEL),
    );

    // Gate
    add(
        format!("/ch/{}/gate/on", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(
        format!("/ch/{}/gate/mode", ch),
        CommandType::Enum,
        Some(XGMODE),
    );
    add(format!("/ch/{}/gate/thr", ch), CommandType::Float, None);
    add(format!("/ch/{}/gate/range", ch), CommandType::Float, None);
    add(format!("/ch/{}/gate/attack", ch), CommandType::Float, None);
    add(format!("/ch/{}/gate/hold", ch), CommandType::Float, None);
    add(format!("/ch/{}/gate/release", ch), CommandType::Float, None);
    add(format!("/ch/{}/gate/keysrc", ch), CommandType::Int, None);
    add(
        format!("/ch/{}/gate/filter/on", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(
        format!("/ch/{}/gate/filter/type", ch),
        CommandType::Enum,
        Some(XDYFTYP),
    );
    add(
        format!("/ch/{}/gate/filter/f", ch),
        CommandType::Float,
        None,
    );

    // Dynamics
    add(
        format!("/ch/{}/dyn/on", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(
        format!("/ch/{}/dyn/mode", ch),
        CommandType::Enum,
        Some(XDYMODE),
    );
    add(
        format!("/ch/{}/dyn/det", ch),
        CommandType::Enum,
        Some(XDYDET),
    );
    add(
        format!("/ch/{}/dyn/env", ch),
        CommandType::Enum,
        Some(XDYENV),
    );
    add(format!("/ch/{}/dyn/thr", ch), CommandType::Float, None);
    add(
        format!("/ch/{}/dyn/ratio", ch),
        CommandType::Enum,
        Some(XDYRAT),
    );
    add(format!("/ch/{}/dyn/knee", ch), CommandType::Float, None);
    add(format!("/ch/{}/dyn/mgain", ch), CommandType::Float, None);
    add(format!("/ch/{}/dyn/attack", ch), CommandType::Float, None);
    add(format!("/ch/{}/dyn/hold", ch), CommandType::Float, None);
    add(format!("/ch/{}/dyn/release", ch), CommandType::Float, None);
    add(
        format!("/ch/{}/dyn/pos", ch),
        CommandType::Enum,
        Some(XDYPPOS),
    );
    add(format!("/ch/{}/dyn/keysrc", ch), CommandType::Int, None);
    add(format!("/ch/{}/dyn/mix", ch), CommandType::Float, None);
    add(
        format!("/ch/{}/dyn/auto", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(
        format!("/ch/{}/dyn/filter/on", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(
        format!("/ch/{}/dyn/filter/type", ch),
        CommandType::Enum,
        Some(XDYFTYP),
    );
    add(format!("/ch/{}/dyn/filter/f", ch), CommandType::Float, None);

    // EQ
    add(format!("/ch/{}/eq/on", ch), CommandType::Enum, Some(OFF_ON));
    for i in 1..=4 {
        add(
            format!("/ch/{}/eq/{}/type", ch, i),
            CommandType::Enum,
            Some(XEQTY1),
        );
        add(format!("/ch/{}/eq/{}/f", ch, i), CommandType::Float, None);
        add(format!("/ch/{}/eq/{}/g", ch, i), CommandType::Float, None);
        add(format!("/ch/{}/eq/{}/q", ch, i), CommandType::Float, None);
    }

    // Mix
    add(
        format!("/ch/{}/mix/on", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(format!("/ch/{}/mix/fader", ch), CommandType::Float, None);
    add(
        format!("/ch/{}/mix/st", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(format!("/ch/{}/mix/pan", ch), CommandType::Float, None);
    add(
        format!("/ch/{}/mix/mono", ch),
        CommandType::Enum,
        Some(OFF_ON),
    );
    add(format!("/ch/{}/mix/mlevel", ch), CommandType::Float, None);

    for i in 1..=16 {
        let mix = format!("{:02}", i);
        add(
            format!("/ch/{}/mix/{}/on", ch, mix),
            CommandType::Enum,
            Some(OFF_ON),
        );
        add(
            format!("/ch/{}/mix/{}/level", ch, mix),
            CommandType::Float,
            None,
        );

        if i % 2 != 0 {
            add(
                format!("/ch/{}/mix/{}/pan", ch, mix),
                CommandType::Float,
                None,
            );
            add(
                format!("/ch/{}/mix/{}/type", ch, mix),
                CommandType::Enum,
                Some(XMTYPE),
            );
            add(
                format!("/ch/{}/mix/{}/panFollow", ch, mix),
                CommandType::Enum,
                Some(OFF_ON),
            );
        }
    }

    // Automix
    add(
        format!("/ch/{}/automix/group", ch),
        CommandType::Enum,
        Some(XAMXGRP),
    );
    add(
        format!("/ch/{}/automix/weight", ch),
        CommandType::Float,
        None,
    );

    commands
}

lazy_static! {
    /// A pre-computed, static vector containing the command lists for all 32 channels.
    ///
    /// This is initialized at compile time and avoids the need to repeatedly generate the
    /// command list for each channel at runtime. Access the commands for a specific
    /// channel using `XCHANNEL_COMMANDS[channel_num - 1]`.
    pub static ref XCHANNEL_COMMANDS: Vec<Vec<Command<'static>>> =
        (1..=32).map(get_channel_commands).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_channel_commands_command_count() {
        let commands = get_channel_commands(1);
        assert_eq!(commands.len(), 126);
    }

    #[test]
    fn test_get_channel_commands_valid_channel() {
        let commands = get_channel_commands(1);
        assert_eq!(commands[0].path, "/ch/01/config/name");
        assert_eq!(commands.last().unwrap().path, "/ch/01/automix/weight");

        let commands_32 = get_channel_commands(32);
        assert_eq!(commands_32[0].path, "/ch/32/config/name");
        assert_eq!(commands_32.last().unwrap().path, "/ch/32/automix/weight");
    }

    #[test]
    #[should_panic]
    fn test_get_channel_commands_invalid_channel() {
        get_channel_commands(0);
    }

    #[test]
    fn test_xchannel_commands_array() {
        assert_eq!(XCHANNEL_COMMANDS.len(), 32);
        for i in 0..32 {
            assert_eq!(XCHANNEL_COMMANDS[i].len(), 126);
        }
    }
}
