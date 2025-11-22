use super::{EffectHandler, EffectConfig};
use crate::network::NetworkManager;
use crate::musical_theory::MusicCalculator;
use anyhow::Result;

/// Handler for standard "Stereo Delay" or "Delay" (DLY).
/// Param 2 is Time.
pub struct GenericDelayHandler;

impl EffectHandler for GenericDelayHandler {
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()> {
        let ms = MusicCalculator::bpm_to_ms(bpm, &config.subdivision);
        // Clamp to 1.0 - 3000.0 ms (Standard X32 Delay Range)
        let ms = ms.clamp(1.0, 3000.0);

        // Normalized value 0.0 - 1.0 maps to 1 - 3000
        // formula: val = (ms - 1) / 2999
        let val = (ms - 1.0) / 2999.0;

        network.set_effect_param(slot, 2, val)?;
        Ok(())
    }

    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        // Kill feedback (Param 6 typically) and set neutral time
        network.set_effect_param(slot, 2, 0.166)?; // ~500ms
        network.set_effect_param(slot, 6, 0.0)?;    // Feedback 0%
        Ok(())
    }
}

/// Handler for Multi-Tap Delays (3TAP, 4TAP).
/// Param 1 is Time (Master).
pub struct TapDelayHandler;

impl EffectHandler for TapDelayHandler {
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()> {
        let ms = MusicCalculator::bpm_to_ms(bpm, &config.subdivision);
        let ms = ms.clamp(1.0, 3000.0);
        let val = (ms - 1.0) / 2999.0;

        network.set_effect_param(slot, 1, val)?;
        Ok(())
    }

    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        network.set_effect_param(slot, 1, 0.166)?;
        // 3TAP feedback is Param 3? 4TAP feedback is Param 6?
        // Safe approach: zero out common feedback params 3 and 6
        network.set_effect_param(slot, 3, 0.0)?;
        network.set_effect_param(slot, 6, 0.0)?;
        Ok(())
    }
}
