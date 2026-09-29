//! Profile matching for Auto-Gain based on channel name and icon metadata.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetProfile {
    pub name: &'static str,
    pub target_dbfs: f32,
    pub peak_target_dbfs: f32,
}

impl Default for TargetProfile {
    fn default() -> Self {
        Self {
            name: "Default / Unknown",
            target_dbfs: -18.0,
            peak_target_dbfs: -10.0,
        }
    }
}

/// Matches channel scribble strip name or icon ID to an instrument target profile.
pub fn match_instrument_profile(name: &str, icon_id: i32) -> TargetProfile {
    let name_lower = name.to_lowercase();

    // 1. Name matching (takes precedence)
    if name_lower.contains("kick") || name_lower.contains("bass drum") {
        return TargetProfile {
            name: "Kick Drum",
            target_dbfs: -12.0,
            peak_target_dbfs: -6.0,
        };
    }

    if name_lower.contains("snare") {
        return TargetProfile {
            name: "Snare Drum",
            target_dbfs: -14.0,
            peak_target_dbfs: -6.0,
        };
    }

    if name_lower.contains("tom") {
        return TargetProfile {
            name: "Toms",
            target_dbfs: -14.0,
            peak_target_dbfs: -6.0,
        };
    }

    if name_lower.contains("oh")
        || name_lower.contains("overhead")
        || name_lower.contains("cymbal")
        || name_lower.contains("ride")
        || name_lower.contains("hat")
    {
        return TargetProfile {
            name: "Overhead / Cymbal",
            target_dbfs: -18.0,
            peak_target_dbfs: -10.0,
        };
    }

    if name_lower.contains("bass") || name_lower.contains("btr") {
        return TargetProfile {
            name: "Bass Guitar",
            target_dbfs: -14.0,
            peak_target_dbfs: -8.0,
        };
    }

    if name_lower.contains("egtr")
        || name_lower.contains("elec")
        || name_lower.contains("guitar amp")
        || name_lower.contains("amp")
    {
        return TargetProfile {
            name: "Electric Guitar",
            target_dbfs: -16.0,
            peak_target_dbfs: -8.0,
        };
    }

    if name_lower.contains("agtr")
        || name_lower.contains("acoustic")
        || name_lower.contains("ac gtr")
    {
        return TargetProfile {
            name: "Acoustic Guitar",
            target_dbfs: -18.0,
            peak_target_dbfs: -10.0,
        };
    }

    if name_lower.contains("piano")
        || name_lower.contains("key")
        || name_lower.contains("syn")
        || name_lower.contains("organ")
    {
        return TargetProfile {
            name: "Piano / Keys",
            target_dbfs: -18.0,
            peak_target_dbfs: -8.0,
        };
    }

    if name_lower.contains("pastor")
        || name_lower.contains("speech")
        || name_lower.contains("lectern")
        || name_lower.contains("podium")
        || name_lower.contains("speak")
    {
        return TargetProfile {
            name: "Speech / Lectern",
            target_dbfs: -20.0,
            peak_target_dbfs: -10.0,
        };
    }

    if name_lower.contains("lav") || name_lower.contains("lapel") {
        return TargetProfile {
            name: "Wireless Lavalier",
            target_dbfs: -22.0,
            peak_target_dbfs: -12.0,
        };
    }

    if name_lower.contains("choir") || name_lower.contains("ensemble") {
        return TargetProfile {
            name: "Choir / Ensemble",
            target_dbfs: -20.0,
            peak_target_dbfs: -10.0,
        };
    }

    if name_lower.contains("vox")
        || name_lower.contains("vocal")
        || name_lower.contains("mic")
        || name_lower.contains("lead")
        || name_lower.contains("sing")
    {
        return TargetProfile {
            name: "Lead Vocal",
            target_dbfs: -18.0,
            peak_target_dbfs: -8.0,
        };
    }

    if name_lower.contains("dj")
        || name_lower.contains("play")
        || name_lower.contains("aux")
        || name_lower.contains("track")
        || name_lower.contains("cd")
    {
        return TargetProfile {
            name: "DJ / Playback",
            target_dbfs: -14.0,
            peak_target_dbfs: -6.0,
        };
    }

    // 2. Icon ID fallback matching
    match icon_id {
        1..=10 => TargetProfile {
            name: "Drums (Icon)",
            target_dbfs: -14.0,
            peak_target_dbfs: -6.0,
        },
        11..=20 => TargetProfile {
            name: "Guitars / Bass (Icon)",
            target_dbfs: -16.0,
            peak_target_dbfs: -8.0,
        },
        21..=30 => TargetProfile {
            name: "Vocals / Microphones (Icon)",
            target_dbfs: -18.0,
            peak_target_dbfs: -8.0,
        },
        _ => TargetProfile::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_instrument_profile_by_name() {
        assert_eq!(match_instrument_profile("Kick", 0).name, "Kick Drum");
        assert_eq!(match_instrument_profile("Snare Top", 0).target_dbfs, -14.0);
        assert_eq!(match_instrument_profile("Pastor Bob", 0).target_dbfs, -20.0);
        assert_eq!(match_instrument_profile("Acoustic Gtr", 0).target_dbfs, -18.0);
        assert_eq!(match_instrument_profile("Wireless Lav", 0).target_dbfs, -22.0);
        assert_eq!(match_instrument_profile("Tracks L", 0).name, "DJ / Playback");
    }

    #[test]
    fn test_match_instrument_profile_by_icon() {
        assert_eq!(match_instrument_profile("Ch 01", 5).name, "Drums (Icon)");
        assert_eq!(match_instrument_profile("Ch 02", 15).name, "Guitars / Bass (Icon)");
        assert_eq!(match_instrument_profile("Ch 03", 25).name, "Vocals / Microphones (Icon)");
        assert_eq!(match_instrument_profile("Ch 04", 0).name, "Default / Unknown");
    }
}
