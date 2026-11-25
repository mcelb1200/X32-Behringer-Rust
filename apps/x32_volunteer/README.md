# `x32_volunteer` — Simplified Volunteer Mode Dashboard

A simplified, guardrail-protected Terminal UI (TUI) for volunteer sound operators on Behringer X32/M32, Wing, and X-Air (`XR18`, `XR16`, `XR12`) digital mixers.

## Quick Start

```bash
# Monitor channels 1–4 and DCA 1 on an X32 at 192.168.1.50 with a -3 dB fader ceiling
cargo run -p x32_volunteer -- --ip 192.168.1.50 --model X32 --channels ch1,ch2,ch3,ch4,dca1 --max-fader-limit-db -3.0
```

Or using `x32_config.json` in the working directory (or `--config path/to/x32_config.json`):

```json
{
  "volunteer_mode": {
    "max_fader_limit_db": -3.0,
    "visible_channels": [1, 2, 3, 4],
    "visible_dcas": [1]
  }
}
```

## Keyboard Controls

| Key | Action |
| :--- | :--- |
| `←` / `→` (or `h` / `l`) | Select previous / next channel or DCA |
| `Space` / `Enter` | Toggle Mute on the selected channel |
| `↑` / `↓` (or `k` / `j`) | Nudge selected fader up/down (clamped to `--max-fader-limit-db`) |
| `m` | Mute all monitored channels |
| `p` (press twice within 3s) | **Panic:** Mute and zero all monitored channels (any other key cancels) |
| `q` / `Esc` | Quit |

## Header & Alert Indicators

- **Connection:** `CONNECTING`, `ONLINE`, or `OFFLINE` (triggers `PROBLEM` if no OSC response is received for >3 seconds).
- **Main LR:** Displays Main LR mute state (`LIVE` / `MUTED`) and fader level.
- **Health Status:**
  - `🟢 ALL OK` — Normal operation, no alerts.
  - `🟡 CAUTION` — A channel exceeds `--max-fader-limit-db`, Main LR is muted, or Panic is armed.
  - `🔴 PROBLEM` — A live channel is near clipping (`> -5.0 dBFS`) or the mixer is `OFFLINE`.
