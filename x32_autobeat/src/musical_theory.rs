pub struct MusicCalculator;

impl MusicCalculator {
    /// Calculate milliseconds for a given BPM and rhythmic subdivision.
    /// `subdivision`: A string like "1/4", "1/8", "1/8d" (dotted), "1/8t" (triplet).
    pub fn bpm_to_ms(bpm: f32, subdivision: &str) -> f32 {
        if bpm <= 0.0 {
            return 0.0;
        }

        let quarter_note_ms = 60_000.0 / bpm;

        // Parse base fraction
        let (base_mult, modifier) = Self::parse_subdivision(subdivision);

        let mut ms = quarter_note_ms * base_mult;

        // Apply modifier
        match modifier {
            'd' => ms *= 1.5,       // Dotted
            't' => ms *= 2.0 / 3.0, // Triplet
            _ => {}
        }

        ms
    }

    /// Calculate Frequency (Hz) for a given BPM and cycles per bar.
    /// Used for Modulation effects (Phaser, Flanger).
    pub fn bpm_to_hz(bpm: f32, cycles_per_bar: f32) -> f32 {
        if bpm <= 0.0 || cycles_per_bar <= 0.0 {
            return 0.0;
        }

        // 1 Bar (4/4) duration in seconds = (60 / BPM) * 4
        let bar_sec = (60.0 / bpm) * 4.0;

        // Hz = Cycles / Seconds
        cycles_per_bar / bar_sec
    }

    fn parse_subdivision(sub: &str) -> (f32, char) {
        let clean_sub = sub.trim();
        let last_char = clean_sub.chars().last().unwrap_or(' ');

        let (frac_str, modifier) = if last_char == 'd' || last_char == 't' {
            (&clean_sub[..clean_sub.len() - 1], last_char)
        } else {
            (clean_sub, ' ')
        };

        let parts: Vec<&str> = frac_str.split('/').collect();
        let multiplier = if parts.len() == 2 {
            let num: f32 = parts[0].parse().unwrap_or(1.0);
            let den: f32 = parts[1].parse().unwrap_or(1.0);
            // Base is relative to quarter note (1/4)
            // 1/4 = 1.0
            // 1/8 = 0.5
            // 1/1 = 4.0
            (num / den) * 4.0
        } else if parts.len() == 1 {
            // Handle "1", "2" (bars)
            let val: f32 = parts[0].parse().unwrap_or(1.0);
            val * 4.0 // Assuming input is in Bars if no fraction? Or Whole notes?
        // Standard convention: "1/4" is a quarter. "1" is a whole note.
        // If input is "1", that is a whole note = 4 beats.
        } else {
            1.0
        };

        (multiplier, modifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpm_to_ms() {
        let bpm = 120.0; // 500ms per beat

        // 1/4 = 500ms
        assert!((MusicCalculator::bpm_to_ms(bpm, "1/4") - 500.0).abs() < 0.1);

        // 1/8 = 250ms
        assert!((MusicCalculator::bpm_to_ms(bpm, "1/8") - 250.0).abs() < 0.1);

        // 1/8d (dotted) = 375ms
        assert!((MusicCalculator::bpm_to_ms(bpm, "1/8d") - 375.0).abs() < 0.1);

        // 1/4t (triplet) = 333.33ms
        assert!((MusicCalculator::bpm_to_ms(bpm, "1/4t") - 333.33).abs() < 0.1);

        // 1/1 (Whole note) = 2000ms
        assert!((MusicCalculator::bpm_to_ms(bpm, "1/1") - 2000.0).abs() < 0.1);
    }

    #[test]
    fn test_bpm_to_hz() {
        let bpm = 120.0;
        // 1 beat = 0.5s. 1 Bar = 2.0s.

        // 1 cycle per bar = 0.5 Hz
        assert!((MusicCalculator::bpm_to_hz(bpm, 1.0) - 0.5).abs() < 0.01);

        // 4 cycles per bar = 2.0 Hz (Quarter notes)
        assert!((MusicCalculator::bpm_to_hz(bpm, 4.0) - 2.0).abs() < 0.01);
    }
}
