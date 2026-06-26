use super::{EffectConfig, EffectHandler};
use crate::musical_theory::MusicCalculator;
use crate::network::NetworkManager;
use crate::scaling::{afine2float, log2float, ratio2float};
use anyhow::Result;

/// Handler for standard "Stereo Delay" or "Delay" (DLY).
/// Param 2 is Time.
pub struct GenericDelayHandler;

#[async_trait::async_trait]
impl EffectHandler for GenericDelayHandler {
    async fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()> {
        let ms = MusicCalculator::bpm_to_ms(bpm, &config.subdivision);
        // Clamp to 1.0 - 3000.0 ms (Standard X32 Delay Range)
        let ms = ms.clamp(1.0, 3000.0);
        let val = afine2float(ms, 1.0, 2999.0);
        network.set_effect_param(slot, 2, val).await?;

        // Dynamic style scaling
        let (mix, offset, feedback, hi_cut) = match config.style.as_str() {
            "Tight" => (15.0, 0.0, 12.0, 8000.0),
            "Natural" => (10.0, -10.0, 22.0, 3000.0),
            "Standard" => (20.0, 20.0, 30.0, 4000.0),
            "Big" => (30.0, 40.0, 45.0, 2500.0),
            "Huge" => (40.0, 60.0, 65.0, 1500.0),
            _ => (20.0, 20.0, 30.0, 4000.0),
        };

        network
            .set_effect_param(slot, 1, ratio2float(mix, 100.0))
            .await?;
        network
            .set_effect_param(slot, 6, afine2float(offset, -100.0, 200.0))
            .await?;
        network
            .set_effect_param(slot, 10, ratio2float(feedback, 100.0))
            .await?;
        network
            .set_effect_param(slot, 11, ratio2float(feedback, 100.0))
            .await?;
        network
            .set_effect_param(slot, 12, log2float(hi_cut, 200.0, 4.605_170_2))
            .await?;

        Ok(())
    }

    async fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        // Kill feedback (Param 6 typically) and set neutral time
        network.set_effect_param(slot, 2, 0.166).await?; // ~500ms
        network.set_effect_param(slot, 6, 0.0).await?; // Feedback 0%
        Ok(())
    }
}

/// Handler for Multi-Tap Delays (3TAP, 4TAP).
/// Param 1 is Time (Master).
pub struct TapDelayHandler {
    pub is_4tap: bool,
}

#[async_trait::async_trait]
impl EffectHandler for TapDelayHandler {
    async fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()> {
        let ms = MusicCalculator::bpm_to_ms(bpm, &config.subdivision);
        let ms = ms.clamp(1.0, 3000.0);
        let val = afine2float(ms, 1.0, 2999.0);
        network.set_effect_param(slot, 1, val).await?;

        // Dynamic style scaling
        let (gain_base, feedback, hi_cut) = match config.style.as_str() {
            "Tight" => (15.0, 12.0, 8000.0),
            "Natural" => (10.0, 22.0, 3000.0),
            "Standard" => (20.0, 30.0, 4000.0),
            "Big" => (30.0, 45.0, 2500.0),
            "Huge" => (40.0, 65.0, 1500.0),
            _ => (20.0, 30.0, 4000.0),
        };

        network
            .set_effect_param(slot, 2, ratio2float(gain_base, 100.0))
            .await?;
        if self.is_4tap {
            network
                .set_effect_param(slot, 3, ratio2float(feedback, 100.0))
                .await?;
            network
                .set_effect_param(slot, 5, log2float(hi_cut, 200.0, 4.605_170_2))
                .await?;
        } else {
            network
                .set_effect_param(slot, 4, ratio2float(feedback, 100.0))
                .await?;
            network
                .set_effect_param(slot, 6, log2float(hi_cut, 200.0, 4.605_170_2))
                .await?;
        }

        Ok(())
    }

    async fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        network.set_effect_param(slot, 1, 0.166).await?;
        // 3TAP feedback is Param 3? 4TAP feedback is Param 6?
        // Safe approach: zero out common feedback params 3 and 6
        network.set_effect_param(slot, 3, 0.0).await?;
        network.set_effect_param(slot, 6, 0.0).await?;
        Ok(())
    }
}

/// Handler for Combined Delays (D_CR, D_FL, MODD).
/// Param 1 is Time.
pub struct CombinedDelayHandler {
    pub is_modd: bool,
}

#[async_trait::async_trait]
impl EffectHandler for CombinedDelayHandler {
    async fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()> {
        let ms = MusicCalculator::bpm_to_ms(bpm, &config.subdivision);
        let ms = ms.clamp(1.0, 3000.0);
        let val = afine2float(ms, 1.0, 2999.0);
        network.set_effect_param(slot, 1, val).await?;

        // Dynamic style scaling
        let (mix, feedback, hi_cut) = match config.style.as_str() {
            "Tight" => (15.0, 12.0, 8000.0),
            "Natural" => (10.0, 22.0, 3000.0),
            "Standard" => (20.0, 30.0, 4000.0),
            "Big" => (30.0, 45.0, 2500.0),
            "Huge" => (40.0, 65.0, 1500.0),
            _ => (20.0, 30.0, 4000.0),
        };

        if self.is_modd {
            network
                .set_effect_param(slot, 13, ratio2float(mix, 100.0))
                .await?;
            network
                .set_effect_param(slot, 3, ratio2float(feedback, 100.0))
                .await?;
            network
                .set_effect_param(slot, 5, log2float(hi_cut, 200.0, 4.605_170_2))
                .await?;
        } else {
            network
                .set_effect_param(slot, 12, ratio2float(mix, 100.0))
                .await?;
            network
                .set_effect_param(slot, 4, ratio2float(feedback, 100.0))
                .await?;
            // Combined feed hi cut is Param 3 [1000..20000]
            network
                .set_effect_param(slot, 3, log2float(hi_cut, 1000.0, 2.995_732_3))
                .await?;
        }

        Ok(())
    }

    async fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        network.set_effect_param(slot, 1, 0.166).await?;
        // D_CR / D_FL feedback is Param 4, MODD feed is Param 3.
        // Safe approach: zero out common feedback params 3 and 4
        network.set_effect_param(slot, 3, 0.0).await?;
        network.set_effect_param(slot, 4, 0.0).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Source;
    use crossbeam_channel::unbounded;

    #[tokio::test]
    async fn test_combined_delay_handler_update() {
        let (tx, _) = unbounded();
        if let Ok(network) =
            NetworkManager::new("127.0.0.1:10023", Source::Channel(1), tx, "/btn", "/enc").await
        {
            let handler = CombinedDelayHandler { is_modd: false };
            let config = EffectConfig {
                subdivision: "1/4".to_string(), // 1/4 at 120bpm is 500ms
                style: "Standard".to_string(),
                enabled: true,
            };
            let _res = handler.update(&network, 1, 120.0, &config).await;
        }
    }

    #[tokio::test]
    async fn test_combined_delay_handler_panic() {
        let (tx, _) = unbounded();
        if let Ok(network) =
            NetworkManager::new("127.0.0.1:10023", Source::Channel(1), tx, "/btn", "/enc").await
        {
            let handler = CombinedDelayHandler { is_modd: false };
            let _res = handler.panic(&network, 1).await;
        }
    }
}
