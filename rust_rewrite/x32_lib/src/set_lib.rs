//! # set_lib
//!
//! A library for generating OSC messages to manage presets on the Behringer X32/X-Air digital mixers.
//! This module provides functions to save, load, and delete presets and scenes.

use osc_lib::{OscMessage, OscArg};

/// Represents the type of preset to be managed.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PresetType {
    /// Channel preset (`.chn`)
    Channel,
    /// Effect preset (`.efx`)
    Effect,
    /// Routing preset (`.rou`)
    Routing,
}

/// Generates an OSC message to save a preset to the X32 library.
///
/// # Arguments
///
/// * `preset_type` - The type of preset to save (Channel, Effect, or Routing).
/// * `index` - The library slot index (0-99).
/// * `name` - The name of the preset.
/// * `source_index` - The index of the source channel/effect to save from.
///                    This argument is ignored for `PresetType::Routing`.
///
/// # Returns
///
/// An `OscMessage` ready to be sent to the X32.
pub fn save_preset(
    preset_type: PresetType,
    index: u8,
    name: &str,
    source_index: u8,
) -> OscMessage {
    match preset_type {
        PresetType::Channel | PresetType::Effect => {
            let lib_type = if preset_type == PresetType::Channel {
                "libchan"
            } else {
                "libfx"
            };
            OscMessage {
                path: "/save".to_string(),
                args: vec![
                    OscArg::String(",sisi".to_string()),
                    OscArg::String(lib_type.to_string()),
                    OscArg::Int(index as i32),
                    OscArg::String(name.to_string()),
                    OscArg::Int(source_index as i32),
                ],
            }
        }
        PresetType::Routing => OscMessage {
            path: "/save".to_string(),
            args: vec![
                OscArg::String(",sis".to_string()),
                OscArg::String("librout".to_string()),
                OscArg::Int(index as i32),
                OscArg::String(name.to_string()),
            ],
        },
    }
}

/// Generates an OSC message to save a scene.
///
/// # Arguments
///
/// * `scene_index` - The scene number to save (0-99).
/// * `name` - The name of the scene.
/// * `note` - A note for the scene.
///
/// # Returns
///
/// An `OscMessage` ready to be sent to the X32.
pub fn save_scene(scene_index: u8, name: &str, note: &str) -> OscMessage {
    OscMessage {
        path: "/save".to_string(),
        args: vec![
            OscArg::String(",sissi".to_string()),
            OscArg::String("scene".to_string()),
            OscArg::Int(scene_index as i32),
            OscArg::String(name.to_string()),
            OscArg::String(note.to_string()),
            OscArg::Int(0),
        ],
    }
}

/// Generates an OSC message to load a preset from the X32 library.
///
/// # Arguments
///
/// * `preset_type` - The type of preset to load (Channel, Effect, or Routing).
/// * `index` - The library slot index (0-99).
/// * `target_index` - The index of the target channel/effect to load to.
///                    This argument is ignored for `PresetType::Routing`.
///
/// # Returns
///
/// An `OscMessage` ready to be sent to the X32.
pub fn load_preset(
    preset_type: PresetType,
    index: u8,
    target_index: u8,
) -> OscMessage {
    match preset_type {
        PresetType::Channel | PresetType::Effect => {
            let lib_type = if preset_type == PresetType::Channel {
                "libchan"
            } else {
                "libfx"
            };
            OscMessage {
                path: "/load".to_string(),
                args: vec![
                    OscArg::String(",sii".to_string()),
                    OscArg::String(lib_type.to_string()),
                    OscArg::Int(index as i32),
                    OscArg::Int(target_index as i32),
                ],
            }
        }
        PresetType::Routing => OscMessage {
            path: "/load".to_string(),
            args: vec![
                OscArg::String(",si".to_string()),
                OscArg::String("librout".to_string()),
                OscArg::Int(index as i32),
            ],
        },
    }
}

/// Generates an OSC message to delete a preset from the X32 library.
///
/// # Arguments
///
/// * `preset_type` - The type of preset to delete (Channel, Effect, or Routing).
/// * `index` - The library slot index (0-99).
///
/// # Returns
///
/// An `OscMessage` ready to be sent to the X32.
pub fn delete_preset(preset_type: PresetType, index: u8) -> OscMessage {
    let lib_type = match preset_type {
        PresetType::Channel => "libchan",
        PresetType::Effect => "libfx",
        PresetType::Routing => "librout",
    };
    OscMessage {
        path: "/delete".to_string(),
        args: vec![
            OscArg::String(",si".to_string()),
            OscArg::String(lib_type.to_string()),
            OscArg::Int(index as i32),
        ],
    }
}

/// Generates an OSC message to load a scene.
///
/// # Arguments
///
/// * `scene_index` - The scene number to load (0-99).
///
/// # Returns
///
/// An `OscMessage` ready to be sent to the X32.
pub fn load_scene(scene_index: u8) -> OscMessage {
    OscMessage {
        path: "/load".to_string(),
        args: vec![
            OscArg::String(",si".to_string()),
            OscArg::String("scene".to_string()),
            OscArg::Int(scene_index as i32),
        ],
    }
}

/// Generates an OSC message to delete a scene.
///
/// # Arguments
///
/// * `scene_index` - The scene number to delete (0-99).
///
/// # Returns
///
/// An `OscMessage` ready to be sent to the X32.
pub fn delete_scene(scene_index: u8) -> OscMessage {
    OscMessage {
        path: "/delete".to_string(),
        args: vec![
            OscArg::String(",si".to_string()),
            OscArg::String("scene".to_string()),
            OscArg::Int(scene_index as i32),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_channel_preset() {
        let msg = save_preset(PresetType::Channel, 10, "My Preset", 5);
        assert_eq!(msg.path, "/save");
        assert_eq!(
            msg.args,
            vec![
                OscArg::String(",sisi".to_string()),
                OscArg::String("libchan".to_string()),
                OscArg::Int(10),
                OscArg::String("My Preset".to_string()),
                OscArg::Int(5),
            ]
        );
    }

    #[test]
    fn test_save_effect_preset() {
        let msg = save_preset(PresetType::Effect, 25, "Reverb", 1);
        assert_eq!(msg.path, "/save");
        assert_eq!(
            msg.args,
            vec![
                OscArg::String(",sisi".to_string()),
                OscArg::String("libfx".to_string()),
                OscArg::Int(25),
                OscArg::String("Reverb".to_string()),
                OscArg::Int(1),
            ]
        );
    }

    #[test]
    fn test_save_routing_preset() {
        let msg = save_preset(PresetType::Routing, 0, "Default Routing", 0);
        assert_eq!(msg.path, "/save");
        assert_eq!(
            msg.args,
            vec![
                OscArg::String(",sis".to_string()),
                OscArg::String("librout".to_string()),
                OscArg::Int(0),
                OscArg::String("Default Routing".to_string()),
            ]
        );
    }

    #[test]
    fn test_load_scene() {
        let msg = load_scene(99);
        assert_eq!(msg.path, "/load");
        assert_eq!(
            msg.args,
            vec![
                OscArg::String(",si".to_string()),
                OscArg::String("scene".to_string()),
                OscArg::Int(99),
            ]
        );
    }

    #[test]
    fn test_delete_scene() {
        let msg = delete_scene(50);
        assert_eq!(msg.path, "/delete");
        assert_eq!(
            msg.args,
            vec![
                OscArg::String(",si".to_string()),
                OscArg::String("scene".to_string()),
                OscArg::Int(50),
            ]
        );
    }

    #[test]
    fn test_save_scene() {
        let msg = save_scene(10, "My Scene", "A note");
        assert_eq!(msg.path, "/save");
        assert_eq!(
            msg.args,
            vec![
                OscArg::String(",sissi".to_string()),
                OscArg::String("scene".to_string()),
                OscArg::Int(10),
                OscArg::String("My Scene".to_string()),
                OscArg::String("A note".to_string()),
                OscArg::Int(0),
            ]
        );
    }

    #[test]
    fn test_load_channel_preset() {
        let msg = load_preset(PresetType::Channel, 10, 5);
        assert_eq!(msg.path, "/load");
        assert_eq!(
            msg.args,
            vec![
                OscArg::String(",sii".to_string()),
                OscArg::String("libchan".to_string()),
                OscArg::Int(10),
                OscArg::Int(5),
            ]
        );
    }

    #[test]
    fn test_delete_effect_preset() {
        let msg = delete_preset(PresetType::Effect, 25);
        assert_eq!(msg.path, "/delete");
        assert_eq!(
            msg.args,
            vec![
                OscArg::String(",si".to_string()),
                OscArg::String("libfx".to_string()),
                OscArg::Int(25),
            ]
        );
    }
}
