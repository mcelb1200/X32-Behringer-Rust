//! Approximate microphone frequency compensation and cut-only guided room tuning.

use crate::automix::eq_gain_db_to_osc;
use crate::meters::{linear_to_dbfs, rta_bin_to_hz};
use crate::model::eq_freq_hz_to_osc;

/// Published approximate datasheet profile and stage-safety guardrails for a microphone model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicModelProfile {
    /// Short CLI/config identifier (e.g., `"sm58"`, `"e835"`, `"c747"`, `"m5"`, `"c5"`).
    pub id: &'static str,
    /// Human-readable brand and model name.
    pub display_name: &'static str,
    /// Microphone capsule/pattern category.
    pub category: &'static str,
    /// Whether `+48V` phantom power is required on the measurement channel.
    pub requires_phantom_48v: bool,
    /// Lowest reliable frequency (Hz) for automated room EQ evaluation.
    pub valid_min_hz: f32,
    /// Highest reliable frequency (Hz) for automated room EQ evaluation.
    pub valid_max_hz: f32,
    /// Default maximum automated cut depth in dB (positive magnitude, e.g. `5.0` = max `-5.0 dB`).
    pub max_auto_cut_db: f32,
    /// Practical aiming/placement guidance for volunteer operators.
    pub placement_hint: &'static str,
    /// Approximate datasheet `(frequency_hz, deviation_db)` knots subtracted from raw RTA bins.
    pub response_knots_db: &'static [(f32, f32)],
}

/// Cut-only Main/Bus parametric EQ recommendation derived from compensated RTA peaks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PeqCutSuggestion {
    /// Center frequency in Hz.
    pub freq_hz: f32,
    /// Cut gain in dB (strictly negative, e.g. `-4.5`).
    pub gain_db: f32,
    /// Filter Q factor.
    pub q: f32,
    /// Normalized `[0.0, 1.0]` OSC frequency value.
    pub osc_freq: f32,
    /// Normalized `[0.0, 1.0]` OSC gain value (`0.5` = `0 dB`).
    pub osc_gain: f32,
    /// Normalized `[0.0, 1.0]` OSC Q value.
    pub osc_q: f32,
}

const SM58_KNOTS: &[(f32, f32)] = &[
    (50.0, -8.0),
    (80.0, -3.5),
    (120.0, -1.0),
    (250.0, 0.0),
    (1000.0, 0.0),
    (2500.0, 2.0),
    (4500.0, 4.0),
    (7000.0, 2.5),
    (10000.0, -1.0),
];

const BETA58_KNOTS: &[(f32, f32)] = &[
    (50.0, -7.0),
    (90.0, -2.5),
    (200.0, 0.0),
    (1000.0, 0.0),
    (4000.0, 5.0),
    (6500.0, 3.0),
    (9000.0, 4.0),
    (12000.0, -1.5),
];

const SM57_KNOTS: &[(f32, f32)] = &[
    (50.0, -9.0),
    (100.0, -4.0),
    (200.0, -0.5),
    (1000.0, 0.0),
    (3000.0, 2.5),
    (5500.0, 5.5),
    (8000.0, 2.0),
    (12000.0, -2.0),
];

const E835_KNOTS: &[(f32, f32)] = &[
    (50.0, -6.5),
    (85.0, -2.0),
    (200.0, 0.0),
    (1000.0, 0.0),
    (4500.0, 3.5),
    (8000.0, 2.5),
    (12000.0, -1.0),
];

const E935_KNOTS: &[(f32, f32)] = &[
    (50.0, -6.5),
    (85.0, -2.0),
    (200.0, -0.5),
    (1000.0, 0.0),
    (5000.0, 4.0),
    (8000.0, 2.5),
    (12000.0, -0.5),
];

const D5_KNOTS: &[(f32, f32)] = &[
    (50.0, -7.5),
    (100.0, 1.0),
    (250.0, 0.0),
    (1000.0, 0.0),
    (4500.0, 4.0),
    (8000.0, 2.0),
    (12000.0, -1.5),
];

const OM2_KNOTS: &[(f32, f32)] = &[
    (50.0, -10.0),
    (120.0, -3.5),
    (250.0, 0.0),
    (1000.0, 0.0),
    (5500.0, 4.0),
    (8000.0, 1.5),
    (12000.0, -2.5),
];

const C747_KNOTS: &[(f32, f32)] = &[
    (40.0, -5.0),
    (80.0, -1.5),
    (200.0, 0.0),
    (2000.0, 0.0),
    (6000.0, 3.5),
    (9000.0, 4.5),
    (12000.0, 2.0),
];

const MX418_KNOTS: &[(f32, f32)] = &[
    (50.0, -9.0),
    (120.0, -3.0),
    (250.0, 0.0),
    (2000.0, 0.0),
    (6000.0, 4.0),
    (9000.0, 3.0),
    (12000.0, -1.0),
];

const M5_KNOTS: &[(f32, f32)] = &[
    (40.0, -2.5),
    (70.0, -0.5),
    (200.0, 0.0),
    (4000.0, 0.5),
    (9000.0, 2.0),
    (14000.0, 1.5),
];

const NT5_KNOTS: &[(f32, f32)] = &[
    (40.0, -2.5),
    (70.0, -0.5),
    (200.0, 0.0),
    (4000.0, 0.5),
    (9000.0, 2.0),
    (14000.0, 1.0),
];

const C5_KNOTS: &[(f32, f32)] = &[
    (40.0, -5.0),
    (80.0, -1.5),
    (200.0, 0.0),
    (3000.0, 0.5),
    (8500.0, 3.5),
    (12000.0, 2.0),
];

const AT2020_KNOTS: &[(f32, f32)] = &[
    (40.0, -3.0),
    (70.0, -0.5),
    (200.0, 0.0),
    (3000.0, 0.5),
    (8000.0, 2.5),
    (12000.0, 2.0),
];

const ECM8000_KNOTS: &[(f32, f32)] = &[
    (20.0, -0.5),
    (40.0, 0.0),
    (4000.0, 0.0),
    (8000.0, 2.5),
    (12000.0, 2.0),
];

const LAPTOP_KNOTS: &[(f32, f32)] = &[
    (60.0, -14.0),
    (160.0, -4.0),
    (350.0, 0.0),
    (2500.0, 3.0),
    (5500.0, -2.0),
    (10000.0, -10.0),
];

const FLAT_KNOTS: &[(f32, f32)] = &[(20.0, 0.0), (20000.0, 0.0)];

/// Built-in 16-model microphone library covering common stage, lectern, studio, and measurement mics.
pub const MIC_PROFILES: &[MicModelProfile] = &[
    MicModelProfile {
        id: "sm58",
        display_name: "Shure SM58",
        category: "Cardioid Dynamic",
        requires_phantom_48v: false,
        valid_min_hz: 90.0,
        valid_max_hz: 7000.0,
        max_auto_cut_db: 5.0,
        placement_hint: "Hold at arm's length aimed ~45 deg upward between L & R speakers.",
        response_knots_db: SM58_KNOTS,
    },
    MicModelProfile {
        id: "beta58",
        display_name: "Shure Beta 58A",
        category: "Supercardioid Dynamic",
        requires_phantom_48v: false,
        valid_min_hz: 90.0,
        valid_max_hz: 7000.0,
        max_auto_cut_db: 5.0,
        placement_hint: "Aim halfway between Main L & R; avoid pointing directly at floor.",
        response_knots_db: BETA58_KNOTS,
    },
    MicModelProfile {
        id: "sm57",
        display_name: "Shure SM57",
        category: "Cardioid Dynamic",
        requires_phantom_48v: false,
        valid_min_hz: 110.0,
        valid_max_hz: 7000.0,
        max_auto_cut_db: 5.0,
        placement_hint: "Hold at ear height pointed toward center between L & R speakers.",
        response_knots_db: SM57_KNOTS,
    },
    MicModelProfile {
        id: "e835",
        display_name: "Sennheiser e835",
        category: "Cardioid Dynamic",
        requires_phantom_48v: false,
        valid_min_hz: 85.0,
        valid_max_hz: 8000.0,
        max_auto_cut_db: 5.0,
        placement_hint: "Hold at ear height aimed ~30-45 deg up between L & R speakers.",
        response_knots_db: E835_KNOTS,
    },
    MicModelProfile {
        id: "e935",
        display_name: "Sennheiser e935 / e945",
        category: "Cardioid/Supercardioid Dynamic",
        requires_phantom_48v: false,
        valid_min_hz: 85.0,
        valid_max_hz: 8000.0,
        max_auto_cut_db: 5.0,
        placement_hint: "Aim evenly between Main L & R; walk 3-5 listener spots.",
        response_knots_db: E935_KNOTS,
    },
    MicModelProfile {
        id: "d5",
        display_name: "AKG D5",
        category: "Supercardioid Dynamic",
        requires_phantom_48v: false,
        valid_min_hz: 90.0,
        valid_max_hz: 7000.0,
        max_auto_cut_db: 5.0,
        placement_hint: "Hold at arm's length aimed midway between L & R mains.",
        response_knots_db: D5_KNOTS,
    },
    MicModelProfile {
        id: "om2",
        display_name: "Audix OM2 / OM5",
        category: "Hypercardioid Dynamic",
        requires_phantom_48v: false,
        valid_min_hz: 120.0,
        valid_max_hz: 7000.0,
        max_auto_cut_db: 4.5,
        placement_hint: "Tight pattern: aim directly toward center of PA coverage.",
        response_knots_db: OM2_KNOTS,
    },
    MicModelProfile {
        id: "c747",
        display_name: "AKG C747",
        category: "Hypercardioid Condenser",
        requires_phantom_48v: true,
        valid_min_hz: 80.0,
        valid_max_hz: 10000.0,
        max_auto_cut_db: 6.0,
        placement_hint: "Requires +48V; very directional - aim directly toward main PA.",
        response_knots_db: C747_KNOTS,
    },
    MicModelProfile {
        id: "mx418",
        display_name: "Shure MX412 / MX418",
        category: "Gooseneck Condenser",
        requires_phantom_48v: true,
        valid_min_hz: 120.0,
        valid_max_hz: 8000.0,
        max_auto_cut_db: 5.0,
        placement_hint:
            "Requires +48V; angle gooseneck toward room center, away from stage wedges.",
        response_knots_db: MX418_KNOTS,
    },
    MicModelProfile {
        id: "m5",
        display_name: "Rode M5",
        category: "Small-Diaphragm Cardioid Condenser",
        requires_phantom_48v: true,
        valid_min_hz: 60.0,
        valid_max_hz: 12000.0,
        max_auto_cut_db: 6.0,
        placement_hint: "Requires +48V; place at ear height aimed ~45 deg up between L & R mains.",
        response_knots_db: M5_KNOTS,
    },
    MicModelProfile {
        id: "nt5",
        display_name: "Rode NT5 / Shure SM81",
        category: "Small-Diaphragm Cardioid Condenser",
        requires_phantom_48v: true,
        valid_min_hz: 60.0,
        valid_max_hz: 12000.0,
        max_auto_cut_db: 6.0,
        placement_hint: "Requires +48V; place on stand at ear height aimed between L & R.",
        response_knots_db: NT5_KNOTS,
    },
    MicModelProfile {
        id: "c5",
        display_name: "Behringer C-2 / C5",
        category: "Small-Diaphragm Cardioid Condenser",
        requires_phantom_48v: true,
        valid_min_hz: 75.0,
        valid_max_hz: 10000.0,
        max_auto_cut_db: 5.5,
        placement_hint: "Requires +48V (disable mic HPF switch if present); aim between L & R.",
        response_knots_db: C5_KNOTS,
    },
    MicModelProfile {
        id: "at2020",
        display_name: "Audio-Technica AT2020",
        category: "Large-Diaphragm Condenser",
        requires_phantom_48v: true,
        valid_min_hz: 60.0,
        valid_max_hz: 10000.0,
        max_auto_cut_db: 5.5,
        placement_hint: "Requires +48V; face front address emblem toward PA center.",
        response_knots_db: AT2020_KNOTS,
    },
    MicModelProfile {
        id: "ecm8000",
        display_name: "Behringer ECM8000 (Uncalibrated)",
        category: "Omni Measurement Electret",
        requires_phantom_48v: true,
        valid_min_hz: 40.0,
        valid_max_hz: 10000.0,
        max_auto_cut_db: 6.0,
        placement_hint: "Requires +48V; point tip straight up at ear height across 3-5 seats.",
        response_knots_db: ECM8000_KNOTS,
    },
    MicModelProfile {
        id: "laptop",
        display_name: "Laptop / Phone Mic",
        category: "Consumer MEMS",
        requires_phantom_48v: false,
        valid_min_hz: 160.0,
        valid_max_hz: 5500.0,
        max_auto_cut_db: 4.0,
        placement_hint: "Conservative boom/honk cuts only; place away from laptop fan noise.",
        response_knots_db: LAPTOP_KNOTS,
    },
    MicModelProfile {
        id: "flat",
        display_name: "Calibrated Measurement Mic",
        category: "Calibrated Omni",
        requires_phantom_48v: true,
        valid_min_hz: 40.0,
        valid_max_hz: 12000.0,
        max_auto_cut_db: 6.0,
        placement_hint:
            "Requires +48V; point straight up or at 90 deg grazing per calibration file.",
        response_knots_db: FLAT_KNOTS,
    },
];

/// Returns all built-in [`MicModelProfile`] definitions.
pub fn list_mic_profiles() -> &'static [MicModelProfile] {
    MIC_PROFILES
}

/// Finds a [`MicModelProfile`] by ID or common alias (case-insensitive).
pub fn find_mic_profile(id: &str) -> Option<&'static MicModelProfile> {
    let norm = id.trim().to_ascii_lowercase();
    let canonical = match norm.as_str() {
        "c-2" | "c2" | "behringer-c2" | "behringer-c5" => "c5",
        "rode-m5" | "rodem5" => "m5",
        "beta58a" | "beta-58" | "beta-58a" => "beta58",
        "e945" => "e935",
        "sm81" => "nt5",
        "mx412" => "mx418",
        "om5" => "om2",
        "phone" | "mems" => "laptop",
        other => other,
    };
    MIC_PROFILES.iter().find(|p| p.id == canonical)
}

/// Interpolates a microphone's approximate dB deviation at `hz` in log-frequency space.
pub fn interpolate_mic_response_db(profile: &MicModelProfile, hz: f32) -> f32 {
    let knots = profile.response_knots_db;
    if knots.is_empty() {
        return 0.0;
    }
    if hz <= knots[0].0 {
        return knots[0].1;
    }
    let last = knots[knots.len() - 1];
    if hz >= last.0 {
        return last.1;
    }

    let target_l = hz.max(1.0).log10();
    for pair in knots.windows(2) {
        let (f0, d0) = pair[0];
        let (f1, d1) = pair[1];
        if hz >= f0 && hz <= f1 {
            let l0 = f0.max(1.0).log10();
            let l1 = f1.max(1.0).log10();
            let span = (l1 - l0).max(1e-6);
            let t = ((target_l - l0) / span).clamp(0.0, 1.0);
            return d0 + t * (d1 - d0);
        }
    }
    0.0
}

/// Averages multiple spatial `/meters/4` linear captures element-wise (`Walk the Room` averaging).
pub fn average_rta_captures(captures: &[Vec<f32>]) -> Vec<f32> {
    if captures.is_empty() {
        return Vec::new();
    }
    let len = captures.iter().map(|c| c.len()).min().unwrap_or(0);
    if len == 0 {
        return Vec::new();
    }
    let count = captures.len() as f32;
    let mut avg = vec![0.0f32; len];
    for cap in captures {
        for i in 0..len {
            avg[i] += cap[i];
        }
    }
    for v in &mut avg {
        *v /= count;
    }
    avg
}

/// Converts raw linear RTA bins to dBFS, removing both the mic's curve and a `-0.8 dB/oct` room house tilt.
pub fn compensate_rta_bins_db(profile: &MicModelProfile, raw_linear_bins: &[f32]) -> Vec<f32> {
    let n = raw_linear_bins.len();
    let mut out = Vec::with_capacity(n);
    for (i, &val) in raw_linear_bins.iter().enumerate() {
        let hz = rta_bin_to_hz(i, n);
        let raw_db = linear_to_dbfs(val);
        let mic_dev_db = interpolate_mic_response_db(profile, hz);
        // Gentle -0.8 dB/oct live-room house curve centered at 1 kHz.
        let house_tilt_db = -0.8 * (hz / 1000.0).log2();
        out.push(raw_db - mic_dev_db - house_tilt_db);
    }
    out
}

/// Detects the top `max_bands` cut-only PEQ notches strictly within `profile.valid_min_hz..=valid_max_hz`.
pub fn suggest_cut_only_peq(
    averaged_linear_bins: &[f32],
    profile: &MicModelProfile,
    max_bands: usize,
    max_cut_override_db: Option<f32>,
) -> Vec<PeqCutSuggestion> {
    let n = averaged_linear_bins.len();
    if n < 8 || max_bands == 0 {
        return Vec::new();
    }

    let comp_db = compensate_rta_bins_db(profile, averaged_linear_bins);
    let mut in_band: Vec<(usize, f32, f32)> = comp_db
        .iter()
        .enumerate()
        .filter_map(|(i, &db)| {
            let hz = rta_bin_to_hz(i, n);
            if hz >= profile.valid_min_hz && hz <= profile.valid_max_hz {
                Some((i, hz, db))
            } else {
                None
            }
        })
        .collect();

    if in_band.len() < 3 {
        return Vec::new();
    }

    let mut sorted_dbs: Vec<f32> = in_band.iter().map(|&(_, _, db)| db).collect();
    sorted_dbs.sort_by(|a, b| a.total_cmp(b));
    let median_db = sorted_dbs[sorted_dbs.len() / 2];

    let max_cut = max_cut_override_db
        .unwrap_or(profile.max_auto_cut_db)
        .abs()
        .clamp(1.0, 9.0);

    // Local peaks exceeding median + 3.0 dB
    let mut peaks: Vec<(f32, f32)> = Vec::new();
    for w in 1..(in_band.len() - 1) {
        let (_, hz, db) = in_band[w];
        let prev = in_band[w - 1].2;
        let next = in_band[w + 1].2;
        let excess = db - median_db;
        if db >= prev && db >= next && excess > 3.0 {
            peaks.push((hz, excess));
        }
    }
    // Include highest peak if all were monotonic edges
    in_band.clear();
    peaks.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut chosen: Vec<PeqCutSuggestion> = Vec::new();
    for (hz, excess) in peaks {
        if chosen.len() >= max_bands {
            break;
        }
        let too_close = chosen.iter().any(|c| (hz / c.freq_hz).log2().abs() < 0.35);
        if too_close {
            continue;
        }
        let cut_mag = (excess * 0.75).clamp(1.0, max_cut);
        let gain_db = -cut_mag;
        let q = 3.0;
        chosen.push(PeqCutSuggestion {
            freq_hz: hz,
            gain_db,
            q,
            osc_freq: eq_freq_hz_to_osc(hz),
            osc_gain: eq_gain_db_to_osc(gain_db),
            osc_q: 0.6,
        });
    }

    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mic_library_contains_requested_models_and_aliases() {
        assert!(list_mic_profiles().len() >= 16);
        for id in [
            "sm58", "e835", "c747", "ecm8000", "m5", "c5", "c-2", "rode-m5", "nt5", "laptop",
        ] {
            assert!(find_mic_profile(id).is_some(), "Missing profile {}", id);
        }
        assert_eq!(find_mic_profile("C-2").unwrap().id, "c5");
        assert_eq!(find_mic_profile("Rode-M5").unwrap().id, "m5");
        assert!(find_mic_profile("m5").unwrap().requires_phantom_48v);
        assert!(find_mic_profile("c5").unwrap().requires_phantom_48v);
        assert!(!find_mic_profile("sm58").unwrap().requires_phantom_48v);
    }

    #[test]
    fn test_sm58_compensation_prevents_false_presence_cut() {
        let sm58 = find_mic_profile("sm58").unwrap();
        let n = 100;
        let mut bins = vec![0.1f32; n];
        // Shape bins according to SM58 response (+ house tilt) so a flat room shouldn't trigger cuts.
        for i in 0..n {
            let hz = rta_bin_to_hz(i, n);
            let dev = interpolate_mic_response_db(sm58, hz) - 0.8 * (hz / 1000.0).log2();
            bins[i] = 0.1 * 10.0f32.powf(dev / 20.0);
        }
        let cuts = suggest_cut_only_peq(&bins, sm58, 4, None);
        assert!(
            cuts.is_empty(),
            "Expected no cuts on compensated flat room, got {:?}",
            cuts
        );
    }

    #[test]
    fn test_guardrail_band_and_cut_only_limits() {
        let n = 100;
        let mut bins = vec![0.1f32; n];
        // Find bins nearest 50 Hz and 250 Hz
        let bin_50 = (0..n)
            .min_by(|&a, &b| {
                (rta_bin_to_hz(a, n) - 50.0)
                    .abs()
                    .total_cmp(&(rta_bin_to_hz(b, n) - 50.0).abs())
            })
            .unwrap();
        let bin_250 = (0..n)
            .min_by(|&a, &b| {
                (rta_bin_to_hz(a, n) - 250.0)
                    .abs()
                    .total_cmp(&(rta_bin_to_hz(b, n) - 250.0).abs())
            })
            .unwrap();

        // Inject +10 dB boom at 50 Hz and +9 dB boom at 250 Hz
        bins[bin_50] = 0.35;
        bins[bin_250] = 0.32;

        let sm58 = find_mic_profile("sm58").unwrap();
        let cuts_sm58 = suggest_cut_only_peq(&bins, sm58, 4, None);
        // SM58 ignores <90 Hz so 50 Hz must NOT be cut, 250 Hz SHOULD be cut <= -5.0 dB
        assert!(cuts_sm58.iter().all(|c| c.freq_hz >= 90.0));
        assert!(cuts_sm58
            .iter()
            .any(|c| (c.freq_hz - 250.0).abs() < 40.0 && c.gain_db < 0.0 && c.gain_db >= -5.0));

        let ecm = find_mic_profile("ecm8000").unwrap();
        let cuts_ecm = suggest_cut_only_peq(&bins, ecm, 4, None);
        // ECM8000 extends down to 40 Hz so 50 Hz IS caught
        assert!(cuts_ecm.iter().any(|c| (c.freq_hz - 50.0).abs() < 15.0));
    }

    #[test]
    fn test_average_rta_captures() {
        let avg = average_rta_captures(&[vec![0.1, 0.5], vec![0.3, 0.7]]);
        assert_eq!(avg.len(), 2);
        assert!((avg[0] - 0.2).abs() < 1e-6);
        assert!((avg[1] - 0.6).abs() < 1e-6);
    }
}
