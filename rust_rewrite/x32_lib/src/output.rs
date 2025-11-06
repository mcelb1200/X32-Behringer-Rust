//! # Output Module
//!
//! Controls the physical output routing on the X32/M32.
//!
//! This module provides functions for routing signals to the main, auxiliary, P16, AES, and record outputs.

use crate::common::On;
use osc_lib::OscArg;

/// Represents the tap point for an output.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum OutputTap {
    InLc = 0,
    InLcM,
    PreEq,
    PreEqM,
    PostEq,
    PostEqM,
    PreFader,
    PreFaderM,
    PostFader,
}

impl OutputTap {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(OutputTap::InLc),
            1 => Some(OutputTap::InLcM),
            2 => Some(OutputTap::PreEq),
            3 => Some(OutputTap::PreEqM),
            4 => Some(OutputTap::PostEq),
            5 => Some(OutputTap::PostEqM),
            6 => Some(OutputTap::PreFader),
            7 => Some(OutputTap::PreFaderM),
            8 => Some(OutputTap::PostFader),
            _ => None,
        }
    }
}

/// Represents the Ultratalk group for P16 outputs.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum UltratalkGroup {
    A = 0,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
}

impl UltratalkGroup {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(UltratalkGroup::A),
            1 => Some(UltratalkGroup::B),
            2 => Some(UltratalkGroup::C),
            3 => Some(UltratalkGroup::D),
            4 => Some(UltratalkGroup::E),
            5 => Some(UltratalkGroup::F),
            6 => Some(UltratalkGroup::G),
            7 => Some(UltratalkGroup::H),
            8 => Some(UltratalkGroup::I),
            9 => Some(UltratalkGroup::J),
            10 => Some(UltratalkGroup::K),
            11 => Some(UltratalkGroup::L),
            12 => Some(UltratalkGroup::M),
            13 => Some(UltratalkGroup::N),
            14 => Some(UltratalkGroup::O),
            15 => Some(UltratalkGroup::P),
            _ => None,
        }
    }
}

/// Represents the speaker model for P16 iQ outputs.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Speaker {
    Spkr1 = 0,
    Spkr2,
    Spkr3,
    Spkr4,
    Spkr5,
    Spkr6,
}

impl Speaker {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Speaker::Spkr1),
            1 => Some(Speaker::Spkr2),
            2 => Some(Speaker::Spkr3),
            3 => Some(Speaker::Spkr4),
            4 => Some(Speaker::Spkr5),
            5 => Some(Speaker::Spkr6),
            _ => None,
        }
    }
}

/// Represents the EQ setting for P16 iQ outputs.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Eq {
    Eq1 = 0,
    Eq2,
    Eq3,
    Eq4,
    Eq5,
    Eq6,
}

impl Eq {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Eq::Eq1),
            1 => Some(Eq::Eq2),
            2 => Some(Eq::Eq3),
            3 => Some(Eq::Eq4),
            4 => Some(Eq::Eq5),
            5 => Some(Eq::Eq6),
            _ => None,
        }
    }
}

/// Sets the source for a main output.
///
/// # Arguments
///
/// * `output_id` - The ID of the main output (1-16).
/// * `source` - The source ID.
///
/// # Example
///
/// ```
/// use x32_lib::output;
/// use osc_lib::OscArg;
///
/// let (address, args) = output::set_main_output_source(1, 1);
/// assert_eq!(address, "/outputs/main/01/src");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_main_output_source(output_id: u8, source: u8) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/main/{:02}/src", output_id);
    let args = vec![OscArg::Int(source as i32)];
    (address, args)
}

/// Sets the tap point for a main output.
///
/// # Arguments
///
/// * `output_id` - The ID of the main output (1-16).
/// * `tap` - The output tap point.
pub fn set_main_output_tap(output_id: u8, tap: OutputTap) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/main/{:02}/pos", output_id);
    let args = vec![OscArg::Int(tap as i32)];
    (address, args)
}

/// Sets the invert state for a main output.
///
/// # Arguments
///
/// * `output_id` - The ID of the main output (1-16).
/// * `on` - The invert state (`On::On` or `On::Off`).
pub fn set_main_output_invert(output_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/main/{:02}/invert", output_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the delay on or off for a main output.
///
/// # Arguments
///
/// * `output_id` - The ID of the main output (1-16).
/// * `on` - The delay state (`On::On` or `On::Off`).
pub fn set_main_output_delay_on(output_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/main/{:02}/delay/on", output_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the delay time for a main output.
///
/// # Arguments
///
/// * `output_id` - The ID of the main output (1-16).
/// * `time` - The delay time in milliseconds.
pub fn set_main_output_delay_time(output_id: u8, time: f32) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/main/{:02}/delay/time", output_id);
    let args = vec![OscArg::Float(time)];
    (address, args)
}

/// Sets the source for an auxiliary output.
///
/// # Arguments
///
/// * `output_id` - The ID of the auxiliary output (1-6).
/// * `source` - The source ID.
pub fn set_aux_output_source(output_id: u8, source: u8) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/aux/{:02}/src", output_id);
    let args = vec![OscArg::Int(source as i32)];
    (address, args)
}

/// Sets the tap point for an auxiliary output.
///
/// # Arguments
///
/// * `output_id` - The ID of the auxiliary output (1-6).
/// * `tap` - The output tap point.
pub fn set_aux_output_tap(output_id: u8, tap: OutputTap) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/aux/{:02}/pos", output_id);
    let args = vec![OscArg::Int(tap as i32)];
    (address, args)
}

/// Sets the invert state for an auxiliary output.
///
/// # Arguments
///
/// * `output_id` - The ID of the auxiliary output (1-6).
/// * `on` - The invert state (`On::On` or `On::Off`).
pub fn set_aux_output_invert(output_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/aux/{:02}/invert", output_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the source for a P16 output.
///
/// # Arguments
///
/// * `output_id` - The ID of the P16 output (1-16).
/// * `source` - The source ID.
pub fn set_p16_output_source(output_id: u8, source: u8) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/p16/{:02}/src", output_id);
    let args = vec![OscArg::Int(source as i32)];
    (address, args)
}

/// Sets the tap point for a P16 output.
///
/// # Arguments
///
/// * `output_id` - The ID of the P16 output (1-16).
/// * `tap` - The output tap point.
pub fn set_p16_output_tap(output_id: u8, tap: OutputTap) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/p16/{:02}/pos", output_id);
    let args = vec![OscArg::Int(tap as i32)];
    (address, args)
}

/// Sets the invert state for a P16 output.
///
/// # Arguments
///
/// * `output_id` - The ID of the P16 output (1-16).
/// * `on` - The invert state (`On::On` or `On::Off`).
pub fn set_p16_output_invert(output_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/p16/{:02}/invert", output_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the iQ group for a P16 output.
///
/// # Arguments
///
/// * `output_id` - The ID of the P16 output (1-16).
/// * `group` - The Ultratalk group.
pub fn set_p16_iq_group(output_id: u8, group: UltratalkGroup) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/p16/{:02}/iQ/group", output_id);
    let args = vec![OscArg::Int(group as i32)];
    (address, args)
}

/// Sets the speaker model for a P16 iQ output.
///
/// # Arguments
///
/// * `output_id` - The ID of the P16 output (1-16).
/// * `speaker` - The speaker model.
pub fn set_p16_iq_speaker(output_id: u8, speaker: Speaker) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/p16/{:02}/iQ/speaker", output_id);
    let args = vec![OscArg::Int(speaker as i32)];
    (address, args)
}

/// Sets the EQ setting for a P16 iQ output.
///
/// # Arguments
///
/// * `output_id` - The ID of the P16 output (1-16).
/// * `eq` - The EQ setting.
pub fn set_p16_iq_eq(output_id: u8, eq: Eq) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/p16/{:02}/iQ/eq", output_id);
    let args = vec![OscArg::Int(eq as i32)];
    (address, args)
}

/// Sets the iQ model for a P16 output.
///
/// # Arguments
///
/// * `output_id` - The ID of the P16 output (1-16).
/// * `model` - The model ID.
pub fn set_p16_iq_model(output_id: u8, model: u8) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/p16/{:02}/iQ/model", output_id);
    let args = vec![OscArg::Int(model as i32)];
    (address, args)
}

/// Sets the source for an AES/EBU output.
///
/// # Arguments
///
/// * `output_id` - The ID of the AES/EBU output (1-2).
/// * `source` - The source ID.
pub fn set_aes_output_source(output_id: u8, source: u8) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/aes/{:02}/src", output_id);
    let args = vec![OscArg::Int(source as i32)];
    (address, args)
}

/// Sets the tap point for an AES/EBU output.
///
/// # Arguments
///
/// * `output_id` - The ID of the AES/EBU output (1-2).
/// * `tap` - The output tap point.
pub fn set_aes_output_tap(output_id: u8, tap: OutputTap) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/aes/{:02}/pos", output_id);
    let args = vec![OscArg::Int(tap as i32)];
    (address, args)
}

/// Sets the invert state for an AES/EBU output.
///
/// # Arguments
///
/// * `output_id` - The ID of the AES/EBU output (1-2).
/// * `on` - The invert state (`On::On` or `On::Off`).
pub fn set_aes_output_invert(output_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/aes/{:02}/invert", output_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the source for a record output.
///
/// # Arguments
///
/// * `output_id` - The ID of the record output (1-2).
/// * `source` - The source ID.
pub fn set_rec_output_source(output_id: u8, source: u8) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/rec/{:02}/src", output_id);
    let args = vec![OscArg::Int(source as i32)];
    (address, args)
}

/// Sets the tap point for a record output.
///
/// # Arguments
///
/// * `output_id` - The ID of the record output (1-2).
/// * `tap` - The output tap point.
pub fn set_rec_output_tap(output_id: u8, tap: OutputTap) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/rec/{:02}/pos", output_id);
    let args = vec![OscArg::Int(tap as i32)];
    (address, args)
}

/// Sets the invert state for a record output.
///
/// # Arguments
///
/// * `output_id` - The ID of the record output (1-2).
/// * `on` - The invert state (`On::On` or `On::Off`).
pub fn set_rec_output_invert(output_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/outputs/rec/{:02}/invert", output_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_main_output_source() {
        let (address, args) = set_main_output_source(1, 1);
        assert_eq!(address, "/outputs/main/01/src");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
