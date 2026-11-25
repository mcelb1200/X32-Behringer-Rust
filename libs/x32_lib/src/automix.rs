//! Dugan-style channel automixing and spectrum-aware mixbus balancing with dynamic vocal pocket EQ.

use crate::meters::{RtaBands, RtaSnapshot};
use crate::model::{clamp_fader_with_limit, db_to_fader, eq_freq_hz_to_osc, fader_to_db};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Calculates Dugan gain-sharing fader targets (`0.0..=1.0`) for input channels using a stack-allocated buffer.
pub fn calculate_dugan_gains(levels: &[f32], noise_floor: f32, gains_out: &mut [f32]) {
    let mut weights = [0.0f32; 40];
    let mut sum_weights = 0.0f32;
    let count = levels.len().min(40).min(gains_out.len());

    for i in 0..count {
        let level = levels[i];
        if level > noise_floor {
            let db = fader_to_db(level);
            let weight = 10.0_f32.powf(db / 20.0);
            weights[i] = weight;
            sum_weights += weight;
        } else {
            weights[i] = 0.0;
        }
    }

    for i in 0..count {
        let weight = weights[i];
        if sum_weights > 0.0 && weight > 0.0 {
            let gain_linear = weight / sum_weights;
            let gain_db = 20.0 * gain_linear.log10();
            gains_out[i] = db_to_fader(gain_db);
        } else {
            gains_out[i] = 0.0;
        }
    }
}

/// Musical/spectral role of a mixbus used to weight Dugan competition and transient smoothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpectralRole {
    /// Lead or backing vocals (800 Hz – 4 kHz priority).
    Vocals,
    /// Spoken word / pulpit / host mic (high intelligibility priority).
    Speech,
    /// Full harmonic band instruments (guitars, pads, keys).
    Band,
    /// Drum kit (bass-heavy kick transients + high-end cymbals).
    Drums,
    /// Bass guitar / synth bass (40–200 Hz).
    Bass,
    /// Keyboards / piano (250 Hz – 3 kHz).
    Keys,
    /// Generic bus profile.
    Custom,
}

impl SpectralRole {
    /// Compensation multiplier reducing sub/kick transient bias on broadband meters.
    pub fn low_end_compensation(self) -> f32 {
        match self {
            Self::Drums => 0.45,
            Self::Bass => 0.55,
            Self::Band => 0.85,
            Self::Keys => 0.90,
            Self::Vocals | Self::Speech | Self::Custom => 1.0,
        }
    }

    /// Returns `(attack_coef, release_coef)` smoothing parameters appropriate for this role.
    pub fn envelope_coefs(self) -> (f32, f32) {
        match self {
            Self::Drums | Self::Bass => (0.35, 0.15),
            Self::Vocals | Self::Speech => (0.80, 0.20),
            Self::Band | Self::Keys | Self::Custom => (0.60, 0.18),
        }
    }

    /// Pairwise spectral overlap coefficient in `[0.0, 1.0]` between two roles.
    pub fn overlap(a: Self, b: Self) -> f32 {
        use SpectralRole::*;
        if a == b {
            return 1.0;
        }
        match (a, b) {
            (Vocals | Speech, Band) | (Band, Vocals | Speech) => 0.85,
            (Vocals | Speech, Keys) | (Keys, Vocals | Speech) => 0.80,
            (Vocals | Speech, Drums) | (Drums, Vocals | Speech) => 0.20,
            (Vocals | Speech, Bass) | (Bass, Vocals | Speech) => 0.15,
            (Drums, Bass) | (Bass, Drums) => 0.75,
            (Band, Keys) | (Keys, Band) => 0.70,
            (Band, Drums) | (Drums, Band) => 0.45,
            (Band, Bass) | (Bass, Band) => 0.40,
            _ => 0.50,
        }
    }

    /// Returns `true` for foreground vocal/speech roles that trigger pocket EQ carving.
    pub fn is_vocal_lead(self) -> bool {
        matches!(self, Self::Vocals | Self::Speech)
    }
}

/// Optional dynamic EQ "vocal pocket" dip applied to a competing bus when vocals are active.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PocketEqConfig {
    /// Parametric EQ band index (`1..=6` on X32, `1..=8` on Wing, `1..=4` on X-Air).
    pub band: u8,
    /// Maximum midrange dip in dB (positive magnitude, e.g., `3.0` dips up to `-3.0 dB`).
    pub max_dip_db: f32,
    /// Whether to slew the band's center frequency toward `/meters/4` RTA vocal peak (`800..=3500 Hz`).
    #[serde(default)]
    pub adaptive_freq: bool,
}

/// Converts an EQ gain in dB (`-15.0` to `+15.0`) to a normalized `[0.0, 1.0]` OSC float (`0.5` = `0 dB`).
pub fn eq_gain_db_to_osc(db: f32) -> f32 {
    ((db.clamp(-15.0, 15.0) + 15.0) / 30.0).clamp(0.0, 1.0)
}

/// Configuration for a single mixbus managed by [`SpectrumAutomixer`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusAutomixSpec {
    /// 1-based mixbus number.
    pub bus: u8,
    /// Human-readable label (e.g., `"Vox"`, `"Band"`, `"Drums"`).
    pub label: String,
    /// Spectral role profile.
    pub role: SpectralRole,
    /// Priority multiplier (higher = holds level more strongly, e.g., `1.5` for Vocals).
    pub priority: f32,
    /// Target baseline fader level in dB (`-90.0` to `+10.0`).
    pub base_db: f32,
    /// Optional dynamic EQ pocket configuration for this bus.
    #[serde(default)]
    pub pocket_eq: Option<PocketEqConfig>,
}

/// Delta-throttled fader and dynamic EQ updates produced by [`SpectrumAutomixer::step`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AutomixStep {
    /// `(bus_num, target_fader_float)` updates.
    pub fader_updates: Vec<(u8, f32)>,
    /// `(bus_num, eq_band, target_osc_gain_float)` updates.
    pub eq_updates: Vec<(u8, u8, f32)>,
    /// `(bus_num, eq_band, target_osc_freq_float)` updates when RTA `adaptive_freq` slews center frequency.
    pub eq_freq_updates: Vec<(u8, u8, f32)>,
}

/// Spectrum-aware mixbus automixer combining role-weighted Dugan gain sharing and vocal pocket EQ carving.
#[derive(Debug, Clone)]
pub struct SpectrumAutomixer {
    buses: Vec<BusAutomixSpec>,
    smoothed_levels: Vec<f32>,
    last_faders: Vec<f32>,
    last_eq_db: Vec<f32>,
    last_eq_freq_hz: Vec<f32>,
    max_cut_db: f32,
    max_boost_db: f32,
    noise_floor: f32,
    rta_hint: Option<RtaBands>,
    rta_snapshot: Option<RtaSnapshot>,
    rta_updated_at: Option<Instant>,
}

impl SpectrumAutomixer {
    /// Creates a new [`SpectrumAutomixer`] with bounded fader cut/boost limits and noise floor threshold.
    pub fn new(
        buses: Vec<BusAutomixSpec>,
        max_cut_db: f32,
        max_boost_db: f32,
        noise_floor: f32,
    ) -> Self {
        let n = buses.len();
        let last_faders = buses.iter().map(|b| db_to_fader(b.base_db)).collect();
        Self {
            buses,
            smoothed_levels: vec![0.0; n],
            last_faders,
            last_eq_db: vec![0.0; n],
            last_eq_freq_hz: vec![2000.0; n],
            max_cut_db: max_cut_db.abs(),
            max_boost_db: max_boost_db.abs(),
            noise_floor: noise_floor.max(1e-5),
            rta_hint: None,
            rta_snapshot: None,
            rta_updated_at: None,
        }
    }

    /// Returns the configured bus specs.
    pub fn buses(&self) -> &[BusAutomixSpec] {
        &self.buses
    }

    /// Supplies optional 4-band `/meters/4` RTA telemetry for adaptive overlap scaling.
    pub fn set_rta_hint(&mut self, rta: Option<RtaBands>) {
        self.rta_hint = rta;
        self.rta_updated_at = rta.map(|_| Instant::now());
    }

    /// Supplies a full `/meters/4` [`RtaSnapshot`] (4-band averages + vocal peak frequency).
    pub fn set_rta_snapshot(&mut self, snap: Option<RtaSnapshot>) {
        self.rta_hint = snap.map(|s| s.bands);
        self.rta_snapshot = snap;
        self.rta_updated_at = snap.map(|_| Instant::now());
    }

    /// Returns `true` if fresh `/meters/4` RTA telemetry was received within the last 500 ms.
    pub fn is_rta_active(&self) -> bool {
        self.rta_updated_at
            .map(|t| t.elapsed() < Duration::from_millis(500))
            .unwrap_or(false)
    }

    /// Advances the automixer by one meter frame (`raw_levels` aligned to `buses`) and emits throttled updates.
    pub fn step(&mut self, raw_levels: &[f32], max_fader_limit_db: Option<f32>) -> AutomixStep {
        let n = self.buses.len().min(raw_levels.len());
        let mut out = AutomixStep::default();
        if n == 0 {
            return out;
        }

        // 1. Role-specific envelope smoothing + low-end transient compensation.
        let mut eff_energy = vec![0.0f32; n];
        let mut vocal_activity = 0.0f32;

        for i in 0..n {
            let (atk, rel) = self.buses[i].role.envelope_coefs();
            let raw = raw_levels[i].clamp(0.0, 1.0);
            let coef = if raw > self.smoothed_levels[i] {
                atk
            } else {
                rel
            };
            self.smoothed_levels[i] = self.smoothed_levels[i] * (1.0 - coef) + raw * coef;

            let level = self.smoothed_levels[i];
            if level > self.noise_floor {
                let comp = self.buses[i].role.low_end_compensation();
                let prio = self.buses[i].priority.max(0.1);
                eff_energy[i] = level * comp * prio;
                if self.buses[i].role.is_vocal_lead() {
                    let norm = ((level - self.noise_floor) / (0.25 - self.noise_floor).max(0.05))
                        .clamp(0.0, 1.0);
                    vocal_activity = vocal_activity.max(norm);
                }
            }
        }

        // Active RTA adjustments (automatically fall back to static profiles if stale >500ms).
        let rta_active = self.is_rta_active();
        let active_bands = if rta_active { self.rta_hint } else { None };
        let active_peak_hz = if rta_active {
            self.rta_snapshot.and_then(|s| s.vocal_peak_hz)
        } else {
            None
        };

        let rta_vocal_boost = active_bands
            .map(|r| {
                if r.vocal_mid > 0.01 && r.sub_bass > r.vocal_mid * 2.0 {
                    1.15
                } else {
                    1.0
                }
            })
            .unwrap_or(1.0);
        let rta_low_mid_mask = active_bands
            .map(|r| {
                if r.vocal_mid > 0.01 && r.low_mid > r.vocal_mid * 1.5 {
                    1.15
                } else {
                    1.0
                }
            })
            .unwrap_or(1.0);

        // 2. Pairwise spectrum-weighted Dugan competition around each bus's base_db.
        for i in 0..n {
            let bus = &self.buses[i];
            let target_db = if eff_energy[i] <= 0.0 {
                bus.base_db
            } else {
                let mut competing_sum = eff_energy[i];
                for j in 0..n {
                    if i != j && eff_energy[j] > 0.0 {
                        let ov = SpectralRole::overlap(bus.role, self.buses[j].role);
                        let weight_j = if self.buses[j].role.is_vocal_lead() {
                            eff_energy[j] * rta_vocal_boost
                        } else {
                            eff_energy[j]
                        };
                        competing_sum += weight_j * ov;
                    }
                }
                let share = (eff_energy[i] / competing_sum.max(1e-6)).clamp(0.01, 1.0);
                let delta_db =
                    (20.0 * (share / 0.55).log10()).clamp(-self.max_cut_db, self.max_boost_db);
                bus.base_db + delta_db
            };

            let target_fader = clamp_fader_with_limit(db_to_fader(target_db), max_fader_limit_db);
            if (target_fader - self.last_faders[i]).abs() > 0.01 {
                self.last_faders[i] = target_fader;
                out.fader_updates.push((bus.bus, target_fader));
            }

            // 3. Dynamic EQ vocal pocket carving on competing buses.
            if let Some(ref eq) = bus.pocket_eq {
                let max_dip = eq.max_dip_db.abs();
                let target_eq_db = if !bus.role.is_vocal_lead() && vocal_activity > 0.0 {
                    let ov = SpectralRole::overlap(SpectralRole::Vocals, bus.role);
                    let mask_scale = if matches!(bus.role, SpectralRole::Band | SpectralRole::Keys)
                    {
                        rta_low_mid_mask
                    } else {
                        1.0
                    };
                    -(max_dip * vocal_activity * ov * mask_scale).min(max_dip)
                } else {
                    0.0
                };
                let eq_coef = if target_eq_db < self.last_eq_db[i] {
                    0.45
                } else {
                    0.20
                };
                let next_eq_db = self.last_eq_db[i] * (1.0 - eq_coef) + target_eq_db * eq_coef;
                let snapped = if next_eq_db.abs() < 0.08 && target_eq_db == 0.0 {
                    0.0
                } else {
                    next_eq_db
                };
                if (snapped - self.last_eq_db[i]).abs() > 0.15
                    || (snapped == 0.0 && self.last_eq_db[i] != 0.0)
                {
                    self.last_eq_db[i] = snapped;
                    out.eq_updates
                        .push((bus.bus, eq.band, eq_gain_db_to_osc(snapped)));
                }

                // 4. Optional RTA adaptive pocket center-frequency slewing (+/-5% per step, 150 Hz deadband).
                if eq.adaptive_freq && !bus.role.is_vocal_lead() && vocal_activity > 0.1 {
                    if let Some(peak_hz) = active_peak_hz {
                        let target_hz = peak_hz.clamp(800.0, 3500.0);
                        let cur_hz = self.last_eq_freq_hz[i];
                        if (target_hz - cur_hz).abs() > 150.0 {
                            let max_step = cur_hz * 0.05;
                            let next_hz = if target_hz > cur_hz {
                                (cur_hz + max_step).min(target_hz)
                            } else {
                                (cur_hz - max_step).max(target_hz)
                            };
                            self.last_eq_freq_hz[i] = next_hz;
                            out.eq_freq_updates.push((
                                bus.bus,
                                eq.band,
                                eq_freq_hz_to_osc(next_hz),
                            ));
                        }
                    }
                }
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_buses() -> Vec<BusAutomixSpec> {
        vec![
            BusAutomixSpec {
                bus: 1,
                label: "Vox".to_string(),
                role: SpectralRole::Vocals,
                priority: 1.5,
                base_db: 0.0,
                pocket_eq: None,
            },
            BusAutomixSpec {
                bus: 2,
                label: "Band".to_string(),
                role: SpectralRole::Band,
                priority: 1.0,
                base_db: -2.0,
                pocket_eq: Some(PocketEqConfig {
                    band: 3,
                    max_dip_db: 3.0,
                    adaptive_freq: false,
                }),
            },
            BusAutomixSpec {
                bus: 3,
                label: "Drums".to_string(),
                role: SpectralRole::Drums,
                priority: 0.9,
                base_db: -3.0,
                pocket_eq: Some(PocketEqConfig {
                    band: 3,
                    max_dip_db: 2.0,
                    adaptive_freq: false,
                }),
            },
        ]
    }

    #[test]
    fn test_dugan_channels_legacy_parity() {
        let levels = vec![0.75, 0.75];
        let mut gains = [0.0; 32];
        calculate_dugan_gains(&levels, 0.01, &mut gains);
        assert!((gains[0] - 0.6).abs() < 0.01);
        assert!((gains[1] - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_kick_heavy_drums_do_not_pump_vocals() {
        let mut am = SpectrumAutomixer::new(sample_buses(), 4.0, 1.5, 0.005);
        let mut last_vox = db_to_fader(0.0);
        // Loud drum transient (0.7) vs steady vocal (0.25)
        for _ in 0..8 {
            let step = am.step(&[0.25, 0.05, 0.70], None);
            for (bus, f) in step.fader_updates {
                if bus == 1 {
                    last_vox = f;
                }
            }
        }
        // Vocals must stay near or above -1.5 dB despite heavy drums.
        assert!(fader_to_db(last_vox) > -1.5);
    }

    #[test]
    fn test_pocket_eq_carves_when_vocals_active_and_releases() {
        let mut am = SpectrumAutomixer::new(sample_buses(), 4.0, 1.5, 0.005);
        let mut band_eq_osc = 0.5;
        for _ in 0..10 {
            let step = am.step(&[0.30, 0.25, 0.20], None);
            for (bus, _band, g) in step.eq_updates {
                if bus == 2 {
                    band_eq_osc = g;
                }
            }
        }
        // Band EQ must dip below 0 dB (0.5).
        assert!(band_eq_osc < 0.45);

        // Now vocals drop silent; Band EQ should recover back toward 0.5 (0 dB).
        for _ in 0..25 {
            let step = am.step(&[0.0, 0.25, 0.20], None);
            for (bus, _band, g) in step.eq_updates {
                if bus == 2 {
                    band_eq_osc = g;
                }
            }
        }
        assert!((band_eq_osc - 0.5).abs() < 0.03);
    }

    #[test]
    fn test_rta_adaptive_pocket_freq_slew_and_deadband() {
        let mut buses = sample_buses();
        buses[1].pocket_eq.as_mut().unwrap().adaptive_freq = true;
        let mut am = SpectrumAutomixer::new(buses, 4.0, 1.5, 0.005);

        // Inactive RTA produces zero eq_freq_updates
        let step_no_rta = am.step(&[0.30, 0.25, 0.20], None);
        assert!(step_no_rta.eq_freq_updates.is_empty());

        // Active RTA peaking at 2800 Hz -> starts at 2000 Hz, slews <= 5% (+100 Hz -> 2100 Hz)
        am.set_rta_snapshot(Some(RtaSnapshot {
            bands: RtaBands {
                sub_bass: 0.1,
                low_mid: 0.2,
                vocal_mid: 0.5,
                air: 0.1,
            },
            vocal_peak_hz: Some(2800.0),
            vocal_peak_energy: 0.6,
        }));
        let step_rta = am.step(&[0.30, 0.25, 0.20], None);
        assert_eq!(step_rta.eq_freq_updates.len(), 1);
        let (bus, band, osc_f) = step_rta.eq_freq_updates[0];
        assert_eq!((bus, band), (2, 3));
        let slewed_hz = crate::model::osc_to_eq_freq_hz(osc_f);
        assert!((slewed_hz - 2100.0).abs() < 2.0);

        // Within 150 Hz deadband of current ~2100 Hz -> no new freq update
        am.set_rta_snapshot(Some(RtaSnapshot {
            bands: RtaBands::default(),
            vocal_peak_hz: Some(2180.0),
            vocal_peak_energy: 0.6,
        }));
        let step_deadband = am.step(&[0.30, 0.25, 0.20], None);
        assert!(step_deadband.eq_freq_updates.is_empty());
    }
}
