use crate::network::NetworkManager;
use anyhow::Result;
use osc_lib::OscMessage;

/// Trait for Effect Handlers
pub trait EffectHandler {
    /// Update the effect parameters based on the current BPM.
    /// `bpm`: Detected Beats Per Minute.
    /// `subdivision`: The musical subdivision (e.g., 1.0 for quarter, 0.5 for eighth).
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        subdivision: f32,
    ) -> Result<()>;

    /// Set effect to a "Safe" conservative state (Panic).
    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()>;
}

/// Handler for "Delay + Chorus" and similar simple delays.
pub struct DelayChorusHandler;

impl EffectHandler for DelayChorusHandler {
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        subdivision: f32,
    ) -> Result<()> {
        // Formula: Time (ms) = (60000 / BPM) * Subdivision
        // X32 Delay Time is often in ms directly? Or 0.0-1.0?
        // Most X32 delays use ms directly in the OSC float argument, OR a mapped 0-1 value.
        // However, the OSC protocol usually accepts physical values if the range is correct,
        // OR we need to map it.
        // Research: /fx/1/par/01 is usually Time.
        // Let's assume Milliseconds for now, as that's standard for "tap" buttons.
        // If it requires 0-1 mapping, we need the scaling factor (usually 3000ms max).

        // Let's calculate MS.
        let ms = (60000.0 / bpm) * subdivision;

        // Clamp to typical range (e.g. 10ms - 3000ms)
        let ms = ms.clamp(10.0, 3000.0);

        // For X32, many params are 0.0-1.0.
        // Time param for standard delay is often mapped 0-3000ms.
        // So val = ms / 3000.0
        let val = ms / 3000.0;

        // Assuming Parameter 1 is Time (Left) and Parameter 2 is Time (Right) or similar.
        // For Delay+Chorus: Param 1 is typically Time.
        network.set_effect_param(slot, 1, val)?;

        Ok(())
    }

    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        // Set to a safe 500ms (neutral delay)
        let safe_val = 500.0 / 3000.0;
        network.set_effect_param(slot, 1, safe_val)?;
        // Maybe lower feedback (Param 3 usually)?
        network.set_effect_param(slot, 3, 0.1)?;
        Ok(())
    }
}

/// Handler for "3-Tap Delay"
pub struct ThreeTapDelayHandler;

impl EffectHandler for ThreeTapDelayHandler {
    fn update(
        &self,
        network: &NetworkManager,
        slot: usize,
        bpm: f32,
        subdivision: f32,
    ) -> Result<()> {
        // 3-Tap has 3 time parameters.
        // We can set them to 1x, 2x, 3x subdivision or a rhythmic pattern.
        // Let's do Quarter, Dotted Eighth, Half.

        let quarter_ms = 60000.0 / bpm;

        // Tap 1: 1/4
        let t1 = (quarter_ms * subdivision).clamp(10.0, 3000.0) / 3000.0;
        // Tap 2: 3/16 (Dotted Eighth relative to quarter) -> just offset it
        let t2 = (quarter_ms * subdivision * 0.75).clamp(10.0, 3000.0) / 3000.0;
        // Tap 3: 1/2
        let t3 = (quarter_ms * subdivision * 2.0).clamp(10.0, 3000.0) / 3000.0;

        network.set_effect_param(slot, 1, t1)?;
        network.set_effect_param(slot, 3, t2)?; // Param indices need verification
        network.set_effect_param(slot, 5, t3)?;

        Ok(())
    }

    fn panic(&self, network: &NetworkManager, slot: usize) -> Result<()> {
        network.set_effect_param(slot, 1, 0.2)?;
        network.set_effect_param(slot, 3, 0.3)?;
        network.set_effect_param(slot, 5, 0.4)?;
        // Kill feedback
        network.set_effect_param(slot, 2, 0.0)?;
        Ok(())
    }
}

// TODO: Stereo Delay
// TODO: Rhythm Delay
// TODO: Modulation Delay
// TODO: Delay + Chamber
// TODO: Delay + Flanger
// TODO: All Reverbs (Hall, Ambience, Plate, Room, Chamber, Gated, Reverse, Vintage)
// TODO: All Modulation (Chorus, Dimension C, Flanger, Phaser)
// TODO: Dynamics (Align Attack/Hold/Release)

pub fn get_handler(fx_type: &str) -> Option<Box<dyn EffectHandler + Send + Sync>> {
    match fx_type {
        "DELAY" | "DLY+CHO" => Some(Box::new(DelayChorusHandler)),
        "3TAP" => Some(Box::new(ThreeTapDelayHandler)),
        // "STEREO_DELAY" => Some(Box::new(StereoDelayHandler)),
        _ => None,
    }
}
