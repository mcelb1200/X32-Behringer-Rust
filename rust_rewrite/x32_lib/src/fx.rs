//! # FX Module
//!
//! Controls the 8 internal FX processors on the X32/M32.
//!
//! This module provides functions for setting the type, source, and parameters of each FX processor.

use crate::common::{FxSource, CommandFlags, CommandFormat, CommandValue, X32Command};
use osc_lib::OscArg;

/// Represents the types of effects available in FX slots 1-4.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum FxType1 {
    Hall = 0,
    Ambi,
    Rplt,
    Room,
    Cham,
    Plat,
    Vrev,
    Vrm,
    Gate,
    Rvrs,
    Dly,
    Tap3,
    Tap4,
    Crs,
    Flng,
    Phas,
    Dimc,
    Filt,
    Rota,
    Pan,
    Sub,
    DRv,
    CrR,
    FlR,
    DCr,
    DFl,
    Modd,
}

impl FxType1 {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(FxType1::Hall),
            1 => Some(FxType1::Ambi),
            2 => Some(FxType1::Rplt),
            3 => Some(FxType1::Room),
            4 => Some(FxType1::Cham),
            5 => Some(FxType1::Plat),
            6 => Some(FxType1::Vrev),
            7 => Some(FxType1::Vrm),
            8 => Some(FxType1::Gate),
            9 => Some(FxType1::Rvrs),
            10 => Some(FxType1::Dly),
            11 => Some(FxType1::Tap3),
            12 => Some(FxType1::Tap4),
            13 => Some(FxType1::Crs),
            14 => Some(FxType1::Flng),
            15 => Some(FxType1::Phas),
            16 => Some(FxType1::Dimc),
            17 => Some(FxType1::Filt),
            18 => Some(FxType1::Rota),
            19 => Some(FxType1::Pan),
            20 => Some(FxType1::Sub),
            21 => Some(FxType1::DRv),
            22 => Some(FxType1::CrR),
            23 => Some(FxType1::FlR),
            24 => Some(FxType1::DCr),
            25 => Some(FxType1::DFl),
            26 => Some(FxType1::Modd),
            _ => None,
        }
    }
}

/// Represents the types of effects available in FX slots 5-8.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum FxType2 {
    Geq2 = 0,
    Geq,
    Teq2,
    Teq,
    Des2,
    Des,
    P1a,
    P1a2,
    Pq5,
    Pq5s,
    Wavd,
    Lim,
    Cmb,
    Cmb2,
    Fac,
    Fac1m,
    Fac2,
    Lec,
    Lec2,
    Ulc,
    Ulc2,
    Enh2,
    Enh,
    Exc2,
    Exc,
    Img,
    Edi,
    Son,
    Amp2,
    Amp,
    Drv2,
    Drv,
    Phas,
    Filt,
    Pan,
    Sub,
}

impl FxType2 {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(FxType2::Geq2),
            1 => Some(FxType2::Geq),
            2 => Some(FxType2::Teq2),
            3 => Some(FxType2::Teq),
            4 => Some(FxType2::Des2),
            5 => Some(FxType2::Des),
            6 => Some(FxType2::P1a),
            7 => Some(FxType2::P1a2),
            8 => Some(FxType2::Pq5),
            9 => Some(FxType2::Pq5s),
            10 => Some(FxType2::Wavd),
            11 => Some(FxType2::Lim),
            12 => Some(FxType2::Cmb),
            13 => Some(FxType2::Cmb2),
            14 => Some(FxType2::Fac),
            15 => Some(FxType2::Fac1m),
            16 => Some(FxType2::Fac2),
            17 => Some(FxType2::Lec),
            18 => Some(FxType2::Lec2),
            19 => Some(FxType2::Ulc),
            20 => Some(FxType2::Ulc2),
            21 => Some(FxType2::Enh2),
            22 => Some(FxType2::Enh),
            23 => Some(FxType2::Exc2),
            24 => Some(FxType2::Exc),
            25 => Some(FxType2::Img),
            26 => Some(FxType2::Edi),
            27 => Some(FxType2::Son),
            28 => Some(FxType2::Amp2),
            29 => Some(FxType2::Amp),
            30 => Some(FxType2::Drv2),
            31 => Some(FxType2::Drv),
            32 => Some(FxType2::Phas),
            33 => Some(FxType2::Filt),
            34 => Some(FxType2::Pan),
            35 => Some(FxType2::Sub),
            _ => None,
        }
    }
}

/// Sets the effect type for a specific FX slot (1-4).
///
/// # Arguments
///
/// * `fx_slot` - The ID of the FX slot (1-4).
/// * `fx_type` - The effect type to set.
///
/// # Example
///
/// ```
/// use x32_lib::fx;
/// use x32_lib::fx::FxType1;
/// use osc_lib::OscArg;
///
/// let (address, args) = fx::set_fx_type1(1, FxType1::Hall);
/// assert_eq!(address, "/fx/1/type");
/// assert_eq!(args, vec![OscArg::Int(0)]);
/// ```
pub fn set_fx_type1(fx_slot: u8, fx_type: FxType1) -> (String, Vec<OscArg>) {
    let address = format!("/fx/{}/type", fx_slot);
    let args = vec![OscArg::Int(fx_type as i32)];
    (address, args)
}

/// Sets the effect type for a specific FX slot (5-8).
///
/// # Arguments
///
/// * `fx_slot` - The ID of the FX slot (5-8).
/// * `fx_type` - The effect type to set.
///
/// # Example
///
/// ```
/// use x32_lib::fx;
/// use x32_lib::fx::FxType2;
/// use osc_lib::OscArg;
///
/// let (address, args) = fx::set_fx_type2(5, FxType2::Geq2);
/// assert_eq!(address, "/fx/5/type");
/// assert_eq!(args, vec![OscArg::Int(0)]);
/// ```
pub fn set_fx_type2(fx_slot: u8, fx_type: FxType2) -> (String, Vec<OscArg>) {
    let address = format!("/fx/{}/type", fx_slot);
    let args = vec![OscArg::Int(fx_type as i32)];
    (address, args)
}

/// Sets the left source for a specific FX slot.
///
/// # Arguments
///
/// * `fx_slot` - The ID of the FX slot (1-8).
/// * `source` - The source to set.
///
/// # Example
///
/// ```
/// use x32_lib::fx;
/// use x32_lib::common::FxSource;
/// use osc_lib::OscArg;
///
/// let (address, args) = fx::set_fx_source_left(1, FxSource::MixBus(1));
/// assert_eq!(address, "/fx/1/source/l");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_fx_source_left(fx_slot: u8, source: FxSource) -> (String, Vec<OscArg>) {
    let address = format!("/fx/{}/source/l", fx_slot);
    let args = vec![OscArg::Int(source.to_id() as i32)];
    (address, args)
}

/// Sets the right source for a specific FX slot.
///
/// # Arguments
///
/// * `fx_slot` - The ID of the FX slot (1-8).
/// * `source` - The source to set.
///
/// # Example
///
/// ```
/// use x32_lib::fx;
/// use x32_lib::common::FxSource;
/// use osc_lib::OscArg;
///
/// let (address, args) = fx::set_fx_source_right(1, FxSource::MixBus(2));
/// assert_eq!(address, "/fx/1/source/r");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_fx_source_right(fx_slot: u8, source: FxSource) -> (String, Vec<OscArg>) {
    let address = format!("/fx/{}/source/r", fx_slot);
    let args = vec![OscArg::Int(source.to_id() as i32)];
    (address, args)
}

/// Sets a specific parameter for an FX processor.
///
/// # Arguments
///
/// * `fx_slot` - The ID of the FX slot (1-8).
/// * `param` - The parameter ID (1-64).
/// * `value` - The value to set (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::fx;
/// use osc_lib::OscArg;
///
/// let (address, args) = fx::set_fx_param(1, 1, 0.5);
/// assert_eq!(address, "/fx/1/par/01");
/// assert_eq!(args, vec![OscArg::Float(0.5)]);
/// ```
pub fn set_fx_param(fx_slot: u8, param: u8, value: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fx/{}/par/{:02}", fx_slot, param);
    let args = vec![OscArg::Float(value)];
    (address, args)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fx_type1() {
        let (address, args) = set_fx_type1(1, FxType1::Hall);
        assert_eq!(address, "/fx/1/type");
        assert_eq!(args, vec![OscArg::Int(0i32)]);
    }

    #[test]
    fn test_set_fx_type2() {
        let (address, args) = set_fx_type2(5, FxType2::Geq2);
        assert_eq!(address, "/fx/5/type");
        assert_eq!(args, vec![OscArg::Int(0i32)]);
    }

    #[test]
    fn test_set_fx_source_left() {
        let (address, args) = set_fx_source_left(1, FxSource::Off);
        assert_eq!(address, "/fx/1/source/l");
        assert_eq!(args, vec![OscArg::Int(0i32)]);
    }

    #[test]
    fn test_set_fx_source_right() {
        let (address, args) = set_fx_source_right(1, FxSource::MixBus(1));
        assert_eq!(address, "/fx/1/source/r");
        assert_eq!(args, vec![OscArg::Int(1i32)]);
    }

    #[test]
    fn test_set_fx_param() {
        let (address, args) = set_fx_param(1, 1, 0.5);
        assert_eq!(address, "/fx/1/par/01");
        assert_eq!(args, vec![OscArg::Float(0.5f32)]);
    }
}