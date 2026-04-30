use crate::network::NetworkManager;
use anyhow::Result;

pub mod delay;
pub mod modulation;
pub mod reverb;

use delay::{GenericDelayHandler, TapDelayHandler};
use modulation::ModulationHandler;
use reverb::{ReverbHandler, ReverbType};

#[derive(Clone, Debug)]
pub struct EffectConfig {
    pub subdivision: String,
    pub style: String,
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
pub trait EffectHandler {
    /// Update the effect parameters based on the current BPM and configuration.
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        config: &EffectConfig,
    ) -> Result<()>;

    /// Set effect to a "Safe" conservative state (Panic).
    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()>;
}

pub fn get_handler(fx_type: &str) -> Option<Box<dyn EffectHandler + Send + Sync>> {
    match fx_type {
        // Delays
        "DLY" | "STEREO_DELAY" => Some(Box::new(GenericDelayHandler)),
        "3TAP" => Some(Box::new(TapDelayHandler)),
        "4TAP" => Some(Box::new(TapDelayHandler)),

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

        // TODO: Combined Effects (DLY+CHO, etc.) often behave like DLY for tempo
        "DLY+CHO" | "D_CR" => Some(Box::new(GenericDelayHandler)),
        "DLY+FLG" | "D_FL" => Some(Box::new(GenericDelayHandler)),

        _ => None,
    }
}
