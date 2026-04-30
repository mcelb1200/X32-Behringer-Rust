# Beat Detection Algorithms Research

## Selected Algorithm: Energy Moving Average (Primary)

### Description
The Energy Moving Average algorithm is a standard, computationally efficient method for beat detection in real-time audio signals. It works by:
1.  **Chunking:** Dividing the audio stream into small blocks (e.g., 1024 samples).
2.  **Energy Calculation:** Computing the Root Mean Square (RMS) or simple average energy of each block.
3.  **Smoothing:** Maintaining a "Moving Average" of the energy history (e.g., last 1 second).
4.  **Comparison:** Comparing the *instant* energy of the current block to the *average* energy.
5.  **Thresholding:** If `Instant > Average * Constant`, a beat onset is registered.

### Why Selected?
*   **Efficiency:** Extremely low CPU usage, suitable for running alongside other tools.
*   **Responsiveness:** Very low latency.
*   **Suitability:** The user requirement specifically requested a "Moving Average" approach. It works particularly well for music with pronounced rhythmic pulses (drums), which is the primary use case for syncing delays.

---

## Alternative: Spectral Flux / Onset Detection (TODO)

### Concept
Unlike the Energy method which looks at total volume, Spectral Flux analyzes changes in the *frequency domain*. It performs an FFT (Fast Fourier Transform) on the audio chunks and measures how much the magnitude of each frequency bin changes between frames.

### Implementation Plan (Future)
1.  **FFT:** Apply a Window function (Hann) and compute FFT.
2.  **Flux Calculation:** Sum the positive differences in magnitude for each bin compared to the previous frame. `Flux = Sum( max(0, Mag[i] - PrevMag[i]) )`.
3.  **Peak Picking:** Apply a similar moving average threshold to the Flux signal.

### Pros/Cons
*   **Pros:** Much better at detecting beats in music without strong volume pulses (e.g., heavily compressed rock, distorted guitars) or where the beat is defined by a tonal change (hi-hats) rather than just a kick drum.
*   **Cons:** Higher CPU load (FFT), slightly higher latency due to windowing.

---

## Rejected: Comb Filter Bank

### Concept
A Comb Filter Bank algorithm passes the audio signal through a series of resonating filters tuned to specific BPMs. The filter with the highest energy output "wins", indicating the tempo.

### Why Rejected?
*   **Complexity:** Requires a complex bank of filters or matrix operations.
*   **Latency:** Requires a significant buffer of audio history to "lock on" to a tempo accurately. It is less "real-time" responsive to sudden tempo changes than onset detection.
*   **Overkill:** For a utility syncing delay effects (which often just need a tap tempo), the computational cost does not justify the marginal accuracy gain over a well-tuned onset detector for live usage.
