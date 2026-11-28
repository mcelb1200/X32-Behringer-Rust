//! Complete catalog of all 48 X32/Wing/X-Air tools and applications exposed via `x32-cli`.

use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// Execution mode of a workspace tool.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolMode {
    /// Runs to completion and exits.
    OneShot,
    /// Long-running background daemon/service.
    Daemon,
    /// Interactive terminal UI requiring a TTY.
    InteractiveTui,
}

/// Input widget type rendered in the Web UI for a tool parameter.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// Free-form string input.
    Text,
    /// Mixer IP address (auto-fills from connected mixer).
    Ip,
    /// Numeric input.
    Number,
    /// Boolean flag switch.
    Bool,
    /// Fixed dropdown selection.
    Select,
    /// File or directory path.
    Path,
}

/// Metadata describing a single CLI flag or positional argument for a tool.
#[derive(Debug, Clone, Serialize)]
pub struct ToolFieldSpec {
    /// Unique field key used in form submissions.
    pub name: &'static str,
    /// CLI flag (e.g. `"--ip"`), or `""` for positional arguments.
    pub flag: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Widget type.
    pub field_type: FieldType,
    /// Whether this argument is required to launch.
    pub required: bool,
    /// Default value pre-populated in the UI.
    pub default_value: &'static str,
    /// Placeholder hint text.
    pub placeholder: &'static str,
    /// Allowed options for [`FieldType::Select`].
    pub options: &'static [&'static str],
}

/// Catalog entry for a workspace tool or app.
#[derive(Debug, Clone, Serialize)]
pub struct ToolSpec {
    /// Subcommand name passed to `x32-cli` (e.g. `"x32-reaper"`).
    pub id: &'static str,
    /// Display name shown in the UI.
    pub name: &'static str,
    /// Category filter tag.
    pub category: &'static str,
    /// Concise functional summary.
    pub description: &'static str,
    /// Execution mode.
    pub mode: ToolMode,
    /// Optional fixed subcommand token appended after `id` if needed.
    pub subcommand: Option<&'static str>,
    /// Ordered CLI parameters.
    pub fields: Vec<ToolFieldSpec>,
}

fn ip_field(flag: &'static str, required: bool, default_value: &'static str) -> ToolFieldSpec {
    ToolFieldSpec {
        name: "ip",
        flag,
        label: "Mixer IP",
        field_type: FieldType::Ip,
        required,
        default_value,
        placeholder: "192.168.1.64",
        options: &[],
    }
}

fn model_field(flag: &'static str) -> ToolFieldSpec {
    ToolFieldSpec {
        name: "model",
        flag,
        label: "Mixer Model",
        field_type: FieldType::Select,
        required: false,
        default_value: "X32",
        placeholder: "",
        options: &["X32", "Wing", "XR18", "XR16", "XR12"],
    }
}

fn text_field(
    name: &'static str,
    flag: &'static str,
    label: &'static str,
    required: bool,
    default_value: &'static str,
    placeholder: &'static str,
) -> ToolFieldSpec {
    ToolFieldSpec {
        name,
        flag,
        label,
        field_type: FieldType::Text,
        required,
        default_value,
        placeholder,
        options: &[],
    }
}

fn path_field(
    name: &'static str,
    flag: &'static str,
    label: &'static str,
    required: bool,
    default_value: &'static str,
    placeholder: &'static str,
) -> ToolFieldSpec {
    ToolFieldSpec {
        name,
        flag,
        label,
        field_type: FieldType::Path,
        required,
        default_value,
        placeholder,
        options: &[],
    }
}

fn num_field(
    name: &'static str,
    flag: &'static str,
    label: &'static str,
    default_value: &'static str,
    placeholder: &'static str,
) -> ToolFieldSpec {
    ToolFieldSpec {
        name,
        flag,
        label,
        field_type: FieldType::Number,
        required: false,
        default_value,
        placeholder,
        options: &[],
    }
}

fn bool_field(
    name: &'static str,
    flag: &'static str,
    label: &'static str,
    default_value: &'static str,
) -> ToolFieldSpec {
    ToolFieldSpec {
        name,
        flag,
        label,
        field_type: FieldType::Bool,
        required: false,
        default_value,
        placeholder: "",
        options: &[],
    }
}

/// Returns the complete catalog of all 48 workspace tools and applications.
pub fn all_tools() -> Vec<ToolSpec> {
    vec![
        // --- 1. Volunteer & Safety ---
        ToolSpec {
            id: "x32-volunteer",
            name: "Volunteer Mode Dashboard",
            category: "Volunteer & Safety",
            description: "Simplified TUI dashboard with fader limits, clipping alerts, automix, and panic mute.",
            mode: ToolMode::InteractiveTui,
            subcommand: None,
            fields: vec![
                ip_field("--ip", false, "auto"),
                model_field("--model"),
                text_field("channels", "--channels", "Channels/DCAs", false, "", "ch1,ch2,dca1"),
                path_field("config", "--config", "Config JSON", false, "", "x32_config.json"),
                num_field("max_fader_limit_db", "--max-fader-limit-db", "Max Fader (dBFS)", "", "-3.0"),
            ],
        },
        ToolSpec {
            id: "x32-safe-mute",
            name: "Safe Panic Mute",
            category: "Volunteer & Safety",
            description: "Smoothly fades Main LR, buses, matrices, or DCAs down to silence without speaker pops.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                ToolFieldSpec {
                    name: "mode",
                    flag: "--mode",
                    label: "Mute Scope",
                    field_type: FieldType::Select,
                    required: false,
                    default_value: "all",
                    placeholder: "",
                    options: &["all", "monitors", "main", "dca"],
                },
                text_field("dcas", "--dcas", "DCAs (if mode=dca)", false, "", "1,2"),
            ],
        },
        ToolSpec {
            id: "x32-scene-checker",
            name: "Scene Risk Checker",
            category: "Volunteer & Safety",
            description: "Pre-flight .scn risk analyzer with safe-load, path locking, and pre-load backup.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                path_field("scene", "--scene", "Scene File (.scn)", true, "", "show.scn"),
                model_field("--model"),
                bool_field("safe_load", "--safe-load", "Safe Load (skip High/Critical)", "true"),
                bool_field("auto_load", "--auto-load", "Auto Load Without Prompt", "false"),
            ],
        },
        ToolSpec {
            id: "x32-speech-mode",
            name: "Speech Mode Macro",
            category: "Volunteer & Safety",
            description: "Applies safe spoken-word HPF, EQ, and compression across selected channels.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                text_field("channels", "--channels", "Channels (e.g. 1,2,3)", true, "1", "1,2,3"),
                model_field("--model"),
            ],
        },
        ToolSpec {
            id: "x32-system-tune",
            name: "Guided Room Tuning",
            category: "Volunteer & Safety",
            description: "Cut-only RTA room calibration TUI with 16 built-in microphone compensation profiles.",
            mode: ToolMode::InteractiveTui,
            subcommand: None,
            fields: vec![
                ip_field("--ip", false, "auto"),
                model_field("--model"),
                text_field("mic_profile", "--mic-profile", "Mic Profile", false, "sm58", "sm58, m5, ecm8000"),
                num_field("mic_channel", "--mic-channel", "Mic Input Channel", "1", "1"),
                num_field("max_level", "--max-level", "Max Pink Level (0..0.25)", "0.25", "0.25"),
            ],
        },
        // --- 2. Intelligent Mixing & DSP ---
        ToolSpec {
            id: "x32-automix",
            name: "Dugan-Style Automixer",
            category: "Intelligent Mixing & DSP",
            description: "Automatic gain-sharing across active speech microphones.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("start_channel", "--start-channel", "Start Channel", "1", "1"),
                num_field("stop_channel", "--stop-channel", "Stop Channel", "8", "8"),
                num_field("sensitivity", "--sensitivity", "Sensitivity", "0.005", "0.005"),
                num_field("down_delay", "--down-delay", "Down Delay (s)", "5", "5"),
            ],
        },
        ToolSpec {
            id: "x32-auto-gain",
            name: "Auto Gain Staging",
            category: "Intelligent Mixing & DSP",
            description: "Continuously targets optimal dBFS headroom on selected channels.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                text_field("channels", "--channels", "Channels", true, "1,2", "1,2,3"),
                model_field("--model"),
                num_field("target_dbfs", "--target-dbfs", "Target dBFS", "-18.0", "-18.0"),
                num_field("tolerance_db", "--tolerance-db", "Tolerance (dB)", "2.0", "2.0"),
            ],
        },
        ToolSpec {
            id: "x32-auto-ringout",
            name: "Monitor Auto-Ringout",
            category: "Intelligent Mixing & DSP",
            description: "Subscribes to 100-band RTA (/meters/4) to notch out feedback frequencies on mix buses.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                text_field("buses", "--buses", "Buses (e.g. 1,2)", true, "1", "1,2"),
                num_field("target_dbfs", "--target-dbfs", "Target dBFS", "-6.0", "-6.0"),
                num_field("max_notches", "--max-notches", "Max Notches/Bus", "5", "5"),
                bool_field("simulate", "--simulate", "Simulate Mode", "false"),
            ],
        },
        ToolSpec {
            id: "x32-feedback-detect",
            name: "Audio Feedback Detector",
            category: "Intelligent Mixing & DSP",
            description: "Monitors a local audio input interface via FFT and inserts dynamic EQ notches on X32.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                num_field("channel", "--channel", "Target X32 Channel", "1", "1"),
                model_field("--model"),
                text_field("device", "--device", "Audio Device", false, "", "optional substring"),
                num_field("input_channel", "--input-channel", "Audio Input Ch", "1", "1"),
                bool_field("list_devices", "--list-devices", "List Audio Devices & Exit", "false"),
            ],
        },
        ToolSpec {
            id: "x32-vocal-ducking",
            name: "Vocal Ducking Sidechain",
            category: "Intelligent Mixing & DSP",
            description: "Ducks an instrument bus whenever vocal energy crosses threshold.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("vocal_bus", "--vocal-bus", "Vocal Bus", "1", "1"),
                num_field("instrument_bus", "--instrument-bus", "Instrument Bus", "2", "2"),
                num_field("threshold", "--threshold", "Threshold (dBFS)", "-35.0", "-35.0"),
            ],
        },
        ToolSpec {
            id: "x32-loudness",
            name: "SPL Loudness EQ",
            category: "Intelligent Mixing & DSP",
            description: "Applies Fletcher-Munson loudness compensation relative to monitored fader level.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                path_field("config", "--config", "Config JSON", false, ".x32_loudness.json", ".x32_loudness.json"),
                text_field("fader_path", "--fader-path", "Monitored Fader", false, "/main/st/mix/fader", "/main/st/mix/fader"),
            ],
        },
        // --- 3. Live Performance ---
        ToolSpec {
            id: "x32-autobeat",
            name: "AutoBeat BPM Sync",
            category: "Live Performance",
            description: "Detects live BPM via audio or OSC meters and syncs X32 delay FX slots.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                text_field("channels", "--channels", "Audio Source", false, "ch1", "ch1,bus8"),
                text_field("slot", "--slot", "FX Slot(s)", false, "1", "1,2"),
                text_field("device", "--device", "Audio Device", false, "", "optional"),
            ],
        },
        ToolSpec {
            id: "x32-tap",
            name: "Tap Tempo (CLI)",
            category: "Live Performance",
            description: "Controls delay tap tempo via CLI or auto gate-meter thresholding.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("slot", "--slot", "FX Slot", "1", "1"),
                bool_field("auto", "--auto", "Auto-Tap Mode", "false"),
                num_field("channel", "--channel", "Trigger Channel", "1", "1"),
                num_field("threshold", "--threshold", "Threshold", "0.5", "0.5"),
            ],
        },
        ToolSpec {
            id: "x32-tapw",
            name: "Tap Tempo (TUI)",
            category: "Live Performance",
            description: "Interactive terminal UI for manual and auto-threshold delay tap tempo.",
            mode: ToolMode::InteractiveTui,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("slot", "--slot", "FX Slot", "1", "1"),
                bool_field("auto", "--auto", "Auto-Tap Mode", "false"),
                num_field("channel", "--channel", "Trigger Channel", "1", "1"),
            ],
        },
        ToolSpec {
            id: "x32-crossfade",
            name: "Scene Crossfader",
            category: "Live Performance",
            description: "Smoothly interpolates faders, EQ, and dynamics between two .scn files.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                path_field("scene_a", "--scene-a", "Scene A (.scn)", true, "", "a.scn"),
                path_field("scene_b", "--scene-b", "Scene B (.scn)", true, "", "b.scn"),
                model_field("--model"),
                num_field("duration", "--duration", "Duration (s)", "5.0", "5.0"),
            ],
        },
        ToolSpec {
            id: "x32-fade",
            name: "Timed Fader Controller",
            category: "Live Performance",
            description: "Executes timed fade-in/fade-out transitions on target OSC fader paths.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", false, "127.0.0.1"),
                model_field("--model"),
                text_field("fader", "--fader", "Fader Path", false, "/main/st/mix/fader", "/ch/01/mix/fader"),
                num_field("fade_in", "--fade-in", "Fade In (s)", "", "3.0"),
                num_field("fade_out", "--fade-out", "Fade Out (s)", "", "3.0"),
            ],
        },
        ToolSpec {
            id: "x32-dca-spill",
            name: "DCA Spill Controller",
            category: "Live Performance",
            description: "Monitors DCA selections and spills member channels onto console banks.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("dca", "--dca", "DCA Group (1-8, blank=all)", "", "1"),
            ],
        },
        ToolSpec {
            id: "x32-custom-layer",
            name: "Custom Layer Manager",
            category: "Live Performance",
            description: "Assigns, saves, and restores custom fader layers on the X32 surface.",
            mode: ToolMode::OneShot,
            subcommand: Some("list"),
            fields: vec![ip_field("--ip", true, "127.0.0.1")],
        },
        // --- 4. DAW & Studio ---
        ToolSpec {
            id: "x32-reaper",
            name: "REAPER OSC Bridge",
            category: "DAW & Studio",
            description: "Bidirectional X32 <-> REAPER sync for faders, mutes, pans, FX, and transport.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                path_field("config", "--config", "INI Config Path", false, ".X32Reaper.ini", ".X32Reaper.ini"),
                text_field("bind_ip", "--bind-ip", "Local Bind IP", false, "127.0.0.1", "127.0.0.1"),
            ],
        },
        ToolSpec {
            id: "x32-punch-control",
            name: "MTC Punch Automation",
            category: "DAW & Studio",
            description: "Records and replays MIDI Time Code (.xpc) synchronized punch automation.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", false, "127.0.0.1"),
                path_field("config", "--config", "INI Config", false, ".X32PunchControl.ini", ".X32PunchControl.ini"),
                path_field("file", "--file", "Punch File (.xpc)", false, "", "show.xpc"),
            ],
        },
        ToolSpec {
            id: "x32-midi2osc",
            name: "MIDI-to-OSC Translator",
            category: "DAW & Studio",
            description: "Translates MIDI Note/CC/Program Change events into X32 OSC via .m2o rules.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", false, "127.0.0.1"),
                path_field("file", "--file", "Mapping File (.m2o)", true, "", "mapping.m2o"),
                text_field("midi_in", "--midi-in", "MIDI In Port", false, "", "MIDI port substring"),
            ],
        },
        ToolSpec {
            id: "x32-commander",
            name: "X32 Commander Triggers",
            category: "DAW & Studio",
            description: "Event-driven X32 OSC/MIDI trigger router driven by X32Commander.txt rules.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                path_field("file", "--file", "Command Rules File", false, "X32Commander.txt", "X32Commander.txt"),
                text_field("output", "--output", "OSC Out Target", false, "", "127.0.0.1:9000"),
            ],
        },
        ToolSpec {
            id: "x32-sync",
            name: "Multi-Console Sync",
            category: "DAW & Studio",
            description: "Bidirectionally mirrors OSC parameters between Console A and Console B.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip-a", true, "127.0.0.1"),
                text_field("ip_b", "--ip-b", "Console B IP", true, "", "192.168.1.65"),
                model_field("--model"),
                text_field("prefix", "--prefix", "OSC Prefix Filter", false, "", "/ch/01"),
            ],
        },
        // --- 5. Backup & Scenes ---
        ToolSpec {
            id: "x32-desk-save",
            name: "Full Desk Backup",
            category: "Backup & Scenes",
            description: "Saves full console state (scene, preferences, routing) to a text backup file.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                bool_field("desk_save", "--desk-save", "Include Desk Prefs", "true"),
                bool_field("scene", "--scene", "Include Scene", "true"),
                bool_field("routing", "--routing", "Include Routing", "true"),
                path_field("destination_file", "", "Destination File", true, "x32_backup.txt", "x32_backup.txt"),
            ],
        },
        ToolSpec {
            id: "x32-desk-restore",
            name: "Full Desk Restore",
            category: "Backup & Scenes",
            description: "Restores a saved full console backup file back onto the mixer.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                path_field("file", "", "Backup File", true, "x32_backup.txt", "x32_backup.txt"),
            ],
        },
        ToolSpec {
            id: "x32-get-scene",
            name: "Download Scene (.scn)",
            category: "Backup & Scenes",
            description: "Queries OSC nodes from stdin and formats an X32 .scn scene dump.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                text_field("scene_name", "--scene-name", "Scene Name", false, "Backup", "Backup"),
                text_field("note", "--note", "Scene Note", false, "", "Notes"),
            ],
        },
        ToolSpec {
            id: "x32-set-scene",
            name: "Upload Scene (.scn)",
            category: "Backup & Scenes",
            description: "Streams .scn / OSC commands from stdin to an X32 console.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("delay", "--delay", "Delay (ms)", "1", "1"),
            ],
        },
        ToolSpec {
            id: "x32-get-scene-name",
            name: "Query Active Scene Name",
            category: "Backup & Scenes",
            description: "Queries or monitors active scene changes on the console.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("onetime", "--onetime", "Exit After First (0/1)", "1", "1"),
            ],
        },
        ToolSpec {
            id: "x32-get-lib",
            name: "Download Library Presets",
            category: "Backup & Scenes",
            description: "Downloads internal channel, FX, and routing library presets (.chn/.efx/.rou).",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                path_field("output_dir", "--output-dir", "Output Directory", false, ".", "."),
            ],
        },
        ToolSpec {
            id: "x32-set-lib",
            name: "Upload Library Presets",
            category: "Backup & Scenes",
            description: "Uploads .chn, .efx, or .rou files into console library slots.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("start_index", "--start-index", "Start Slot (1-100)", "1", "1"),
                path_field("files", "", "Preset File", true, "", "vocal.chn"),
            ],
        },
        ToolSpec {
            id: "x32-set-preset",
            name: "Load Preset to Slot",
            category: "Backup & Scenes",
            description: "Applies a .chn/.efx/.rou preset directly to a target channel/bus/FX with safe flags.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                path_field("file", "", "Preset File", true, "", "kick.chn"),
                text_field("target", "--target", "Target Slot", false, "ch01", "ch01, bus01, fx1"),
            ],
        },
        ToolSpec {
            id: "x32-copy-fx",
            name: "Copy FX Slot",
            category: "Backup & Scenes",
            description: "Copies effect slot settings between FX slots or defaults.",
            mode: ToolMode::OneShot,
            subcommand: Some("copy"),
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("from", "--from", "Source FX Slot", "1", "1"),
                num_field("to", "--to", "Dest FX Slot", "2", "2"),
            ],
        },
        ToolSpec {
            id: "x32-geq2-cpy",
            name: "Copy GEQ2 Bands",
            category: "Backup & Scenes",
            description: "Copies 31-band Graphic EQ curves between FX slots / A-B sides.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                num_field("from", "--from", "From FX Slot", "1", "1"),
                num_field("to", "--to", "To FX Slot", "2", "2"),
            ],
        },
        ToolSpec {
            id: "xair-get-scene",
            name: "X-Air Get Scene",
            category: "Backup & Scenes",
            description: "Downloads scene data from XR18/XR16/XR12 consoles.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "192.168.0.64"),
                text_field("scene_name", "--scene-name", "Scene Name", false, "XAirBackup", "XAirBackup"),
            ],
        },
        ToolSpec {
            id: "xair-set-scene",
            name: "X-Air Set Scene",
            category: "Backup & Scenes",
            description: "Uploads scene OSC commands from stdin to XR18/XR16/XR12.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "192.168.0.64"),
                num_field("delay", "--delay", "Delay (ms)", "1", "1"),
            ],
        },
        // --- 6. Recording & X-Live ---
        ToolSpec {
            id: "x32-xlive-wav",
            name: "X-Live! -> Mono WAVs",
            category: "Recording & X-Live",
            description: "Splits multi-channel X-Live! SD recordings into individual mono WAV tracks for REAPER.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                path_field("session_dir", "", "X-Live Session Dir", true, "", "./X_LIVE_SESSION"),
                path_field("dest_dir", "--dest-dir", "Output Dir", false, "./", "./out"),
                num_field("num_channels", "--num-channels", "Channels (1-32)", "", "32"),
                path_field("scene_file", "--scene-file", "Optional .scn for names", false, "", "show.scn"),
            ],
        },
        ToolSpec {
            id: "x32-wav-xlive",
            name: "Mono WAVs -> X-Live!",
            category: "Recording & X-Live",
            description: "Merges mono WAV files into an X-Live! SD card multitrack session.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                path_field("session_dir", "", "WAV Source Dir", true, "", "./stems"),
                text_field("session_name", "", "Session Name", false, "", "VirtualSoundcheck"),
            ],
        },
        ToolSpec {
            id: "x32-cpxlivemarkers",
            name: "X-Live! Markers -> REAPER",
            category: "Recording & X-Live",
            description: "Extracts X-Live! SE_LOG.BIN markers into REAPER or Audition marker CSV.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                path_field("file", "", "SE_LOG.BIN Path", true, "", "SE_LOG.BIN"),
                ToolFieldSpec {
                    name: "format",
                    flag: "--format",
                    label: "Format",
                    field_type: FieldType::Select,
                    required: false,
                    default_value: "reaper",
                    placeholder: "",
                    options: &["reaper", "audition"],
                },
                text_field("prefix", "--prefix", "Marker Prefix", false, "", "Song "),
            ],
        },
        ToolSpec {
            id: "x32-jog4xlive",
            name: "X-Live! Hardware Jog Wheel",
            category: "Recording & X-Live",
            description: "Maps X32 User Assign Bank C encoders 1 & 3 to scrub X-Live! playback.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                bool_field("verbose", "--verbose", "Verbose", "true"),
            ],
        },
        ToolSpec {
            id: "x32-replay",
            name: "OSC Traffic Recorder/Player",
            category: "Recording & X-Live",
            description: "Records or replays timestamped OSC mixer traffic to/from binary files.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                path_field("file", "--file", "Replay File", false, "X32ReplayFile.bin", "X32ReplayFile.bin"),
                bool_field("verbose", "--verbose", "Verbose", "true"),
            ],
        },
        // --- 7. Network & Diagnostics ---
        ToolSpec {
            id: "x32-emulator",
            name: "Local X32 Emulator",
            category: "Network & Diagnostics",
            description: "Runs a full stateful X32/M32/Wing/X-Air simulator locally on UDP port 10023.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                text_field("ip", "--ip", "Bind IP", false, "127.0.0.1", "127.0.0.1"),
                num_field("port", "--port", "UDP Port", "10023", "10023"),
                model_field("--model"),
            ],
        },
        ToolSpec {
            id: "command",
            name: "X32 Command Sender",
            category: "Network & Diagnostics",
            description: "Sends batch or interactive OSC commands to X32/Wing/X-Air.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                model_field("--model"),
                path_field("file", "--file", "Batch File", false, "", "commands.txt"),
            ],
        },
        ToolSpec {
            id: "xair-command",
            name: "X-Air Command Sender",
            category: "Network & Diagnostics",
            description: "Sends batch or interactive OSC commands to X-Air mixers.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "192.168.0.64"),
                path_field("file", "--file", "Batch File", false, "", "commands.txt"),
            ],
        },
        ToolSpec {
            id: "x32-udp",
            name: "One-Shot UDP OSC Query",
            category: "Network & Diagnostics",
            description: "Sends a single OSC path/query over UDP and prints the mixer response.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                num_field("port", "--port", "UDP Port", "10023", "10023"),
                text_field("command", "", "OSC Command", false, "/status", "/ch/01/mix/fader"),
            ],
        },
        ToolSpec {
            id: "x32-tcp",
            name: "TCP-to-UDP OSC Bridge",
            category: "Network & Diagnostics",
            description: "Exposes a TCP OSC server that forwards traffic to the X32 UDP port.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                text_field("bind_ip", "--bind-ip", "TCP Bind IP", false, "127.0.0.1", "127.0.0.1"),
                num_field("port", "--port", "TCP Port", "10041", "10041"),
            ],
        },
        ToolSpec {
            id: "x32-usb",
            name: "X32 USB Drive Manager",
            category: "Network & Diagnostics",
            description: "Lists directories and manages files on the X32 front-panel USB drive.",
            mode: ToolMode::OneShot,
            subcommand: Some("ls"),
            fields: vec![ip_field("--ip", true, "127.0.0.1")],
        },
        ToolSpec {
            id: "x32-ssavergw",
            name: "Console Screen Saver",
            category: "Network & Diagnostics",
            description: "Dims X32 desk/scribble-strip LEDs after an idle timeout.",
            mode: ToolMode::Daemon,
            subcommand: None,
            fields: vec![
                ip_field("--ip", true, "127.0.0.1"),
                num_field("delay", "--delay", "Idle Delay (s)", "5", "5"),
            ],
        },
        ToolSpec {
            id: "x32-dump",
            name: "OSC Packet Dumper",
            category: "Network & Diagnostics",
            description: "Decodes and prints binary OSC packet captures from a file.",
            mode: ToolMode::OneShot,
            subcommand: None,
            fields: vec![path_field("file", "", "Capture File", true, "", "capture.bin")],
        },
    ]
}

/// Builds the `x32-cli` argument list for a cataloged tool from a JSON map of field values.
pub fn build_cli_args(
    tool: &ToolSpec,
    values: &HashMap<String, serde_json::Value>,
) -> anyhow::Result<Vec<String>> {
    let mut args = vec![tool.id.to_string()];

    // Separate flagged options and positional arguments (flag == "").
    // Some clap tools require positional args after flags; ordering flags first then positional works cleanly.
    let mut positional = Vec::new();

    for field in &tool.fields {
        let raw_val = values.get(field.name);
        let str_val = match raw_val {
            Some(serde_json::Value::String(s)) => s.trim().to_string(),
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::Bool(b)) => b.to_string(),
            _ => field.default_value.to_string(),
        };

        if field.required && str_val.is_empty() {
            anyhow::bail!("Missing required parameter '{}' ({})", field.label, field.name);
        }

        if field.field_type == FieldType::Bool {
            let is_true = str_val.eq_ignore_ascii_case("true") || str_val == "1";
            if is_true && !field.flag.is_empty() {
                args.push(field.flag.to_string());
            }
            continue;
        }

        if str_val.is_empty() {
            continue;
        }

        if field.flag.is_empty() {
            positional.push(str_val);
        } else {
            args.push(field.flag.to_string());
            args.push(str_val);
        }
    }

    if let Some(sub) = tool.subcommand {
        args.push(sub.to_string());
    }
    args.extend(positional);
    Ok(args)
}

/// Looks up a [`ToolSpec`] by its subcommand ID.
pub fn find_tool(id: &str) -> Option<ToolSpec> {
    all_tools().into_iter().find(|t| t.id == id)
}

/// Validates catalog invariants (unique IDs, non-empty metadata).
pub fn validate_catalog() -> Result<(), String> {
    let tools = all_tools();
    let mut seen = HashSet::new();
    for t in &tools {
        if t.id.trim().is_empty() || t.name.trim().is_empty() || t.category.trim().is_empty() {
            return Err(format!("Invalid tool metadata for {:?}", t.id));
        }
        if !seen.insert(t.id) {
            return Err(format!("Duplicate tool id: {}", t.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_all_tools_and_unique_ids() {
        validate_catalog().expect("Catalog should be valid");
        let tools = all_tools();
        assert_eq!(tools.len(), 48);
    }

    #[test]
    fn test_build_cli_args_flags_and_positional() {
        let tool = find_tool("x32-desk-save").unwrap();
        let mut map = HashMap::new();
        map.insert("ip".into(), serde_json::json!("192.168.1.50"));
        map.insert("desk_save".into(), serde_json::json!(true));
        map.insert("scene".into(), serde_json::json!(false));
        map.insert("destination_file".into(), serde_json::json!("bak.txt"));
        let args = build_cli_args(&tool, &map).unwrap();
        assert_eq!(args[0], "x32-desk-save");
        assert!(args.windows(2).any(|w| w == ["--ip", "192.168.1.50"]));
        assert!(args.contains(&"--desk-save".to_string()));
        assert!(!args.contains(&"--scene".to_string()));
        assert_eq!(args.last().unwrap(), "bak.txt");
    }
}
