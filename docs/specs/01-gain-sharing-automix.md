# PRD: Progressive Gain Sharing Automixer

## 1. Executive Summary
Replace the current threshold-based gating in `x32_automix` with a professional-grade **Gain Sharing** algorithm (Dugan Pattern). This ensures transparent transitions, consistent system gain, and reduced background noise in multi-microphone environments.

## 2. Technical Specification

### 2.1 Algorithm (Gain Sharing)
The gain $G_i$ for each microphone $i$ is calculated as:
$$G_i = \frac{L_i}{\sum_{j=1}^{n} L_j}$$
Where:
*   $L_i$ is the linear level (RMS) of the current microphone.
*   $\sum L_j$ is the sum of linear levels of all microphones in the automix group.

### 2.2 Core Components
*   **Level Monitor:** Real-time extraction of RMS levels via `/meters/1`.
*   **Coefficient Engine:** High-frequency calculation of gain shared weights.
*   **Fader Controller:** Asynchronous dispatch of `/ch/XX/mix/fader` commands.

## 3. Gaps & Limitations
*   **Network Jitter:** X32 OSC updates via UDP can have jitter. Rapid fader changes might cause "stepping" noise if not smoothed.
*   **Meter Resolution:** The 4-byte float in `/meters/1` is sufficient, but polling rate is capped at ~20-50ms.

## 4. Alternative Implementations & Redundancy

### Mode 1: Virtual Gain Sharing (Primary)
*   **Approach:** Tool calculates gain and sends fader moves directly to X32.
*   **User Value:** Works on all 32 channels simultaneously; no group limitations.
*   **Redundancy:** Provides high-resolution control but increases network traffic.

### Mode 2: Hardware-Assisted Weights (Failover/Efficiency)
*   **Approach:** Use X32 internal "Automix" groups but control the **Weights** dynamically via the tool.
*   **User Value:** Offloads DSP to the hardware; zero-latency gain sharing.
*   **Redundancy:** Acts as a failover mode if network bandwidth is constrained; limited to 8 channels per hardware group.

## 5. Directory Structure
```text
apps/x32_automix/
├── src/
│   ├── engine/
│   │   ├── dugan.rs      <-- New: Gain sharing logic
│   │   └── nom.rs        <-- Legacy: Threshold gating
│   ├── network/
│   │   └── batcher.rs    <-- New: Batch OSC fader updates
│   └── main.rs
```

## 6. Verification Plan
*   **Unit Test:** Verify gain sharing coefficients sum to 1.0.
*   **Integration Test:** Use `x32_core` to simulate 4 mics at different levels and verify fader positions.
