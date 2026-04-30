# PRD: Phase-Synced Time Effects

## 1. Executive Summary
Synchronize the **Phase** (downbeat alignment) of X32 time-based effects (Delays, LFOs) with a detected audio transient. This ensures that echoes and modulation occur exactly on the beat, not just at the right speed.

## 2. Technical Specification

### 2.1 Transient Detection
*   Utilize `SpectralFluxDetector` in `x32_autobeat` to identify sudden energy shifts in the 50Hz-150Hz range (Kick Drum).
*   Timestamp the transient arrival with microsecond precision using `Instant::now()`.

### 2.2 Phase-Locked Loop (PLL) Logic
*   Internal oscillator maintains a virtual "beat clock".
*   For every detected transient, adjust the virtual clock phase using a proportional-integral (PI) controller to minimize the phase error ($\Delta\phi$).
*   When a "Sync" command is triggered, send a burst of OSC "Tap" or "Reset" commands timed to the virtual clock's downbeat.

## 3. Gaps & Limitations
*   **OSC Command Jitter:** Sending an OSC packet over UDP has a 1-5ms arrival jitter. Perfect sub-millisecond phase alignment is theoretically impossible via external OSC control.
*   **Hardware Buffer:** The X32's internal DSP cycle may align taps to its own 48kHz clock, causing slight rounding errors.

## 4. Alternative Implementations & Redundancy

### Method 1: Hard OSC Re-Tap (Global Sync)
*   **Approach:** Send a new "Tap" command exactly on the detected downbeat.
*   **User Value:** Universal compatibility; works with any delay effect on the console.
*   **Redundancy:** Provides a broad failover if specific FX algorithms do not support phase resets.

### Method 2: LFO Direct Reset (Precision Sync)
*   **Approach:** Use the `/fx/X/par/Y` reset command for specific modulation effects.
*   **User Value:** Audio-transparent; no pitch artifacts during resync.
*   **Redundancy:** Preferred for high-fidelity studio use; user-configurable based on FX slot loadout.

## 5. Directory Structure
```text
apps/x32_autobeat/src/
├── detection/
│   └── transient.rs    <-- Transient extraction logic
├── sync/
│   └── pll.rs          <-- PI controller for phase alignment
└── effects/
    └── syncer.rs       <-- Timed OSC dispatch
```

## 6. Verification Plan
*   **Unit Test:** Verify PI controller converges on a stable phase with noisy transient data.
*   **Hardware Test:** Use a loopback cable to record the delayed output and measure the delta between original kick and echo transient.
