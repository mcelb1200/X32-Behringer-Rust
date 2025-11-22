use super::{EffectConfig, EffectHandler};
use crate::musical_theory::MusicCalculator;
use crate::network::NetworkManager;
use anyhow::Result;

#[derive(Clone, Copy)]
pub enum ReverbType {
    Hall,        // 0.2 - 5.0s
    Ambience,    // 0.2 - 7.3s
    Plate,       // 0.5 - 10.0s
    Room,        // 0.3 - 29.0s
    Chamber,     // 0.3 - 29.0s
    Vintage,     // 0.4 - 4.5s (VREV)
    VintageRoom, // 0.1 - 20.0s (VRM)
}

impl ReverbType {
    fn decay_range(&self) -> (f32, f32, bool) {
        // (Min, Max, IsLog)
        match self {
            ReverbType::Hall => (0.2, 5.0, true),
            ReverbType::Ambience => (0.2, 7.3, true),
            ReverbType::Plate => (0.5, 10.0, true),
            ReverbType::Room => (0.3, 29.0, true),
            ReverbType::Chamber => (0.3, 29.0, true),
            ReverbType::Vintage => (0.4, 4.5, false), // VREV Time is linear 0.4-4.5
            ReverbType::VintageRoom => (0.1, 20.0, true),
        }
    }
}

pub struct ReverbHandler {
    rv_type: ReverbType,
}

impl ReverbHandler {
    pub fn new(rv_type: ReverbType) -> Self {
        Self { rv_type }
    }

    fn get_style_settings(style: &str) -> (&str, &str) {
        // Returns (Pre-Delay Subdiv, Decay Subdiv)
        match style {
            "Tight" => ("1/128", "1/4"),
            "Natural" => ("1/64", "1/2"),
            "Big" => ("1/32", "1/1"),
            "Huge" => ("1/16", "2/1"), // 2 Bars
            _ => ("1/64", "1/2"),      // Default to Natural
        }
    }
}

impl EffectHandler for ReverbHandler {
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()> {
        let (pre_sub, decay_sub) = Self::get_style_settings(&config.style);

        // 1. Pre-Delay (Param 1)
        // X32 Pre-Delay is typically 0-200ms Linear
        let pre_ms = MusicCalculator::bpm_to_ms(bpm, pre_sub);
        let pre_ms = pre_ms.clamp(0.0, 200.0);
        let pre_val = pre_ms / 200.0;
        network.set_effect_param(slot, 1, pre_val)?;

        // 2. Decay (Param 2)
        let decay_ms = MusicCalculator::bpm_to_ms(bpm, decay_sub);
        let decay_sec = decay_ms / 1000.0;
        let (min, max, is_log) = self.rv_type.decay_range();
        let decay_sec = decay_sec.clamp(min, max);

        let decay_val = if is_log {
            // Map log range [min, max] to [0.0, 1.0]
            // val = (log(x) - log(min)) / (log(max) - log(min))
            (decay_sec.ln() - min.ln()) / (max.ln() - min.ln())
        } else {
            // Linear mapping
            (decay_sec - min) / (max - min)
        };

        network.set_effect_param(slot, 2, decay_val)?;

        Ok(())
    }

    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        // Short decay (bottom 10%)
        network.set_effect_param(slot, 2, 0.1)?;
        // Zero pre-delay
        network.set_effect_param(slot, 1, 0.0)?;
        Ok(())
    }
}
