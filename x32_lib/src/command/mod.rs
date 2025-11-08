//! This module organizes and provides the data structures for representing X32 commands.
//!
//! It serves as the central hub for all command-related modules, each corresponding
//! to a specific section of the X32 console (e.g., `channel`, `bus`, `fx`). The
//! primary goal is to provide a type-safe and idiomatic Rust interface for generating
//! OSC messages that can be sent to the mixer.

use bitflags::bitflags;

pub mod auxin;
pub mod bus;
pub mod cfg_main;
pub mod channel;
pub mod dca;
pub mod fx;
pub mod fxrtn;
pub mod headamp;
pub mod libs;
pub mod misc;
pub mod mtx;
pub mod output;
pub mod prefstat;
pub mod show;

/// An enum representing the different types of OSC address arguments.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    /// No argument.
    Nil,
    /// A 32-bit integer.
    Int,
    /// A 32-bit float.
    Float,
    /// A string.
    String,
    /// A blob of binary data.
    Blob,
    /// An enumerated type, represented as a list of strings.
    Enum,
    /// A bitmask, represented as a list of strings.
    Bitmask,
}

/// A struct that defines a command that can be sent to an X32 console.
#[derive(Debug, PartialEq, Clone)]
pub struct Command<'a> {
    /// The OSC address path for the command (e.g., "/ch/01/mix/fader").
    pub path: &'a str,
    /// The type of data associated with the command.
    pub command_type: CommandType,
    /// Flags that specify the behavior of the command (e.g., GET, SET).
    pub flags: CommandFlags,
    /// An optional list of string representations for enum or bitmask values.
    pub nodes: Option<&'static [&'static str]>,
}

bitflags! {
    /// Flags that specify the behavior of an X32 command.
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct CommandFlags: u8 {
        /// The command can be used to get a parameter value from the mixer.
        const GET = 0x01;
        /// The command can be used to set a parameter value on the mixer.
        const SET = 0x02;
        /// The command is part of a node in the OSC address space, meaning it has sub-commands.
        const NODE = 0x08;
    }
}

/// String representations for boolean "OFF" and "ON" states.
pub const OFF_ON: &[&str] = &["OFF", "ON"];
/// String representations for scribble strip colors.
pub const XCOLORS: &[&str] = &[
    "OFF", "RED", "GREEN", "YELLOW", "BLUE", "MAGENTA", "CYAN", "WHITE", "OFFi", "REDi", "GREENi",
    "YELLOWi", "BLUEi", "MAGENTAi", "CYANi", "WHITEi",
];
/// String representations for high-pass and low-pass filter slopes.
pub const XHSLOP: &[&str] = &["12", "24"];
/// String representations for dynamics processing position (pre/post).
pub const XDYPPOS: &[&str] = &["PRE", "POST"];
/// String representations for insert selections.
pub const XISEL: &[&str] = &[
    "OFF", "Ins", "FX1L", "FX1R", "FX2L", "FX2R", "FX3L", "FX3R", "FX4L", "FX4R", "FX5L", "FX5R",
    "FX6L", "FX6R", "FX7L", "FX7R", "FX8L", "FX8R", "Aux1", "Aux2", "Aux3", "Aux4", "Aux5", "Aux6",
];
/// String representations for gate modes.
pub const XGMODE: &[&str] = &["GATE", "EXP2", "EXP3", "EXP4", "DUCK"];
/// String representations for dynamics filter types.
pub const XDYFTYP: &[&str] = &["LC6", "LC12", "HC6", "HC12", "BP", "NOTCH"];
/// String representations for dynamics modes (compressor/limiter).
pub const XDYMODE: &[&str] = &["COMP", "LIMIT"];
/// String representations for dynamics detector types.
pub const XDYDET: &[&str] = &["PEAK", "RMS"];
/// String representations for dynamics envelope types.
pub const XDYENV: &[&str] = &["LIN", "LOG"];
/// String representations for dynamics ratio settings.
pub const XDYRAT: &[&str] = &[
    "1.1", "1.3", "1.5", "2.0", "2.5", "3.0", "4.0", "5.0", "7.0", "10.0", "20.0", "100.0",
];
/// String representations for EQ types.
pub const XEQTY1: &[&str] = &["LCut", "LShv", "PEQ", "VEQ", "HCut", "HShv"];
/// String representations for bus mix types.
pub const XMTYPE: &[&str] = &["SUBGROUP", "PREFDR", "POSTFDR"];
/// String representations for automix group assignments.
pub const XAMXGRP: &[&str] = &["OFF", "X", "Y"];
