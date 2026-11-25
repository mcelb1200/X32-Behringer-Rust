# `x32_volunteer` — Simplified Volunteer Mode Dashboard

A simplified, guardrail-protected Terminal UI (TUI) for volunteer sound operators on Behringer X32/M32, Wing, and X-Air (`XR18`, `XR16`, `XR12`) digital mixers.

## Quick Start

```bash
# Auto-discover mixer on LAN (or pass --ip 192.168.1.50 -M X32)
cargo run -p x32_volunteer -- --ip auto --channels ch1,ch2,ch3,ch4,dca1 --max-fader-limit-db -3.0
```

Or copy [`docs/examples/x32_config.volunteer.json`](../../docs/examples/x32_config.volunteer.json) to `x32_config.json` in your working directory (or pass `--config path/to/x32_config.json`) to configure:
- **`max_fader_limit_db`**, **`visible_channels`**, **`visible_dcas`**
- **`mixbus_automix`**: Spectrum-aware Dugan mixbus balancing (`vocals`, `speech`, `band`, `drums`, `bass`, `keys`) with optional per-bus **Dynamic EQ Vocal Pocket** carving (`pocket_eq`) so bass-heavy drums do not pump vocals.
- **`stream_monitor`**: Broadcast/livestream RMS & peak loudness monitoring (`SILENT`, `QUIET`, `GOOD`, `HOT`, `CLIP`), defaulting to monitor-only (`"auto_trim": false`).

## Keyboard Controls

| Key | Action |
| :--- | :--- |
| `←` / `→` (or `h` / `l`) | Select previous / next channel or DCA |
| `Space` / `Enter` | Toggle Mute on the selected channel |
| `↑` / `↓` (or `k` / `j`) | Nudge selected fader up/down (clamped to `--max-fader-limit-db`) |
| `a` | Toggle Spectrum-Aware Mixbus Automix + Dynamic Pocket EQ (restores EQ to `0 dB` when turned off) |
| `c` | **Pre-Service Check:** Verify console online status, Main LR mute state, and save `volunteer_backup_<epoch>.json` |
| `u` | **Undo:** Restore the last captured pre-Panic / pre-MuteAll / pre-Check snapshot |
| `m` | Mute all monitored channels (captures snapshot for `[U]ndo`) |
| `p` (press twice within 3s) | **Panic:** Smooth exponential fade-out + mute all monitored channels (any other key cancels; `[U]ndo` restores) |
| `q` / `Esc` | Quit |

## Header & Alert Indicators

- **Connection:** `CONNECTING`, `ONLINE`, or `OFFLINE` (triggers `PROBLEM` if no OSC response is received for >3 seconds; renews `/xremote` every 8s).
- **Main LR:** Model-accurate Main LR mute state (`LIVE` / `MUTED`) and fader level across X32, Wing, and X-Air.
- **Automix / Stream:** Live `Automix: ON/OFF` state and `Stream: GOOD / QUIET / HOT / CLIP / SILENT` dBFS readout when configured.
- **Health Status:** `🟢 ALL OK`, `🟡 CAUTION`, or `🔴 PROBLEM`.

## Future Roadmap
See [docs/specs/rta_spectrum_automix.md](../../docs/specs/rta_spectrum_automix.md) for the planned X32 `/meters/4` 100-band RTA adaptive weighting enhancement.
