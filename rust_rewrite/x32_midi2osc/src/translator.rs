//! # Translator Module
//!
//! Handles the translation of MIDI messages to OSC messages.

use crate::config::MidiToOsc;
use osc_lib::{OscMessage, OscArg};

/// Translates a MIDI message to an OSC message based on the provided mappings.
///
/// # Arguments
///
/// * `message` - The raw MIDI message bytes.
/// * `commands` - A slice of `MidiToOsc` structs representing the MIDI-to-OSC mappings.
///
/// # Returns
///
/// An `Option<OscMessage>` containing the translated OSC message if a match is found,
/// otherwise `None`.
pub fn translate(message: &[u8], commands: &[MidiToOsc]) -> Option<OscMessage> {
    if message.len() < 2 {
        return None;
    }

    let command = (message[0] as u32) & 0xF0;
    let channel = (message[0] as u8) & 0x0F;
    let data1 = message[1];
    let data2 = if message.len() > 2 { message[2] } else { 0 };

    for cmd in commands {
        if cmd.midi_command == command &&
           (cmd.midi_channel == 0 || cmd.midi_channel == channel + 1) &&
           (cmd.midi_data1 == 0 || cmd.midi_data1 == data1) {

            // RPN evaluation will be implemented in a future step.
            // For now, we'll just parse the OSC command string.
            if let Ok(mut osc_msg) = OscMessage::from_str(&cmd.osc_command) {
                // Replace placeholders with MIDI data
                for arg in &mut osc_msg.args {
                    if let OscArg::String(s) = arg {
                        match s.as_str() {
                            "$d1" => *arg = OscArg::Int(data1 as i32),
                            "$d2" => *arg = OscArg::Int(data2 as i32),
                            "$ch" => *arg = OscArg::Int(channel as i32 + 1),
                            _ => {}
                        }
                    }
                }
                return Some(osc_msg);
            }
        }
    }

    None
}
