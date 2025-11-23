use crate::network::NetworkManager;
use anyhow::Result;
use crate::effects::fx_data::{FX_DATA, BaseUnit};

pub mod fx_data;

pub trait EffectHandler {
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        subdivision: f32,
    ) -> Result<()>;

    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()>;
}

pub struct GenericFxHandler {
    pub effect_name: String,
}

impl GenericFxHandler {
    pub fn new(name: &str) -> Self {
        Self {
            effect_name: name.to_string(),
        }
    }
}

impl EffectHandler for GenericFxHandler {
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        subdivision: f32,
    ) -> Result<()> {
        if let Some(def) = FX_DATA.get(self.effect_name.as_str()) {
            for param in &def.parameters {
                let target_val = match param.base_unit {
                    BaseUnit::Milliseconds => {
                        // ms = (60000 / BPM) * subdiv
                        let ms = (60000.0 / bpm) * subdivision;
                        ms
                    },
                    BaseUnit::Hertz => {
                        // Hz = 1.0 / ((60.0/bpm) * subdivision) = bpm / (60.0 * subdivision)
                        let hz = bpm / (60.0 * subdivision);
                        hz
                    }
                };

                let float_val = param.scaling.scale(target_val);
                // Param index in FX_DATA is 1-based (usually matches document).
                // NetworkManager might expect 1-based.
                network.set_effect_param(slot, param.index, float_val)?;
            }
        }
        Ok(())
    }

    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        // Safe panic values:
        // Time -> ~500ms
        // Hz -> ~1Hz
        if let Some(def) = FX_DATA.get(self.effect_name.as_str()) {
            for param in &def.parameters {
                let safe_val = match param.base_unit {
                    BaseUnit::Milliseconds => 500.0,
                    BaseUnit::Hertz => 1.0,
                };
                let float_val = param.scaling.scale(safe_val);
                network.set_effect_param(slot, param.index, float_val)?;
            }
        }
        Ok(())
    }
}

pub fn get_handler(fx_type: &str) -> Option<Box<dyn EffectHandler + Send + Sync>> {
    if FX_DATA.contains_key(fx_type) {
        return Some(Box::new(GenericFxHandler::new(fx_type)));
    }
    None
}
