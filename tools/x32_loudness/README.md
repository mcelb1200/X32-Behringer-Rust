# X32 Loudness: Dynamic Equal-Loudness (Fletcher-Munson) EQ Control

`x32_loudness` is a command-line tool that dynamically adjusts the frequency response of the Behringer X32/Midas M32 main outputs based on fader level (equal-loudness / Fletcher-Munson contours). This preserves perceived tonal balance (bass/treble fullness) at lower monitoring volumes.

It supports three modes (PEQ, GEQ, DEQ) and allows offsetting calculations with static room correction EQ data.

---

## Installation & Compilation
From the repository root:
```bash
cargo build --release -p x32_loudness
```

---

## 1. Subcommand: Calibrate
Calibrates the system without needing a measurement microphone.
It uses the X32 internal oscillator to send pink noise at a known level (`-18 dBFS`), and prompts the user to adjust the fader to a comfortable conversation level (`~60 dBA SPL` by default).

```bash
./target/release/x32_loudness --ip 192.168.1.50 calibrate --target-db 60
```

*   Saves the calculated room acoustics offset $C_{room}$ to `.x32_loudness.json`.

---

## 2. Subcommand: Run (Daemon)
Runs the background listener daemon. It subscribes to fader changes and dynamically updates the target EQ gains in real-time.

### Mode A: Main Stereo PEQ (Zero FX slots used)
Uses the main output 6-band parametric EQ (Bands 1 and 6):
```bash
./target/release/x32_loudness --ip 192.168.1.50 --mode peq run
```

### Mode B: Graphic EQ / TruEQ (Preserves FX 1-4)
Uses a 31-band graphic EQ inserted on the Main bus (default slot 5):
```bash
./target/release/x32_loudness --ip 192.168.1.50 --mode geq --slot 5 run
```

### Mode C: Dynamic EQ (High Precision)
Uses a 4-band Stereo DEQ inserted on the Main bus (default slot 1):
```bash
./target/release/x32_loudness --ip 192.168.1.50 --mode deq --slot 1 run
```

---

## Room Correction Offset Integration
You can pass a JSON file containing static room correction EQ offsets using the `--room-eq` flag. These offsets will be summed with the dynamic equal-loudness contour curves before gains are clamped and written to the console:

```bash
./target/release/x32_loudness --ip 192.168.1.50 --room-eq room_correction.json run
```

### Example `room_correction.json` Format:
```json
{
  "peq": [1.5, 0.0, 0.0, 0.0, 0.0, -2.0],
  "geq": [
    0.0, 0.0, -1.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
  ],
  "deq": [-1.0, 0.0, 2.0, 0.5]
}
```
*(PEQ array length = 6, GEQ array length = 31, DEQ array length = 4)*
