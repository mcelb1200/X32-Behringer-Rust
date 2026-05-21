# PRD: SPL-Aware Dynamic EQ Normalization

## 1. Executive Summary
Implement **Fletcher-Munson EQ Curve** compensation. As the master volume (SPL) is reduced, the system automatically boosts Low (Bass) and High (Treble) frequencies to maintain a consistent perceived tonal balance, adhering to ISO 226:2003 Equal-Loudness Contours.

## 2. Technical Specification

### 2.1 SPL Proxy Model
Since the software cannot directly measure room SPL without a calibrated mic, it uses the **Master Fader position** as a proxy:
*   Reference Level (0dB on fader) = 100dB SPL (User Configurable).
*   Curve slope: Reciprocal of ISO 226 contours.

### 2.2 Control Targets
*   **Low Shelf:** `/main/st/eq/1/g` (Gain) and `/main/st/eq/1/f` (Freq).
*   **High Shelf:** `/main/st/eq/6/g` (Gain) and `/main/st/eq/6/f` (Freq).

## 3. Gaps & Limitations
*   **Calibration:** The "Reference SPL" is a guess unless calibrated by the user.
*   **EQ Competition:** If the user is already using the Main EQ for room tuning, the automation will overwrite those specific bands.

## 4. Alternative Implementations & Redundancy

### Strategy 1: Main Bus EQ Normalization (User Configurable)
*   **Approach:** Control the 6-band EQ on the Main Stereo Bus.
*   **User Value:** Simplest setup; works with standard L/R PA configurations.
*   **Redundancy:** Primary method for small setups.

### Strategy 2: Matrix Output Offset (Redundant/Professional)
*   **Approach:** Apply the compensation to a Matrix output fed by the Main Bus.
*   **User Value:** Keeps Main Bus EQ free for artistic tuning and room correction.
*   **Redundancy:** Preferred fail-safe for large touring systems to avoid accidental core-mix modification.

## 5. Directory Structure
```text
libs/x32_lib/src/
├── dsp/
│   ├── fletcher_munson.rs  <-- Curve interpolation logic
│   └── iso_226.json        <-- Static contour data points
├── command/
│   └── main_bus.rs         <-- EQ command generation
```

## 6. Verification Plan
*   **Unit Test:** Compare interpolated gains against standard ISO 226 tables.
*   **Integration Test:** Smoothly sweep Master Fader from -60dB to 0dB and verify EQ gain increments.
