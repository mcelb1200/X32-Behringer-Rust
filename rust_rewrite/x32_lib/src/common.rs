//! # Common Module
//!
//! This module contains common enumerations and structs used across the library.
//! These enums provide a type-safe way to interact with the X32/M32 console's parameters.

use bitflags::bitflags;

#[derive(Debug, Clone)]
pub enum CommandFormat {
    Int,
    Float,
    String,
    StringList(&'static [&'static str]),
}

#[derive(Debug, Clone)]
pub enum CommandValue {
    Int(i32),
    Float(f32),
    String(String),
    None,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CommandFlags: u32 {
        const F_GET = 0x0001;
        const F_SET = 0x0002;
        const F_XET = Self::F_GET.bits() | Self::F_SET.bits();
        const F_NPR = 0x0004;
        const F_FND = 0x0008;
    }
}

#[derive(Debug, Clone)]
pub struct X32Command {
    pub command: String,
    pub format: CommandFormat,
    pub flags: CommandFlags,
    pub value: CommandValue,
}

/// Represents the `On` or `Off` state of a parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum On {
    Off = 0,
    On = 1,
}

impl On {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(On::Off),
            1 => Some(On::On),
            _ => None,
        }
    }
}

/// Represents the color of a channel or DCA scribble strip.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Color {
    Off = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    OffInverted,
    RedInverted,
    GreenInverted,
    YellowInverted,
    BlueInverted,
    MagentaInverted,
    CyanInverted,
    WhiteInverted,
}

impl Color {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Color::Off),
            1 => Some(Color::Red),
            2 => Some(Color::Green),
            3 => Some(Color::Yellow),
            4 => Some(Color::Blue),
            5 => Some(Color::Magenta),
            6 => Some(Color::Cyan),
            7 => Some(Color::White),
            8 => Some(Color::OffInverted),
            9 => Some(Color::RedInverted),
            10 => Some(Color::GreenInverted),
            11 => Some(Color::YellowInverted),
            12 => Some(Color::BlueInverted),
            13 => Some(Color::MagentaInverted),
            14 => Some(Color::CyanInverted),
            15 => Some(Color::WhiteInverted),
            _ => None,
        }
    }
}

/// Represents the type of an EQ band.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum EqType {
    Lcut = 0,
    LShv,
    Peq,
    Veq,
    HShv,
    Hcut,
}

impl EqType {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(EqType::Lcut),
            1 => Some(EqType::LShv),
            2 => Some(EqType::Peq),
            3 => Some(EqType::Veq),
            4 => Some(EqType::HShv),
            5 => Some(EqType::Hcut),
            _ => None,
        }
    }
}

/// Represents the position of an insert (`Pre` or `Post` fader).
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum InsertPosition {
    Pre = 0,
    Post = 1,
}

impl InsertPosition {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(InsertPosition::Pre),
            1 => Some(InsertPosition::Post),
            _ => None,
        }
    }
}

/// Represents the selection for an insert.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum InsertSelection {
    Off = 0,
    Fx1L,
    Fx1R,
    Fx2L,
    Fx2R,
    Fx3L,
    Fx3R,
    Fx4L,
    Fx4R,
    Fx5L,
    Fx5R,
    Fx6L,
    Fx6R,
    Fx7L,
    Fx7R,
    Fx8L,
    Fx8R,
    Aux1,
    Aux2,
    Aux3,
    Aux4,
    Aux5,
    Aux6,
}

impl InsertSelection {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(InsertSelection::Off),
            1 => Some(InsertSelection::Fx1L),
            2 => Some(InsertSelection::Fx1R),
            3 => Some(InsertSelection::Fx2L),
            4 => Some(InsertSelection::Fx2R),
            5 => Some(InsertSelection::Fx3L),
            6 => Some(InsertSelection::Fx3R),
            7 => Some(InsertSelection::Fx4L),
            8 => Some(InsertSelection::Fx4R),
            9 => Some(InsertSelection::Fx5L),
            10 => Some(InsertSelection::Fx5R),
            11 => Some(InsertSelection::Fx6L),
            12 => Some(InsertSelection::Fx6R),
            13 => Some(InsertSelection::Fx7L),
            14 => Some(InsertSelection::Fx7R),
            15 => Some(InsertSelection::Fx8L),
            16 => Some(InsertSelection::Fx8R),
            17 => Some(InsertSelection::Aux1),
            18 => Some(InsertSelection::Aux2),
            19 => Some(InsertSelection::Aux3),
            20 => Some(InsertSelection::Aux4),
            21 => Some(InsertSelection::Aux5),
            22 => Some(InsertSelection::Aux6),
            _ => None,
        }
    }
}

/// Represents the source for an FX processor.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum FxSource {
    Off = 0,
    MixBus(u8),
    Bus(u8),
    Mtx(u8),
    Main(u8),
    Group(u8),
}

impl FxSource {
    pub fn to_id(&self) -> u8 {
        match self {
            FxSource::Off => 0,
            FxSource::MixBus(bus) => *bus,
            FxSource::Bus(bus) => *bus + 16,
            FxSource::Mtx(mtx) => *mtx + 32,
            FxSource::Main(main) => *main + 40,
            FxSource::Group(grp) => *grp + 48,
        }
    }
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(FxSource::Off),
            1..=16 => Some(FxSource::MixBus(id)),
            17..=32 => Some(FxSource::Bus(id - 16)),
            33..=38 => Some(FxSource::Mtx(id - 32)),
            39..=40 => Some(FxSource::Main(id - 40)),
            41..=48 => Some(FxSource::Group(id - 48)),
            _ => None,
        }
    }
}

pub static XCOLORS: [&'static str; 16] = [
    "OFF", "RD", "GN", "YE", "BL", "MG", "CY", "WH",
    "OFFi", "RDi", "GNi", "YEi", "BLi", "MGi", "CYi", "WHi"];

pub static OFF_ON: [&'static str; 2] = ["OFF", "ON"];
pub static XDYMODE: [&'static str; 2] = ["COMP", "EXP"];
pub static XDYDET: [&'static str; 2] = ["PEAK", "RMS"];
pub static XDYENV: [&'static str; 2] = ["LIN", "LOG"];
pub static XDYPPOS: [&'static str; 2] = ["PRE", "POST"];
pub static XDYFTYP: [&'static str; 9] = ["LC6", "LC12", "HC6", "HC12", "1.0", "2.0", "3.0", "5.0", "10.0"];
pub static XISEL: [&'static str; 23] = [
    "OFF", "FX1L", "FX1R", "FX2L", "FX2R", "FX3L", "FX3R", "FX4L", "FX4R",
    "FX5L", "FX5R", "FX6L", "FX6R", "FX7L", "FX7R", "FX8L", "FX8R", "AUX1",
    "AUX2", "AUX3", "AUX4", "AUX5", "AUX6"];
pub static XEQTY1: [&'static str; 6] = ["LCut", "LShv", "PEQ", "VEQ", "HShv", "HCut"];