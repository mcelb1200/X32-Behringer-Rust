use crate::network::NetworkManager;
use crate::scaling::log2float;
use anyhow::Result;

pub struct CompressorHandler {
    pub target_channels: Vec<usize>,
    pub hold_subdivision: f32,
    pub release_subdivision: f32,
    last_hold_val: Option<f32>,
    last_release_val: Option<f32>,
}

impl CompressorHandler {
    pub fn new(channels: Vec<usize>) -> Self {
        Self {
            target_channels: channels,
            hold_subdivision: 0.25,   // Default: 1/16th note approx
            release_subdivision: 1.0, // Default: 1/4 note
            last_hold_val: None,
            last_release_val: None,
        }
    }

    pub fn update(&mut self, network: &NetworkManager, bpm: f32) -> Result<()> {
        if self.target_channels.is_empty() {
            return Ok(());
        }

        let quarter_ms = 60000.0 / bpm;
        let hold_ms = quarter_ms * self.hold_subdivision;
        let release_ms = quarter_ms * self.release_subdivision;

        // Dynamics scaling (standard X32 estimates)
        // Hold: [0.02, 2000] log
        // Release: [20, 2000] log (using XLIM reference from fxparse.c as a safe bet for dynamics)

        let hold_val = log2float(hold_ms, 0.02, 11.512925);
        let release_val = log2float(release_ms, 20.0, 4.605170);

        let hold_changed = self
            .last_hold_val
            .map_or(true, |v| (v - hold_val).abs() > 0.001);
        let release_changed = self
            .last_release_val
            .map_or(true, |v| (v - release_val).abs() > 0.001);

        if !hold_changed && !release_changed {
            return Ok(());
        }

        for ch in &self.target_channels {
            // Channels 1-32
            if *ch >= 1 && *ch <= 32 {
                if hold_changed {
                    let hold_path = format!("/ch/{:02}/dyn/hold", ch);
                    network.send_osc_float(&hold_path, hold_val)?;
                }
                if release_changed {
                    let release_path = format!("/ch/{:02}/dyn/release", ch);
                    network.send_osc_float(&release_path, release_val)?;
                }
            }
        }

        self.last_hold_val = Some(hold_val);
        self.last_release_val = Some(release_val);

        Ok(())
    }
}
