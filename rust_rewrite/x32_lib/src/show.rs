//! # Show Module
//!
//! Controls the show, scene, cue, and snippet management on the X32/M32.
//!
//! This module provides functions for managing show files, scenes, cues, and snippets.

use crate::common::On;
use osc_lib::OscArg;

/// Sets the name of the current show.
///
/// # Arguments
///
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::show;
/// use osc_lib::OscArg;
///
/// let (address, args) = show::set_show_name("My Show");
/// assert_eq!(address, "/-show/showfile/show/name");
/// assert_eq!(args, vec![OscArg::String("My Show".to_string())]);
/// ```
pub fn set_show_name(name: &str) -> (String, Vec<OscArg>) {
    let address = "/-show/showfile/show/name".to_string();
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the name for a specific cue.
///
/// # Arguments
///
/// * `cue_id` - The ID of the cue (0-499).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::show;
/// use osc_lib::OscArg;
///
/// let (address, args) = show::set_cue_name(1, "First Cue");
/// assert_eq!(address, "/-show/showfile/cue/001/name");
/// assert_eq!(args, vec![OscArg::String("First Cue".to_string())]);
/// ```
pub fn set_cue_name(cue_id: u16, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/-show/showfile/cue/{:03}/name", cue_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the skip state for a specific cue.
///
/// # Arguments
///
/// * `cue_id` - The ID of the cue (0-499).
/// * `on` - The skip state (`On::On` or `On::Off`).
pub fn set_cue_skip(cue_id: u16, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/-show/showfile/cue/{:03}/skip", cue_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the scene for a specific cue.
///
/// # Arguments
///
/// * `cue_id` - The ID of the cue (0-499).
/// * `scene` - The scene ID (0-99).
pub fn set_cue_scene(cue_id: u16, scene: u16) -> (String, Vec<OscArg>) {
    let address = format!("/-show/showfile/cue/{:03}/scene", cue_id);
    let args = vec![OscArg::Int(scene as i32)];
    (address, args)
}

/// Sets the name for a specific scene.
///
/// # Arguments
///
/// * `scene_id` - The ID of the scene (0-99).
/// * `name` - The name to set.
pub fn set_scene_name(scene_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/-show/showfile/scene/{:03}/name", scene_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the notes for a specific scene.
///
/// # Arguments
///
/// * `scene_id` - The ID of the scene (0-99).
/// * `notes` - The notes to set.
pub fn set_scene_notes(scene_id: u8, notes: &str) -> (String, Vec<OscArg>) {
    let address = format!("/-show/showfile/scene/{:03}/notes", scene_id);
    let args = vec![OscArg::String(notes.to_string())];
    (address, args)
}

/// Sets the name for a specific snippet.
///
/// # Arguments
///
/// * `snippet_id` - The ID of the snippet (0-99).
/// * `name` - The name to set.
pub fn set_snippet_name(snippet_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/-show/showfile/snippet/{:03}/name", snippet_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_show_name() {
        let (address, args) = set_show_name("My Show");
        assert_eq!(address, "/-show/showfile/show/name");
        assert_eq!(args, vec![OscArg::String("My Show".to_string())]);
    }

    #[test]
    fn test_set_cue_name() {
        let (address, args) = set_cue_name(1, "First Cue");
        assert_eq!(address, "/-show/showfile/cue/001/name");
        assert_eq!(args, vec![OscArg::String("First Cue".to_string())]);
    }
}
