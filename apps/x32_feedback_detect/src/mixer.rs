use anyhow::Result;
use osc_lib::OscArg;
use std::collections::HashMap;
use x32_lib::MixerClient;

#[derive(Debug, Clone)]
pub struct AppliedNotch {
    pub frequency: f32,
    pub depth: f32, // in dB, typically negative
    pub q: f32,
}

pub struct MixerState {
    client: MixerClient,
    target_channel: u8,
    // Max 3 notches. Key is eq band index (1..=6)
    pub applied_notches: HashMap<u8, AppliedNotch>,
}

impl MixerState {
    pub fn new(client: MixerClient, target_channel: u8) -> Self {
        Self {
            client,
            target_channel,
            applied_notches: HashMap::new(),
        }
    }

    /// Map a real frequency to X32 float parameter [0.0, 1.0] representing 20Hz-20kHz.
    fn freq_to_float(freq: f32) -> f32 {
        let f_min = 20.0;
        let f_max = 20000.0;
        let clamped = freq.clamp(f_min, f_max);

        // The console uses an exponential scale for EQ frequency
        // log10(clamped / f_min) / log10(f_max / f_min)
        ((clamped / f_min).log10()) / ((f_max / f_min).log10())
    }

    /// Convert a depth in dB (e.g. -6.0) to an X32 float parameter.
    /// EQ Gain range is typically -15dB to +15dB mapping to 0.0 to 1.0.
    fn gain_to_float(gain_db: f32) -> f32 {
        let clamped = gain_db.clamp(-15.0, 15.0);
        (clamped + 15.0) / 30.0
    }

    /// Map Q to float parameter [0.0, 1.0]. Typically Q ranges from 0.3 to 10.0 (or similar).
    fn q_to_float(_q: f32) -> f32 {
        // Just setting to max Q for surgical notch, typical value near 1.0 is highest Q
        1.0
    }

    pub async fn apply_notch(&mut self, frequency: f32) -> Result<()> {
        let max_depth = -12.0;

        // Find if this frequency is already notched (within 5% tolerance)
        let mut existing_band = None;
        for (&band, notch) in &self.applied_notches {
            let diff = (notch.frequency - frequency).abs();
            if diff < (frequency * 0.05) {
                existing_band = Some(band);
                break;
            }
        }

        let (band_to_use, new_notch) = if let Some(band) = existing_band {
            // Deepen existing notch
            let mut notch = self.applied_notches[&band].clone();
            notch.depth -= 3.0;
            if notch.depth < max_depth {
                notch.depth = max_depth;
                // At max depth, could optionally pull fader down, but sticking to EQ here
            }
            (band, notch)
        } else {
            // Create new notch if we have free bands (max 3)
            if self.applied_notches.len() >= 3 {
                // Ignore if all allowed bands are used
                return Ok(());
            }

            // Find first available band between 1..=6. Prefer higher bands for notches.
            let mut free_band = None;
            for b in [6, 5, 4, 3, 2, 1] {
                if !self.applied_notches.contains_key(&b) {
                    free_band = Some(b);
                    break;
                }
            }

            if let Some(band) = free_band {
                (band, AppliedNotch {
                    frequency,
                    depth: -3.0,
                    q: 10.0, // High Q
                })
            } else {
                return Ok(());
            }
        };

        // Apply settings
        let ch_str = format!("{:02}", self.target_channel);

        // 1. Set type to PEQ (3)
        let path_type = format!("/ch/{}/eq/{}/type", ch_str, band_to_use);
        self.client.send_message(&path_type, vec![OscArg::Int(3)]).await?;

        // 2. Set Frequency
        let path_freq = format!("/ch/{}/eq/{}/freq", ch_str, band_to_use);
        let freq_val = Self::freq_to_float(new_notch.frequency);
        self.client.send_message(&path_freq, vec![OscArg::Float(freq_val)]).await?;

        // 3. Set Gain
        let path_gain = format!("/ch/{}/eq/{}/gain", ch_str, band_to_use);
        let gain_val = Self::gain_to_float(new_notch.depth);
        self.client.send_message(&path_gain, vec![OscArg::Float(gain_val)]).await?;

        // 4. Set Q
        let path_q = format!("/ch/{}/eq/{}/q", ch_str, band_to_use);
        let q_val = Self::q_to_float(new_notch.q);
        self.client.send_message(&path_q, vec![OscArg::Float(q_val)]).await?;

        // 5. Ensure EQ block is on
        let path_on = format!("/ch/{}/eq/on", ch_str);
        self.client.send_message(&path_on, vec![OscArg::Int(1)]).await?;

        self.applied_notches.insert(band_to_use, new_notch);

        Ok(())
    }

    pub async fn reset_notches(&mut self) -> Result<()> {
        let ch_str = format!("{:02}", self.target_channel);

        for band in self.applied_notches.keys() {
            let path_gain = format!("/ch/{}/eq/{}/gain", ch_str, band);
            // Reset to 0 dB
            let gain_val = Self::gain_to_float(0.0);
            self.client.send_message(&path_gain, vec![OscArg::Float(gain_val)]).await?;
        }

        self.applied_notches.clear();
        Ok(())
    }
}
