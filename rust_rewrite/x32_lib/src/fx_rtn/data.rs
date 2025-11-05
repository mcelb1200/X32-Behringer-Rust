
use crate::common::{CommandFlags, CommandFormat, CommandValue, X32Command};

pub static OFF_ON: &[&str] = &["OFF", "ON"];
pub static X_COLORS: &[&str] = &[
    "OFF", "RD", "GN", "YE", "BL", "MG", "CY", "WH", "OFFi", "RDi", "GNi", "YEi", "BLi", "MGi",
    "CYi", "WHi",
];
pub static X_EQ_TYPE_1: &[&str] = &["LCut", "LShv", "PEQ", "VEQ", "HShv", "HCut"];
pub static X_M_TYPE: &[&str] = &["IN/LC", "<-EQ", "EQ->", "PRE", "POST", "GRP"];

#[rustfmt::skip]
pub static XFXRTN01: &[X32Command] = &[
    X32Command { command: "/fxrtn".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: "/fxrtn/01".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
];
