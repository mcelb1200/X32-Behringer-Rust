# `x32_tapw`

An interactive Terminal UI (TUI) for setting X32 delay tap tempo.

## 🚀 Features

*   **Visual Feedback:** High-refresh-rate level meters and BPM display using `ratatui`.
*   **Dual Modes:** Manual tapping via `Space/Enter` or automatic detection based on configurable level thresholds.
*   **Safety Heartbeat:** Automatically manages the `/xremote` subscription to ensure consistent console feedback.

## 🛠️ Usage

```bash
# Start the TUI
./x32_tapw --ip 192.168.1.50 --slot 1
```

### Controls
*   `Space / Enter`: Manual tap.
*   `T`: Toggle Auto-Tap mode.
*   `Up/Down`: Adjust Auto-Tap threshold.
*   `Q`: Quit.
