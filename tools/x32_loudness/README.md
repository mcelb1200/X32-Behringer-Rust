# x32_loudness User Documentation

An advanced mix automation utility for dynamic, equal-loudness (Fletcher-Munson) EQ compensation and room correction offsets on Behringer X32 and Midas M32 consoles.

---

## Layer 1: Quick Start

Get `x32_loudness` running in 60 seconds.

### 1. Compilation
Build the executable from the repository root:
```bash
cargo build --release -p x32_loudness
```
Executable will be generated at `./target/release/x32_loudness`.

### 2. Connect & Calibrate Room
Connect your computer to the same network as the X32 console. Run calibration to automatically calculate your room's acoustics constant ($C_{room}$) without needing a measurement microphone:
```bash
./target/release/x32_loudness --ip 192.168.1.50 calibrate --target-db 60
```
1. Press `ENTER` to activate the internal pink noise generator (sent to Main L/R at `-18 dBFS`).
2. Move the **Main Fader** on the console until the sound level in the room feels like a comfortable conversation volume (~60 dBA SPL).
3. Press `ENTER` to save the calibration config to `.x32_loudness.json` and turn off the tone.

### 3. Launch Dynamic EQ Daemon
Run the background listener in the default parametric EQ mode (uses Bands 1 and 6 on the Main output, zero FX slots required):
```bash
./target/release/x32_loudness --ip 192.168.1.50 --mode peq run
```
Now, adjusting the mixer's Main Fader will automatically scale the low-frequency and high-frequency shelving filters to maintain consistent perceived tonal balance.

---

## Layer 2: Volunteer User Guide

For sound board operators, venue staff, and live-event volunteers.

### What is Equal-Loudness Compensation?
The human ear is non-linear. At quiet listening levels, our hearing becomes significantly less sensitive to low frequencies (bass) and high frequencies (treble). This is why a mix that sounds warm and punchy at concert volume sounds thin, hollow, and distant when the master volume is turned down.

`x32_loudness` automatically compensates for this effect. As you pull down the Main fader, the tool gently boosts the low and high frequencies. As you push the fader back up to show volume, the EQ flattens back out.

### Calibration Safety
Calibration is simple and safe:
1. Always turn the main fader all the way down before running `calibrate`.
2. When the pink noise starts, slowly push the fader up. Stop when it sounds like a normal conversation.
3. The program automatically shuts off the noise generator the moment you press `ENTER`.

### EQ Modes
Select a mode that matches your console's FX slot availability:
*   **PEQ Mode (Default)**: Automatically adjusts the built-in 6-band Parametric EQ on the Main L/R channel. Band 1 becomes a low shelf and Band 6 becomes a high shelf. *Cost: Zero FX slots.*
*   **GEQ Mode**: Adjusts a 31-band graphic EQ inserted on the Main L/R channel. *Cost: 1 Graphic EQ insert (typically Slots 5-8).*
*   **DEQ Mode**: Adjusts a 4-band Stereo Dynamic EQ inserted on the Main L/R channel. *Cost: 1 FX rack space (Slots 1-4).*

---

## Layer 3: Power User Guide

For systems integrators, acousticians, and power users.

### Mathematical Foundations
The tool translates normalized fader values $f \in [0.0, 1.0]$ into decibels ($dB_F$) using the Behringer X32 fader curve approximation:
$$dB_F = \begin{cases}
  40.0 f - 30.0 & \text{if } f \ge 0.5 \\
  80.0 f - 50.0 & \text{if } 0.25 \le f < 0.5 \\
  160.0 f - 70.0 & \text{if } 0.0625 \le f < 0.25 \\
  480.0 f - 90.0 & \text{if } 0.0 < f < 0.0625 \\
  -120.0 & \text{if } f = 0.0
\end{cases}$$

Estimated SPL in the room is calculated as:
$$SPL_{est} = dB_F + C_{room}$$
Where $C_{room}$ is the calibration constant. $SPL_{est}$ is clamped to $[40.0, 100.0]$ dBA to avoid excessive EQ processing at extreme positions.

The target equal-loudness reference is $85.0$ dB SPL. The level difference is:
$$\Delta SPL = 85.0 - SPL_{est}$$

#### Mode-Specific Gain Calculations:
*   **PEQ Mode**:
    - Low Shelf Gain: $G_{low} = 0.22 \times \Delta SPL$ (clamped $[0.0, 9.0]$ dB).
    - High Shelf Gain: $G_{high} = 0.12 \times \Delta SPL$ (clamped $[0.0, 5.0]$ dB).
*   **GEQ Mode** (31-band TruEQ/GEQ):
    - For each frequency $f_i$, the sensitivity factor $K(f_i)$ is computed from ISO 226 contours:
        - $f_i < 100$ Hz: $K = 0.22$
        - $100 \text{ Hz} \le f_i < 1000 \text{ Hz}$: $K = 0.22 - \frac{f_i - 100}{900} \times 0.22$
        - $1000 \text{ Hz} \le f_i < 4000 \text{ Hz}$: $K = -0.08$
        - $f_i \ge 4000$ Hz: $K = 0.12$
    - Dynamic Gain: $G_i = K(f_i) \times \Delta SPL$ (clamped to $[0.0, 12.0]$ dB if $K > 0$, or $[-4.0, 0.0]$ dB if $K < 0$).
*   **DEQ Mode** (4-band Dynamic EQ):
    - Band 1 (Low Shelf, 80 Hz): $G_1 = 0.22 \times \Delta SPL$ (clamped $[0.0, 9.0]$ dB).
    - Band 2 (Bell, 500 Hz): $G_2 = -0.05 \times \Delta SPL$ (clamped $[-2.0, 0.0]$ dB).
    - Band 3 (Bell, 3200 Hz): $G_3 = -0.10 \times \Delta SPL$ (clamped $[-4.0, 0.0]$ dB).
    - Band 4 (High Shelf, 10 kHz): $G_4 = 0.12 \times \Delta SPL$ (clamped $[0.0, 5.0]$ dB).

All final gains are mapped to X32's raw OSC float range $[0.0..1.0]$ representing $[-15.0..15.0]$ dB:
$$\text{OSC Gain} = \frac{\text{Gain}_{dB} + 15.0}{30.0}$$

### Command Line Reference
```text
Options:
  -i, --ip <IP>              X32/M32 Mixer IP Address [default: 192.168.1.50]
      --config <PATH>        Path to calibration config file [default: .x32_loudness.json]
      --room-eq <PATH>       Path to room correction EQ JSON [optional]
      --mode <peq|geq|deq>   EQ mode to adjust [default: peq]
      --fx-slot <SLOT>       FX Slot index (1-8) for geq/deq [default: 5]
      --fader-path <PATH>    OSC fader path to monitor [default: /main/st/mix/fader]
  -h, --help                 Print help

Subcommands:
  calibrate                  Run calibration using the built-in pink noise generator
  run                        Run the dynamic EQ automation daemon
```

### Static Room Correction Offsets
You can apply a static room EQ profile alongside the dynamic equal-loudness curves using the `--room-eq` flag. The static offsets are summed directly with the dynamic gains before the final $[-15.0, 15.0]$ dB clamping:
$$\text{Final Gain}_{dB} = \text{Dynamic Gain}_{dB} + \text{Static Offset}_{dB}$$

#### Example `room_eq.json` Structure
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
*   `peq` must contain exactly 6 float values representing bands 1 through 6.
*   `geq` must contain exactly 31 float values matching the 1/3 ISO octave bands from 20 Hz to 20 kHz.
*   `deq` must contain exactly 4 float values representing bands 1 through 4.
