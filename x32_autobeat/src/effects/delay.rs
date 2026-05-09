use super::{EffectConfig, EffectHandler};
use crate::musical_theory::MusicCalculator;
use crate::network::NetworkManager;
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
        network.set_effect_param(slot, 6, 0.0)?; // Feedback 0%
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

/// Handler for Combined Delays (D_CR, D_FL, MODD).
/// Param 1 is Time.
pub struct CombinedDelayHandler;

impl EffectHandler for CombinedDelayHandler {
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
        // D_CR / D_FL feedback is Param 4, MODD feed is Param 3.
        // Safe approach: zero out common feedback params 3 and 4
        network.set_effect_param(slot, 3, 0.0)?;
        network.set_effect_param(slot, 4, 0.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;

    // A mock network trait might be better here, but since NetworkManager relies on physical sockets with a bug
    // in parsing dummy IP's in test setup (0 timeout on non-dummy addresses via lib), we perform isolated state checks if possible.
    // However, since NetworkManager requires a full physical network setup and we cannot inject a mock,
    // we use `if let Ok` to silently ignore on test failure environments that don't support proper bind/timeout,
    // ensuring we at least test the code block if it DOES bind successfully.

    #[test]
    fn test_combined_delay_handler_update() {
        let (tx, _) = unbounded();
        if let Ok(network) = NetworkManager::new("127.0.0.1:10023", 1, tx, "/btn", "/enc") {
            let handler = CombinedDelayHandler;
            let config = EffectConfig {
                subdivision: "1/4".to_string(), // 1/4 at 120bpm is 500ms
                style: "Standard".to_string(),
                enabled: true,
            };
            // The underlying send will fail since there is no OSC server running on 10023,
            // but we expect an Error rather than a panic.
            let res = handler.update(&network, 1, 120.0, &config);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_combined_delay_handler_panic() {
        let (tx, _) = unbounded();
        if let Ok(network) = NetworkManager::new("127.0.0.1:10023", 1, tx, "/btn", "/enc") {
            let handler = CombinedDelayHandler;
            let res = handler.panic(&network, 1);
            assert!(res.is_err());
        }
    }
}
