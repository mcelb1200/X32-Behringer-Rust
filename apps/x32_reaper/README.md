# `x32_reaper`

A high-performance bidirectional bridge between Behringer X32 consoles and the Reaper DAW.

## 🚀 Features

*   **Synchronized Controls:** Bidirectional sync for Faders, Mutes, and Pans.
*   **Transport Control:** Control Reaper's transport (Play, Stop, Record) directly from the console.
*   **Bank Switching:** Supports banking through large Reaper projects using console controls.
*   **Bolt Optimized:** Hand-optimized serialization loop to ensure no "zipper noise" or lag during fast fader movements.

## 🛠️ Usage

```bash
# Connect to X32 and Reaper (running OSC on default ports)
./x32_reaper --ip 192.168.1.64 --reaper-ip 127.0.0.1
```

## 🧪 Testing
Run integration tests with:
```bash
cargo test -p x32_reaper
```
