use super::{EffectConfig, EffectHandler};
use crate::musical_theory::MusicCalculator;
use crate::network::NetworkManager;
use anyhow::Result;

/// Handler for Modulation Effects (Chorus, Flanger, Phaser).
/// Param 1 is typically Rate/Speed (Hz).
/// Range is usually 0.05 Hz to 5.0 Hz (Logarithmic).
pub struct ModulationHandler;

impl EffectHandler for ModulationHandler {
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()> {
        // Interpret subdivision as "Cycles per Bar" or similar.
        // e.g. "1/4" -> 1 cycle every quarter note? Or 1 cycle every 4 bars?
        // Convention: "1/1" = 1 cycle per bar. "1/4" = 4 cycles per bar (quarter note rate).
        // "4/1" = 1 cycle every 4 bars.

        // Let's reuse the subdivision parser.
        // If sub is "1/4" (0.25), that usually means duration.
        // Rate = 1 / Duration.
        // So we calculate MS duration for the subdivision, then Hz = 1000 / ms.

        let period_ms = MusicCalculator::bpm_to_ms(bpm, &config.subdivision);
        if period_ms <= 0.0 {
            return Ok(());
        }

        let hz: f32 = 1000.0 / period_ms;

        // Clamp to X32 Range (0.05 - 5.0 Hz, Log)
        let hz = hz.clamp(0.05, 5.0);

        // Log Mapping: val = (ln(hz) - ln(min)) / (ln(max) - ln(min))
        let min = 0.05f32;
        let max = 5.0f32;
        let val = (hz.ln() - min.ln()) / (max.ln() - min.ln());

        network.set_effect_param(slot, 1, val)?;
        Ok(())
    }

    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        // Set to slow speed (bottom 10%)
        network.set_effect_param(slot, 1, 0.1)?;
        // Reduce depth (Param 2 or 3 usually)
        network.set_effect_param(slot, 3, 0.0)?;
        Ok(())
    }
}
