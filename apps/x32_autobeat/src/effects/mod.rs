use crate::network::NetworkManager;
use anyhow::Result;

/// Delay effect handlers.
pub mod delay;
/// Modulation effect handlers.
pub mod modulation;
/// Reverb effect handlers.
pub mod reverb;

use delay::{CombinedDelayHandler, GenericDelayHandler, TapDelayHandler};
use modulation::ModulationHandler;
use reverb::{ReverbHandler, ReverbType};

/// Configuration for synchronizing an effect slot.
#[derive(Clone, Debug)]
pub struct EffectConfig {
    /// Musical subdivision (e.g. `"1/4"`, `"1/8D"`).
    pub subdivision: String,
    /// Effect style preset name (e.g. `"Standard"`, `"Tight"`, `"Big"`).
    pub style: String,
    /// Whether synchronization is enabled for this slot.
    pub enabled: bool,
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self {
            subdivision: "1/4".to_string(),
            style: "Standard".to_string(),
            enabled: true,
        }
    }
}

/// Trait for Effect Handlers
#[async_trait::async_trait]
pub trait EffectHandler {
    /// Updates the effect slot parameters to match the detected `bpm` and `config`.
    async fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()>;
    /// Resets feedback/time parameters on the effect slot to a safe state.
    async fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()>;
}

/// Returns an [`EffectHandler`] for the given mixer effect type code, if supported.
pub fn get_handler(fx_type: &str) -> Option<Box<dyn EffectHandler + Send + Sync>> {
    match fx_type {
        // Delays
        "DLY" | "STEREO_DELAY" => Some(Box::new(GenericDelayHandler)),
        "3TAP" => Some(Box::new(TapDelayHandler { is_4tap: false })),
        "4TAP" => Some(Box::new(TapDelayHandler { is_4tap: true })),

        // Reverbs
        "HALL" => Some(Box::new(ReverbHandler::new(ReverbType::Hall))),
        "AMBI" => Some(Box::new(ReverbHandler::new(ReverbType::Ambience))),
        "PLAT" => Some(Box::new(ReverbHandler::new(ReverbType::Plate))),
        "ROOM" => Some(Box::new(ReverbHandler::new(ReverbType::Room))),
        "CHAM" => Some(Box::new(ReverbHandler::new(ReverbType::Chamber))),
        "VREV" => Some(Box::new(ReverbHandler::new(ReverbType::Vintage))),
        "VRM" => Some(Box::new(ReverbHandler::new(ReverbType::VintageRoom))),

        // Modulation
        "CRS" | "CHORUS" => Some(Box::new(ModulationHandler)),
        "FLNG" | "FLANGER" => Some(Box::new(ModulationHandler)),
        "PHAS" | "PHASER" => Some(Box::new(ModulationHandler)),

        // Combined Effects (Delay is always Param 1)
        "DLY+CHO" | "D_CR" => Some(Box::new(CombinedDelayHandler { is_modd: false })),
        "DLY+FLG" | "D_FL" => Some(Box::new(CombinedDelayHandler { is_modd: false })),
        "DLY+REV" | "D_RV" => Some(Box::new(CombinedDelayHandler { is_modd: false })),
        "MODD" | "MODULATION_DELAY" => Some(Box::new(CombinedDelayHandler { is_modd: true })),

        _ => None,
    }
}
