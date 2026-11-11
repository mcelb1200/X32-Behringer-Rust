# `x32_tapw`

An interactive Terminal UI (TUI) for setting delay tap tempo across the Behringer X32, Midas M32, and Wing family of digital mixers (`X32`, `Wing`, `XR18`, `XR16`, `XR12`).

## 🚀 Features

*   **MixerModel Family Support:** Compatible with `X32`, `Wing`, `XR18`, `XR16`, and `XR12` consoles with model-aware target port resolution (10023 for X32, 2222 for Wing, 10024 for XR series).
*   **Visual Feedback:** High-refresh-rate level meters and BPM display using `ratatui`.
*   **Dual Modes:** Manual tapping via `Enter` or automatic detection based on configurable level thresholds.
*   **Safety Heartbeat:** Automatically manages the `/xremote` subscription to ensure consistent console feedback.

## 🛠️ Usage

```bash
# Start the TUI
./x32_tapw -M Wing --ip 192.168.1.50 --slot 1
```

### Controls
*   `Enter`: Manual tap.
*   `a`: Toggle Auto-Tap mode.
*   `i`: Edit IP address.
*   `s`: Edit FX slot.
*   `c`: Edit Channel.
*   `e`: Edit Sensitivity threshold.
*   `q` / `Esc`: Quit.
