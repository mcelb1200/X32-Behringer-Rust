use super::{EffectConfig, EffectHandler};
use crate::musical_theory::MusicCalculator;
use crate::network::NetworkManager;
use crate::scaling::ratio2float;
use anyhow::Result;

/// Handler for Modulation Effects (Chorus, Flanger, Phaser).
/// Param 1 is typically Rate/Speed (Hz).
/// Range is usually 0.05 Hz to 5.0 Hz (Logarithmic).
pub struct ModulationHandler;

#[async_trait::async_trait]
impl EffectHandler for ModulationHandler {
    async fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()> {
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

        network.set_effect_param(slot, 1, val).await?;

        // Dynamic style scaling for Depth and Mix
        let (depth, mix) = match config.style.as_str() {
            "Tight" => (15.0, 30.0),
            "Natural" => (10.0, 10.0),
            "Standard" => (25.0, 25.0),
            "Big" => (50.0, 40.0),
            "Huge" => (80.0, 50.0),
            _ => (25.0, 25.0),
        };

        network.set_effect_param(slot, 2, ratio2float(depth, 100.0)).await?;
        network.set_effect_param(slot, 3, ratio2float(depth, 100.0)).await?; // Depth R for Chorus/Flanger
        network.set_effect_param(slot, 6, ratio2float(mix, 100.0)).await?;

        Ok(())
    }

    async fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        // Set to slow speed (bottom 10%)
        network.set_effect_param(slot, 1, 0.1).await?;
        // Reduce depth (Param 2 or 3 usually)
        network.set_effect_param(slot, 3, 0.0).await?;
        Ok(())
    }
}
