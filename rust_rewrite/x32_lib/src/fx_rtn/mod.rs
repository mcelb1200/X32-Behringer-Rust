
//! Defines the OSC command structures for the X32's FX return channels.
//!
//! This module translates the static C arrays from `X32Fxrtn.h` into idiomatic
//! and type-safe Rust data structures. It provides a static representation of
//! all available OSC commands for the 8 FX return channels on the Behringer X32.
//!
//! ## Example
//!
//! ```
//! use x32_lib::fx_rtn::data::XFXRTN01;
//!
//! // Get the first command for the first FX return channel
//! let command = &XFXRTN01[0];
//!
//! // Print the OSC command path
//! println!("Command: {}", command.command);
//! ```

/// Represents the format of an OSC command's arguments.
#[derive(Debug)]
pub enum CommandFormat {
    /// A 32-bit integer.
    Int,
    /// A 32-bit float.
    Float,
    /// A string.
    String,
    /// A list of possible string values (enum).
    StringList(&'static [&'static str]),
}

/// Represents the value of an OSC command's argument.
#[derive(Debug)]
pub enum CommandValue {
    /// A 32-bit integer value.
    Int(i32),
    /// A 32-bit float value.
    Float(f32),
    /// A string value.
    String(&'static str),
    /// No value.
    None,
}

bitflags::bitflags! {
    /// Flags that describe the behavior of an OSC command.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CommandFlags: u32 {
        /// The command can be used to get a value.
        const F_GET = 0x0001;
        /// The command can be used to set a value.
        const F_SET = 0x0002;
        /// The command can be used to both get and set a value.
        const F_XET = Self::F_GET.bits() | Self::F_SET.bits();
        /// Do not propagate the command to other clients.
        const F_NPR = 0x0004;
        /// The first of a series or node data header.
        const F_FND = 0x0008;
    }
}

/// Represents a single OSC command for the Behringer X32.
#[derive(Debug)]
pub struct X32Command {
    /// The OSC command path (e.g., "/fxrtn/01/mix/fader").
    pub command: &'static str,
    /// The format of the command's arguments.
    pub format: CommandFormat,
    /// Flags that describe the command's behavior.
    pub flags: CommandFlags,
    /// The value of the command's argument, if any.
    pub value: CommandValue,
}

pub mod data;

#[cfg(test)]
mod tests;
