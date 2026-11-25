//! Safe little-endian `/meters/*` blob parsing and dBFS helpers.

/// Aggregated 4-band perceptual energy extracted from a 100-band RTA meter blob (`/meters/4`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RtaBands {
    /// Sub-bass / kick fundamental energy (~20–120 Hz).
    pub sub_bass: f32,
    /// Low-mid / bass & body energy (~120–500 Hz).
    pub low_mid: f32,
    /// Vocal clarity / presence band (~500 Hz – 4 kHz).
    pub vocal_mid: f32,
    /// High / cymbal air band (~4–20 kHz).
    pub air: f32,
}

/// Parses an OSC meter blob containing a 4-byte little-endian count header followed by LE `f32` samples.
pub fn parse_meter_blob(data: &[u8]) -> Vec<f32> {
    if data.len() < 4 {
        return Vec::new();
    }
    let available = (data.len() - 4) / 4;
    let header_raw = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    // X32 sometimes encodes byte length vs float count in the 4-byte header; handle both safely.
    let declared = if header_raw == available * 4 {
        available
    } else {
        header_raw.min(available)
    };
    let count = declared.min(available);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        if let Some(val) = meter_value_at(data, i) {
            out.push(val);
        }
    }
    out
}

/// Zero-allocation lookup of a single LE `f32` meter value at `idx` (skipping the 4-byte header).
pub fn meter_value_at(data: &[u8], idx: usize) -> Option<f32> {
    let start = 4usize.checked_add(idx.checked_mul(4)?)?;
    let end = start.checked_add(4)?;
    let bytes: [u8; 4] = data.get(start..end)?.try_into().ok()?;
    Some(f32::from_le_bytes(bytes))
}

/// Converts a linear `[0.0, 1.0]` meter value to dBFS (clamped at `-144.0` dBFS floor).
pub fn linear_to_dbfs(val: f32) -> f32 {
    if val <= 1e-6 {
        -144.0
    } else {
        20.0 * val.log10()
    }
}

/// Full RTA telemetry snapshot parsed from a 100-band `/meters/4` blob.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RtaSnapshot {
    /// Aggregated 4-band energy averages.
    pub bands: RtaBands,
    /// Peak frequency (in Hz) detected in the `800..=3500 Hz` vocal presence range, if above floor.
    pub vocal_peak_hz: Option<f32>,
    /// Linear energy at `vocal_peak_hz`.
    pub vocal_peak_energy: f32,
}

/// Maps a 0-based RTA bin index (`0..total_bins`) to approximate center frequency in Hz (`20..=20000 Hz`).
pub fn rta_bin_to_hz(bin: usize, total_bins: usize) -> f32 {
    if total_bins <= 1 {
        return 20.0;
    }
    let ratio = (bin.min(total_bins - 1) as f32) / ((total_bins - 1) as f32);
    let min_l = 20.0f32.log10();
    let max_l = 20000.0f32.log10();
    10.0f32.powf(min_l + ratio * (max_l - min_l))
}

/// Parses a `/meters/4` RTA blob into 4-band averages plus the peak vocal-range frequency (`800..=3500 Hz`).
pub fn parse_rta_snapshot(data: &[u8]) -> Option<RtaSnapshot> {
    let bins = parse_meter_blob(data);
    let bands = parse_rta_bands(data)?;
    let n = bins.len();

    let mut best_hz = None;
    let mut best_val = 0.0f32;
    for (i, &val) in bins.iter().enumerate() {
        let hz = rta_bin_to_hz(i, n);
        if (800.0..=3500.0).contains(&hz) && val > best_val {
            best_val = val;
            best_hz = Some(hz);
        }
    }

    let vocal_peak_hz = if best_val > 0.02 { best_hz } else { None };
    Some(RtaSnapshot {
        bands,
        vocal_peak_hz,
        vocal_peak_energy: best_val,
    })
}

/// Aggregates a 100-band `/meters/4` RTA blob into 4 perceptual bands (`sub_bass`, `low_mid`, `vocal_mid`, `air`).
pub fn parse_rta_bands(data: &[u8]) -> Option<RtaBands> {
    let bins = parse_meter_blob(data);
    if bins.len() < 16 {
        return None;
    }
    let n = bins.len();
    let b1 = (n * 22 / 100).max(1);
    let b2 = (n * 45 / 100).max(b1 + 1);
    let b3 = (n * 78 / 100).max(b2 + 1);

    let avg = |slice: &[f32]| -> f32 {
        if slice.is_empty() {
            0.0
        } else {
            slice.iter().copied().sum::<f32>() / slice.len() as f32
        }
    };

    Some(RtaBands {
        sub_bass: avg(&bins[..b1]),
        low_mid: avg(&bins[b1..b2]),
        vocal_mid: avg(&bins[b2..b3]),
        air: avg(&bins[b3..]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_blob(vals: &[f32]) -> Vec<u8> {
        let mut blob = Vec::with_capacity(4 + vals.len() * 4);
        blob.extend_from_slice(&(vals.len() as u32).to_le_bytes());
        for v in vals {
            blob.extend_from_slice(&v.to_le_bytes());
        }
        blob
    }

    #[test]
    fn test_parse_meter_blob_and_index() {
        let blob = make_blob(&[0.25, 0.5, 1.0]);
        let parsed = parse_meter_blob(&blob);
        assert_eq!(parsed.len(), 3);
        assert!((parsed[1] - 0.5).abs() < 1e-6);
        assert!((meter_value_at(&blob, 2).unwrap() - 1.0).abs() < 1e-6);
        assert!(meter_value_at(&blob, 3).is_none());
    }

    #[test]
    fn test_truncated_blob_and_dbfs() {
        assert!(parse_meter_blob(&[1, 2]).is_empty());
        assert_eq!(linear_to_dbfs(0.0), -144.0);
        assert!((linear_to_dbfs(1.0) - 0.0).abs() < 0.01);
        assert!((linear_to_dbfs(0.1) - -20.0).abs() < 0.05);
    }

    #[test]
    fn test_parse_rta_bands() {
        let mut vals = vec![0.1; 100];
        for v in &mut vals[0..22] {
            *v = 0.8;
        }
        let blob = make_blob(&vals);
        let rta = parse_rta_bands(&blob).unwrap();
        assert!((rta.sub_bass - 0.8).abs() < 1e-5);
        assert!((rta.vocal_mid - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_rta_bin_to_hz_and_snapshot() {
        assert!((rta_bin_to_hz(0, 100) - 20.0).abs() < 0.01);
        assert!((rta_bin_to_hz(99, 100) - 20000.0).abs() < 0.1);

        let mut vals = vec![0.01; 100];
        // Bin 66 on 0..99 log scale: 20 * 10^(3 * 66/99) = 20 * 10^2 = 2000 Hz
        vals[66] = 0.72;
        let blob = make_blob(&vals);
        let snap = parse_rta_snapshot(&blob).unwrap();
        let peak = snap.vocal_peak_hz.expect("Expected vocal peak");
        assert!((peak - 2000.0).abs() < 1.0);
        assert!((snap.vocal_peak_energy - 0.72).abs() < 1e-5);
    }
}
