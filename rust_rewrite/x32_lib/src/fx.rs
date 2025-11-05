use crate::common::{CommandFlags, CommandFormat, CommandValue, X32Command};

pub const SFX_TYP1: &[&str] = &[
    " HALL", " AMBI", " RPLT", " ROOM", " CHAM", " PLAT", " VREV", " VRM", " GATE", " RVRS", " DLY",
    " 3TAP", " 4TAP", " CRS", " FLNG", " PHAS", " DIMC", " FILT", " ROTA", " PAN", " SUB", " D/RV",
    " CR/R", " FL/R", " D/CR", " D/FL", " MODD", " GEQ2", " TEQ2", " GEQ", " TEQ", " DES2", " DES",
    " P1A2", " P1A", " PQ5S", " PQ5", " WAVD", " LIM", " CMB2", " CMB", " FAC2", " FAC1M", " FAC",
    " LEC2", " LEC", " ULC2", " ULC", " ENH2", " ENH", " EXC2", " EXC", " IMG", " EDI", " SON",
    " AMP2", " AMP", " DRV2", " DRV", " PIT2", " PIT",
];
pub const SFX_TYP2: &[&str] = &[
    " GEQ2", " TEQ2", " GEQ", " TEQ", " DES2", " DES", " P1A", " P1A2", " PQ5", " PQ5S", " WAVD",
    " LIM", " FAC", " FAC1M", " FAC2", " LEC", " LEC2", " ULC", " ULC2", " ENH2", " ENH", " EXC2",
    " EXC", " IMG", " EDI", " SON", " AMP2", " AMP", " DRV2", " DRV", " PHAS", " FILT", " PAN",
    " SUB",
];
pub const SFX_SRC: &[&str] = &[
    " INS", " MIX1", " MIX2", " MIX3", " MIX4", " MIX5", " MIX6", " MIX7", " MIX8", " MIX9",
    " MIX10", " MIX11", " MIX12", " MIX13", " MIX14", " MIX15", " MIX16", " M/C",
];

#[rustfmt::skip]
#[allow(dead_code)]
pub const XFX1: &[X32Command] = &[
    X32Command { command: "/fx".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: "/fx/1".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: "/fx/1/type".to_string(), format: CommandFormat::StringList(&SFX_TYP1), flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: "/fx/1/source".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: "/fx/1/source/l".to_string(), format: CommandFormat::StringList(&SFX_SRC), flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: "/fx/1/source/r".to_string(), format: CommandFormat::StringList(&SFX_SRC), flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: "/fx/1/par".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
];
