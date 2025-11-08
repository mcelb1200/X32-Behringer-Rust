//! This module provides the data structures for representing X32 commands.
//! It is a Rust rewrite of the original C implementation by Patrick-Gilles Maillot.

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

/// Enum representing the different types of OSC address arguments.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    Nil,
    Int,
    Float,
    String,
    Blob,
    Enum,
    Bitmask,
}

/// A struct that defines a command that can be sent to an X32 console.
#[derive(Debug, PartialEq, Clone)]
pub struct Command<'a> {
    /// The OSC address path for the command.
    pub path: &'a str,
    /// The type of data associated with the command.
    pub command_type: CommandType,
    /// Flags that specify the behavior of the command.
    pub flags: CommandFlags,
    /// An optional list of string representations for enum or bitmask values.
    pub nodes: Option<&'static [&'static str]>,
}

bitflags! {
    /// Flags that specify the behavior of an X32 command.
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct CommandFlags: u8 {
        /// The command can be used to get a parameter value.
        const GET = 0x01;
        /// The command can be used to set a parameter value.
        const SET = 0x02;
        /// The command is part of a node in the OSC address space.
        const NODE = 0x08;
    }
}

pub const OFF_ON: &[&str] = &["OFF", "ON"];
pub const XCOLORS: &[&str] = &[
    "OFF", "RED", "GREEN", "YELLOW", "BLUE", "MAGENTA", "CYAN", "WHITE", "OFFi", "REDi", "GREENi",
    "YELLOWi", "BLUEi", "MAGENTAi", "CYANi", "WHITEi",
];
pub const XHSLOP: &[&str] = &["12", "24"];
pub const XDYPPOS: &[&str] = &["PRE", "POST"];
pub const XISEL: &[&str] = &[
    "OFF", "Ins", "FX1L", "FX1R", "FX2L", "FX2R", "FX3L", "FX3R", "FX4L", "FX4R", "FX5L", "FX5R",
    "FX6L", "FX6R", "FX7L", "FX7R", "FX8L", "FX8R", "Aux1", "Aux2", "Aux3", "Aux4", "Aux5", "Aux6",
];
pub const XGMODE: &[&str] = &["GATE", "EXP2", "EXP3", "EXP4", "DUCK"];
pub const XDYFTYP: &[&str] = &["LC6", "LC12", "HC6", "HC12", "BP", "NOTCH"];
pub const XDYMODE: &[&str] = &["COMP", "LIMIT"];
pub const XDYDET: &[&str] = &["PEAK", "RMS"];
pub const XDYENV: &[&str] = &["LIN", "LOG"];
pub const XDYRAT: &[&str] = &[
    "1.1", "1.3", "1.5", "2.0", "2.5", "3.0", "4.0", "5.0", "7.0", "10.0", "20.0", "100.0",
];
pub const XEQTY1: &[&str] = &["LCut", "LShv", "PEQ", "VEQ", "HCut", "HShv"];
pub const XMTYPE: &[&str] = &["SUBGROUP", "PREFDR", "POSTFDR"];
pub const XAMXGRP: &[&str] = &["OFF", "X", "Y"];
