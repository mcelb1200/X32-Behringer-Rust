//! Shared mixer model limits, OSC path resolution, and fader-to-dB curve helpers.

use x32_fxparse::MixerModel;

/// Hardware channel/bus/matrix/DCA/FX limits for a digital mixer model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MixerLimits {
    /// Maximum input channels.
    pub channels: u32,
    /// Maximum mix buses.
    pub buses: u32,
    /// Maximum aux inputs.
    pub aux_ins: u32,
    /// Maximum matrices.
    pub matrices: u32,
    /// Maximum DCA groups.
    pub dcas: u32,
    /// Maximum FX slots.
    pub fx_slots: u32,
    /// Maximum parametric EQ bands per bus.
    pub eq_bands: u32,
}

/// Returns the hardware limits for the given [`MixerModel`].
pub fn limits_for_model(model: MixerModel) -> MixerLimits {
    match model {
        MixerModel::X32 => MixerLimits {
            channels: 32,
            buses: 16,
            aux_ins: 8,
            matrices: 6,
            dcas: 8,
            fx_slots: 8,
            eq_bands: 6,
        },
        MixerModel::Wing => MixerLimits {
            channels: 40,
            buses: 28,
            aux_ins: 16,
            matrices: 8,
            dcas: 16,
            fx_slots: 16,
            eq_bands: 8,
        },
        MixerModel::XR18 => MixerLimits {
            channels: 16,
            buses: 6,
            aux_ins: 2,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            eq_bands: 4,
        },
        MixerModel::XR16 => MixerLimits {
            channels: 16,
            buses: 4,
            aux_ins: 2,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            eq_bands: 4,
        },
        MixerModel::XR12 => MixerLimits {
            channels: 12,
            buses: 2,
            aux_ins: 2,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            eq_bands: 4,
        },
    }
}

/// Returns the default UDP OSC port for the given [`MixerModel`].
pub fn default_port_for_model(model: MixerModel) -> u16 {
    match model {
        MixerModel::X32 => 10023,
        MixerModel::Wing => 2222,
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 10024,
    }
}

/// Resolves an IP or `host:port` string, appending the model's default OSC port when omitted.
pub fn resolve_target_address(ip: &str, model: MixerModel) -> String {
    if (ip.contains(':') && !ip.starts_with('[')) || ip.contains("]:") {
        ip.to_string()
    } else {
        format!("{}:{}", ip, default_port_for_model(model))
    }
}

/// Converts a normalized `[0.0, 1.0]` fader float to decibels (`-90.0` to `+10.0` dB)
/// using the standard 4-segment Behringer X32 fader taper.
pub fn fader_to_db(fader: f32) -> f32 {
    let f = fader.clamp(0.0, 1.0);
    if f >= 0.5 {
        40.0 * f - 30.0
    } else if f >= 0.25 {
        80.0 * f - 50.0
    } else if f >= 0.0625 {
        160.0 * f - 70.0
    } else if f > 0.0 {
        480.0 * f - 90.0
    } else {
        -90.0
    }
}

/// Converts decibels (`-90.0` to `+10.0` dB) back to a normalized `[0.0, 1.0]` fader float.
pub fn db_to_fader(db: f32) -> f32 {
    let level = if db > -10.0 {
        (db + 30.0) / 40.0
    } else if db > -30.0 {
        (db + 50.0) / 80.0
    } else if db > -60.0 {
        (db + 70.0) / 160.0
    } else {
        (db + 90.0) / 480.0
    };
    level.clamp(0.0, 1.0)
}

/// Clamps a target fader float within `[0.0, 1.0]` and any configured `max_limit_db` ceiling.
pub fn clamp_fader_with_limit(fader: f32, max_limit_db: Option<f32>) -> f32 {
    let max_float = max_limit_db.map(db_to_fader).unwrap_or(1.0);
    fader.clamp(0.0, max_float)
}

/// Returns the model-accurate Main LR fader OSC path.
pub fn main_fader_path(model: MixerModel) -> &'static str {
    match model {
        MixerModel::X32 => "/main/st/mix/fader",
        MixerModel::Wing => "/main/1/fader",
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => "/lr/mix/fader",
    }
}

/// Returns the model-accurate Main LR mute/on OSC path.
pub fn main_mute_path(model: MixerModel) -> &'static str {
    match model {
        MixerModel::X32 => "/main/st/mix/on",
        MixerModel::Wing => "/main/1/mute",
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => "/lr/mix/on",
    }
}

/// Returns the model-accurate Main LR base OSC path (for `/fader` and `/on` suffixes).
pub fn main_mix_base_path(model: MixerModel) -> &'static str {
    match model {
        MixerModel::X32 => "/main/st/mix",
        MixerModel::Wing => "/main/1",
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => "/lr/mix",
    }
}

/// Returns the channel fader OSC path for the given model and 1-based channel number.
pub fn channel_fader_path(_model: MixerModel, ch: u8) -> String {
    format!("/ch/{:02}/mix/fader", ch)
}

/// Returns the channel mute/on OSC path for the given model and 1-based channel number.
pub fn channel_mute_path(_model: MixerModel, ch: u8) -> String {
    format!("/ch/{:02}/mix/on", ch)
}

/// Returns the mixbus master fader OSC path for the given model and 1-based bus number.
pub fn bus_fader_path(_model: MixerModel, bus: u8) -> String {
    format!("/bus/{:02}/mix/fader", bus)
}

/// Returns the mixbus master mute/on OSC path for the given model and 1-based bus number.
pub fn bus_mute_path(_model: MixerModel, bus: u8) -> String {
    format!("/bus/{:02}/mix/on", bus)
}

/// Returns the mixbus parametric EQ band gain OSC path (`/bus/XX/eq/{band}/g`).
pub fn bus_eq_gain_path(_model: MixerModel, bus: u8, band: u8) -> String {
    format!("/bus/{:02}/eq/{}/g", bus, band)
}

/// Returns the headamp gain OSC path for the given model and 1-based channel number.
pub fn headamp_gain_path(_model: MixerModel, ch: u8) -> String {
    format!("/headamp/{:02}/gain", ch)
}

/// Returns the headamp +48V phantom power OSC path for the given model and 1-based channel number.
pub fn headamp_phantom_path(_model: MixerModel, ch: u8) -> String {
    format!("/headamp/{:02}/+48V", ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limits_and_ports_across_models() {
        assert_eq!(limits_for_model(MixerModel::X32).channels, 32);
        assert_eq!(limits_for_model(MixerModel::Wing).channels, 40);
        assert_eq!(limits_for_model(MixerModel::XR18).buses, 6);
        assert_eq!(limits_for_model(MixerModel::XR16).buses, 4);
        assert_eq!(limits_for_model(MixerModel::XR12).channels, 12);

        assert_eq!(default_port_for_model(MixerModel::X32), 10023);
        assert_eq!(default_port_for_model(MixerModel::Wing), 2222);
        assert_eq!(default_port_for_model(MixerModel::XR18), 10024);
        assert_eq!(
            resolve_target_address("192.168.1.50", MixerModel::Wing),
            "192.168.1.50:2222"
        );
        assert_eq!(
            resolve_target_address("192.168.1.50:10025", MixerModel::X32),
            "192.168.1.50:10025"
        );
    }

    #[test]
    fn test_main_paths_across_models() {
        assert_eq!(main_fader_path(MixerModel::X32), "/main/st/mix/fader");
        assert_eq!(main_fader_path(MixerModel::Wing), "/main/1/fader");
        assert_eq!(main_fader_path(MixerModel::XR18), "/lr/mix/fader");

        assert_eq!(main_mute_path(MixerModel::X32), "/main/st/mix/on");
        assert_eq!(main_mute_path(MixerModel::Wing), "/main/1/mute");
        assert_eq!(main_mute_path(MixerModel::XR12), "/lr/mix/on");

        assert_eq!(bus_eq_gain_path(MixerModel::X32, 2, 3), "/bus/02/eq/3/g");
    }

    #[test]
    fn test_fader_db_roundtrip_and_clamp() {
        assert!((fader_to_db(1.0) - 10.0).abs() < 0.01);
        assert!((fader_to_db(0.75) - 0.0).abs() < 0.01);
        assert!((fader_to_db(0.5) - -10.0).abs() < 0.01);
        assert!((fader_to_db(0.25) - -30.0).abs() < 0.01);
        assert!((fader_to_db(0.0) - -90.0).abs() < 0.01);

        assert!((db_to_fader(0.0) - 0.75).abs() < 0.001);
        assert!((db_to_fader(-10.0) - 0.50).abs() < 0.001);
        assert!((db_to_fader(-30.0) - 0.25).abs() < 0.001);
        assert_eq!(db_to_fader(-90.0), 0.0);
        assert_eq!(db_to_fader(10.0), 1.0);

        let clamped = clamp_fader_with_limit(0.95, Some(-3.0));
        assert!((clamped - db_to_fader(-3.0)).abs() < 0.001);
    }
}
