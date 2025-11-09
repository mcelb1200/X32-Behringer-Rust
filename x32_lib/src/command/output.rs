//! Provides functions for generating OSC commands to control X32/M32 physical outputs.
//!
//! This module is responsible for routing signals to the physical output connectors on the
//! back of the console, such as the main XLR outputs, auxiliary outputs, and AES/EBU outputs.
use osc_lib::OscArg;

// --- Address String Getters ---

/// Returns the OSC address for a main output's source.
pub fn main_output_source(output_num: u8) -> String {
    format!("/outputs/main/{:02}/src", output_num)
}

// --- OSC Message Setters ---

/// Creates an OSC message to set the main output source.
///
/// # Arguments
///
/// * `output_num` - The output number (1-16).
/// * `source` - The source for the output.
///
/// ```
/// use x32_lib::command::output;
///
/// let (address, args) = output::set_main_output_source(1, 2);
/// assert_eq!(address, "/outputs/main/01/src");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_main_output_source(output_num: u8, source: i32) -> (String, Vec<OscArg>) {
    (main_output_source(output_num), vec![OscArg::Int(source)])
}
