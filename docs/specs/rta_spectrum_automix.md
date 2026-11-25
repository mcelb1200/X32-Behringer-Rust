# Spec: RTA-Assisted Spectrum-Aware Mixbus Automixing (Future Enhancement)

## 1. Motivation
`x32_lib::automix::SpectrumAutomixer` currently prevents bass-heavy kick/drum transients from pumping vocals using static `SpectralRole` profiles (`low_end_compensation`, pairwise `overlap` matrix) and dynamic vocal-pocket EQ carving (`PocketEqConfig`). While effective out of the box across X32, Wing, and X-Air consoles, static role profiles cannot detect when a live room or guest band shifts energy abnormally (e.g., an unusually bright guitar cab colliding at `3.2 kHz`, or sub-heavy synth pads masking bass).

## 2. X32 `/meters/4` Real-Time Analyzer (RTA) Feed
Behringer X32/M32 consoles stream a 100-band FFT spectrum via `/meters/4` (`0.0..=1.0` linear per 1/3-octave-like bin, spanning ~20 Hz – 20 kHz, little-endian `f32` after the 4-byte length header).

### Existing Hook in `x32_lib`
- [`x32_lib::meters::RtaBands`](../../libs/x32_lib/src/meters.rs) aggregates `/meters/4` into 4 perceptual bands:
  - `sub_bass` (~20–120 Hz)
  - `low_mid` (~120–500 Hz)
  - `vocal_mid` (~500 Hz – 4 kHz)
  - `air` (~4–20 kHz)
- [`x32_lib::automix::SpectrumAutomixer::set_rta_hint`](../../libs/x32_lib/src/automix.rs) already accepts `Option<RtaBands>` so future RTA integration requires zero breaking API changes.

## 3. Planned Pipeline
1. **Optional RTA Subscription (X32/M32 only):**
   - Subscribe to `/meters/4` pointing at Main LR or the Vocal/Band bus (`/meters` arg `"meters/4"`, target index).
   - Parse into `RtaBands` (plus peak-bin frequency lookup for adaptive pocket EQ).
2. **Dynamic Masking Ratio:**
   - Compare `vocal_mid` energy against `Band`/`Keys` midrange when `Vocals` are active.
   - If `sub_bass > 2.0 * vocal_mid` (bass-heavy kick/synth build-up), bias `Drums`/`Bass` `low_end_compensation` down slightly without pulling `Vocals` down.
   - If `Band` `vocal_mid` exceeds `Vocals` `vocal_mid`, increase `overlap(Vocals, Band)` up to `0.95` and allow deeper `pocket_eq` dip (still strictly bounded by `max_dip_db`).
3. **Adaptive Pocket EQ Center Frequency (`/bus/XX/eq/{band}/f`):**
   - Optionally track the peak vocal fundamental/presence bin (`800 Hz – 3.5 kHz`) and slew the competing bus's `PocketEqConfig` band frequency smoothly (`<= 5%` change per second, hysteresis deadband `+/- 150 Hz`) so the carve follows the actual singer's timbre.
4. **Stage-Safety Guardrails:**
   - All RTA-driven gains and frequency slews must remain strictly clamped to `[0, max_dip_db]` and `max_fader_limit_db`.
   - Disable RTA adaptation automatically and fall back to static `SpectralRole` profiles if `/meters/4` stalls for `> 500 ms` or on Wing/X-Air models without `/meters/4`.
