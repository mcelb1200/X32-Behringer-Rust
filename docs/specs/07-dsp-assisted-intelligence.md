# PRD: DSP-Assisted Controller Intelligence

## 1. Executive Summary
Leverage the internal DSP capabilities of the X32 (Meters, RTA, Side-chains) to provide high-fidelity data to the controller application. This reduces the CPU load on the host machine and ensures that automation logic (like Autobeat or Automix) uses the exact same signal data as the console's internal processing.

## 2. Technical Specification

### 2.1 Virtual Side-chain Analysis
Instead of calculating audio RMS locally, use the console's internal **Dynamics Side-chain** to perform analysis.
*   **Mechanism:** Route a source channel to an unused FX slot (e.g., Precision Limiter). Query the FX gain reduction or input meter to determine transient timing.

### 2.2 RTA Data Harvesting
*   **Mechanism:** Query the high-resolution Real-Time Analyzer (RTA) data via `/meters/10` (RTA Peak) and `/meters/11` (RTA RMS).
*   **Benefit:** Provides frequency-domain data (up to 100 bands) without performing an FFT on the host machine.

### 2.3 Internal Delay Taps
*   **Mechanism:** Use the internal `tap` command of FX units to verify that the controller's software-detected BPM matches the hardware's internal clock.

## 3. Gaps & Limitations
*   **Polling Overhead:** Reading high-resolution RTA data via OSC can consume significant network bandwidth.
*   **FX Slot Availability:** DSP-assisted analysis requires the user to sacrifice an internal FX slot for the controller's use.

## 4. Alternative Implementations & Redundancy

### Level 1: Standard Meter Harvesting (Transparent)
*   **Approach:** Strictly use the standard `/meters/` endpoints for level data.
*   **User Value:** Requires no change to the existing mixer configuration or FX setup.
*   **Redundancy:** Default fallback if FX slots are fully utilized by the user.

### Level 2: "Deep Dive" DSP Side-chain (High Precision)
*   **Approach:** Tool automatically allocates an internal FX slot for precise transient detection.
*   **User Value:** Professional-grade beat tracking accuracy; bypasses channel fader dependencies.
*   **Redundancy:** Primary mode for critical time-sync applications (e.g., rhythmic delay sync).

## 5. Directory Structure
```text
libs/x32_core/src/
├── dsp/
│   ├── rta_harvester.rs  <-- RTA data extraction
│   └── meter_sync.rs    <-- Hardware/Software level sync
```

## 6. Verification Plan
*   **Accuracy Check:** Compare host-calculated FFT with console-provided RTA data.
*   **Bandwidth Stress:** Measure network impact of 50ms RTA updates.
