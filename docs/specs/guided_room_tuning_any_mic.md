# Proposal: Guided Room Tuning Using "What We Have" Microphones

## 1. Problem & Constraints
Small venues and volunteer sound teams rarely own a calibrated omnidirectional RTA measurement mic (like an ECM8000) or an external audio interface running Smaart/REW. However, they **always** have:
- Everyday stage/desk microphones plugged directly into the console (`SM58`, `Beta58`, small-diaphragm lectern condensers, or even a laptop/phone mic via aux).
- The console's built-in **Pink Noise Oscillator** (`/config/osc`) and **100-band `/meters/4` RTA** (or `cpal` host audio input on Wing/X-Air/offline).
- Existing safety/tuning primitives in this workspace (`x32_system_tune`, `x32_lib::meters::parse_rta_snapshot`, `x32_lib::safety::ConsoleSnapshot`).

**Core Design Goal:** Use the mic you already have on stage to tame obvious room boom and harsh resonances **safely and repeatably**, while preventing non-flat vocal mics from tricking the room EQ into cutting the wrong frequencies.

---

## 2. Why Everyday Mics Need Compensation (and Where the Limits Are)
- A dynamic vocal mic (e.g., `SM58`) has an intentional **+4 to +6 dB presence bump around 3–5 kHz** and **steep roll-off below 100 Hz** (plus proximity boost if held close).
- Without compensation, an automated room tuner would think the room is "+5 dB harsh at 4 kHz" and cut 4 kHz out of the PA — leaving the room dull and muddy.
- **Honest Acoustic Guardrail:** Non-measurement mics are **not** phase-accurate or valid above ~8–10 kHz (off-axis beaming) or below ~70 Hz. Therefore, guided calibration must:
  1. Apply an **approximate inverse mic profile** across `80 Hz – 8 kHz`.
  2. Never touch `< 70 Hz` sub-bass or `> 8 kHz` air with automated cuts.
  3. Apply **cut-only** Main/Bus PEQ notches (never boosts, capped at `-6.0 dB` per band and max 3–4 bands).

---

## 3. Proposed Calibration Set ("What We Have" Profiles)

| Profile (`MicProfile`) | Typical Hardware | Compensated Region (`80 Hz – 8 kHz`) | Guardrails |
| :--- | :--- | :--- | :--- |
| **`Sm58`** | Shure SM58 / PGA58 / standard cardioid dynamic | Undo `3–5 kHz` presence hump (`-4 dB`), mild `100 Hz` roll-off offset (`+2 dB`) | Hold at arm's length aimed up ~45° (away from floor bounce); ignore `< 80 Hz` & `> 7 kHz` |
| **`Beta58`** | Supercardioid vocal dynamic (Beta 58A, e945) | Undo `4–6 kHz` presence peak (`-5 dB`) & `200 Hz` body dip | Narrower pickup: aim halfway between L & R speakers |
| **`Condenser`** | Lectern / choir / acoustic SDC | Mild `+2 dB` HF tilt compensation above `4 kHz` | Watch phantom `+48V` check in Pre-Service Check |
| **`LaptopPhone`** | Built-in laptop mic (`cpal`) or TRRS phone into Aux | Strong `< 150 Hz` roll-off ignore + `2–4 kHz` voice-band flatten | Limited to `150 Hz – 6 kHz` room boom/honk detection |
| **`Flat`** | Calibrated measurement mic (ECM8000, M30, etc.) | `0 dB` flat offset | Full `40 Hz – 12 kHz` range enabled |

---

## 4. Guided 5-Step Volunteer Workflow (Extension to `x32_system_tune`)

1. **Step 1 — Safety Snapshot & Mic Selection:**
   - Capture `ConsoleSnapshot` of Main LR / target bus EQ + oscillator settings (`[U]ndo` / auto-restore on abort).
   - Volunteer picks mic channel (e.g., `Ch 1`) and `MicProfile` (`SM58`, `Beta58`, etc.); tool ensures that mic channel is **unrouted from Main LR** during pink noise so it cannot feed back!
2. **Step 2 — Ambient Floor & EBU R128 Oscillator Check:**
   - Measure 2 seconds of room silence (HVAC/crowd baseline), then ramp pink noise safely up to the `MAX_EBU_R128_LEVEL` (`-18 dBFS` / `0.25`) cap until SNR is `>= 15 dB` above ambient.
3. **Step 3 — "Walk the Room" 3-to-5 Point Spatial Averaging:**
   - Prompts the volunteer: *"Stand at Front-Center — press [Space] to sample 3s"*, *"Mid-Left"*, *"Mid-Right"*, *"Back-Center"*.
   - Averages the `/meters/4` 100-band RTA magnitude across all positions (minimizing single-spot standing waves and comb filtering).
4. **Step 4 — Cut-Only Room Resonance Detection & Preview:**
   - Subtracts the selected `MicProfile` inverse curve and a gentle `-1 dB/octave` live-room target tilt from the averaged RTA spectrum.
   - Identifies the top **3–4 narrow/medium resonance peaks** exceeding `+3.0 dB` within `80 Hz – 6 kHz` (e.g., `125 Hz` room boom, `400 Hz` boxiness, `2.5 kHz` honk).
   - Displays proposed PEQ cuts (`Freq`, `Q`, `Gain` capped at `-6.0 dB`) for confirmation — **zero boosts**.
5. **Step 5 — `[B]ypass` vs `[T]uned` A/B Listening Check & Save:**
   - Volunteer plays familiar reference music or speaks on stage and toggles `[B]` (flat) vs `[T]` (tuned) live, then presses `[S]` to keep & save snapshot or `[U]` to revert completely.

---

## 5. Planned Implementation Scope (If Approved)
- **`libs/x32_lib/src/meters.rs`:** Add `MicProfile`, `rta_band_compensate(profile, bins)`, and `suggest_cut_only_peq(averaged_bins, profile, max_bands)` (pure unit-testable DSP math).
- **`apps/x32_system_tune`:** Add `--mic-channel`, `--mic-profile`, and the interactive 5-step guided calibration wizard with `[Space]` multi-point capture, `[B]`/`[T]` A/B toggle, and `ConsoleSnapshot` undo.
- **`apps/x32_volunteer`:** Expose `[R]oom Tune` pre-service shortcut or CLI handoff.
