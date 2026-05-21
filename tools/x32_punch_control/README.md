# `x32_punch_control`

A precision automation tool for X32 consoles, synchronized via MIDI Time Code (MTC).

## 🚀 Features

*   **Automation Recording:** Records fader and mute changes to a custom `.xpc` binary format.
*   **DAW Sync:** Synchronizes playback and recording with any DAW emitting MTC.
*   **Punch-In/Out:** Allows for non-destructive overwriting of automation segments.
*   **Hardware Control:** Map X32 user-assignable buttons to record/play triggers.

## 🛠️ Usage

```bash
# Record automation to a file
./x32_punch_control --ip 192.168.1.50 --file song_v1.xpc --record

# Playback automation
./x32_punch_control --ip 192.168.1.50 --file song_v1.xpc --play
```

## 🧪 Testing
Run binary format tests with:
```bash
cargo test -p x32_punch_control
```
