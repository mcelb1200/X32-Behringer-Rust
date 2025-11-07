//! This module provides the command definitions for the X32 output channels.
use osc_lib::OscArg;

/// Sets the main output source.
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
    let address = format!("/outputs/main/{:02}/src", output_num);
    (address, vec![OscArg::Int(source)])
}
