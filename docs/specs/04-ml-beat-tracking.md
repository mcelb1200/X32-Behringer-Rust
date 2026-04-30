# PRD: Probabilistic Beat Tracking (Comb Filter Bank)

## 1. Executive Summary
Move from reactive "Peak Detection" to predictive **Probabilistic Beat Tracking**. This allows the suite to maintain rhythmic stability during syncopation, polyrhythms, and moments of silence, significantly reducing the "BPM Jitter" found in simple energy-based detectors.

## 2. Technical Specification

### 2.1 Algorithm (Comb Filter Bank)
*   Decompose incoming audio into multiple frequency bands.
*   Calculate the **Onset Strength Envelope** for each band.
*   Pass the envelope through a bank of parallel Comb Filters tuned from 60 to 180 BPM.
*   The filter with the highest "resonance" (energy output) represents the most probable BPM.

### 2.2 Probabilistic Model
*   Maintain a "BPM Hypothesis" set with confidence scores.
*   Use a temporal smoothing window (e.g., 4 bars) to prevent sudden jumps if a single transient is missed or a ghost note is played.

## 3. Gaps & Limitations
*   **CPU Overhead:** Parallel Comb Filters (FFT-based) are computationally expensive compared to Energy detectors.
*   **Latency:** Probabilistic models require a history of transients, introducing a "warm-up" period (usually 2-4 seconds) before a locked BPM is reported.

## 4. Alternative Implementations & Redundancy

### Mode 1: FFT-based Comb Bank (High Precision)
*   **Approach:** Use the `rustfft` crate for high-performance multi-band decomposition.
*   **User Value:** Most accurate for complex musical arrangements and syncopation.
*   **Redundancy:** Primary mode for dedicated control PCs.

### Mode 2: Energy-Zero RNN (Efficient Failover)
*   **Approach:** Use a tiny Recurrent Neural Network (RNN) for efficient onset prediction.
*   **User Value:** Ultra-low CPU usage; suitable for embedded/Raspberry Pi controllers.
*   **Redundancy:** Automatic failover if the FFT engine exceeds CPU thermal/usage thresholds.

## 5. Directory Structure
```text
libs/x32_lib/src/
├── dsp/
│   ├── fft/
│   │   └── analyzer.rs   <-- FFT wrappers
│   └── tracker/
│       ├── comb_bank.rs  <-- Core algorithm
│       └── model.rs      <-- Probability scores
```

## 6. Verification Plan
*   **Synthetic Test:** Feed the detector a steady metronome with missing pulses and verify it maintains the correct tempo.
*   **Stress Test:** Feed syncopated Jazz and Latin tracks and compare results against professional DAW beat-mappers.
