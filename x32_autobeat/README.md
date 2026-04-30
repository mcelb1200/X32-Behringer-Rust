# `x32_autobeat`

An advanced live-performance tool for automatic beat detection and effect synchronization.

## 🚀 Features

*   **Real-time Detection:** Analyzes audio from system input or monitors OSC level meters to calculate BPM.
*   **Dual Algorithms:** Supports standard Energy-based detection and Spectral Flux for more complex rhythms.
*   **Full Effect Sync:** Automatically updates Delays, Reverbs, and Modulation effects across all 8 FX slots.
*   **Musical Theory Engine:** Intelligent subdivision and style mapping (Standard, Tight, Natural, etc.).
*   **Hardware Integration:** Feedback via console scribble strips and control via assignable encoders.

## 🛠️ Usage

```bash
# Start with default settings (Auto-detect IP)
./x32_autobeat --ip 192.168.1.50 --channel 1

# List available audio devices
./x32_autobeat list-devices
```

### Controls (TUI)
*   `S`: Cycle through FX slots.
*   `A`: Switch between Energy and Spectral algorithms.
*   `P`: Panic (Clear all delays).
*   `Up/Down`: Cycle through subdivisions (1/4, 1/8, etc.).
