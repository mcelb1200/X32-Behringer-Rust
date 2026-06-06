# x32_autobeat User Documentation

An advanced live-performance tool for automatic beat detection and time-based effect synchronization on Behringer X32 and X-Air mixers.

---

## Layer 1: Quick Start

Get `x32_autobeat` running in 60 seconds.

### 1. Connection
Ensure your computer is connected to the same network as the mixer. Find the mixer's IP address.

### 2. Identify Audio Input
List available system audio inputs:
```bash
./x32_autobeat list-devices
```

### 3. Launch Tool
Run the application specifying your mixer's IP, target input channel, and audio device index/name:
```bash
./x32_autobeat --ip 192.168.1.50 --channel 1 --device "Built-in Audio"
```
*(If `--device` is omitted, the tool automatically falls back to OSC network-level monitoring.)*

### 4. Keyboard Controls
*   `q` : Quit program.
*   `p` : **Panic** (Instantly kills all delay feedback/reverb tails).
*   `r` : **Reset** (Clears panic state to resume normal operation).
*   `s` : Cycle active FX slot.
*   `a` : Toggle between **Energy** (drums/electronic) and **Spectral** (complex rock/indie) detection.
*   `Up` / `Down` : Cycle tempo subdivisions (1/4, 1/8, etc.).
*   `Left` / `Right` : Cycle style presets (Tight, Natural, etc.).

---

## Layer 2: Volunteer User Guide

For sound board operators and church/live-event volunteers.

### Dynamic Syncing
Instead of manually tapping tempos during a live set, `x32_autobeat` listens to the music (e.g. from the drum channel or master output) and constantly updates the mixer's delays/reverbs in real time. 

### Visual Screen Layout
*   **BPM**: Displays the currently detected music tempo.
*   **Level Bar**: Shows incoming audio volume. If flat, check connection or selected input channel.
*   **Algorithm Status**:
    *   *Audio OK*: Tool is analyzing crisp, high-fidelity system audio.
    *   *Fallback (OSC)*: No system audio detected. Tool is monitoring network volume packets instead.
*   **Selected Slot**: Shows the mixer's current effect slot you are modifying.

### Rhythmic Subdivisions
Change how the delay repeats relative to the beat:
*   `1/4` : Quarter note (standard pop/rock delay).
*   `1/8` : Eighth note (fast).
*   `1/8d` : Dotted eighth (spacious, modern worship/U2 style).
*   `1/8t` : Triplet eighth (swing).

### Style Presets
Presets adjust secondary parameters (decay length, feedback level, high-frequency brightness) to fit the style of the song:
*   **Tight** (Pop/Funk): Very short reverb tails, low delay feedback, crisp bright tone. Keeps the mix clean.
*   **Natural** (Acoustic/Jazz): Soft, organic wash. Low volume mix.
*   **Standard** (General Rock): Balanced decay, medium feedback, warm filters. Safe default.
*   **Big** (EDM/Ballads): Lush long reverbs, high feedback delay with L/R offset width.
*   **Huge** (Ambient/Shoegaze): Massively long space washes, extreme delay repeats. Great for slow atmospheric sections.

---

## Layer 3: Power User Guide

For systems integrators, engineers, and power users.

### Dynamic Failover Logic
To ensure uninterrupted live synchronization, the tool runs a hybrid detection engine:
```
System Audio Input Active?
       ├── YES ──> Process local audio stream via Energy/Spectral Flux.
       └── NO (200ms silence) ──> Query /meters/1 over OSC. Analyze volume envelope.
```

### Advanced Command Line Options
```text
  --ip <IP>                      X32/X-Air Mixer IP Address [default: 192.168.1.50]
  --device <DEVICE>              Audio Device Name or Substring [optional]
  --channel <CHANNEL>            Console Input Channel (1-32) [default: 1]
  --slot <SLOT>                  Default FX Slot (1-8) [default: 1]
  --panic-btn <PANIC_PATH>       OSC path to trigger Panic remotely [default: "A/btn/5"]
  --preset-enc <ENC_PATH>        OSC path to assignable encoder for preset style [default: "A/enc/5"]
  --btn-led-path <PATH>          OSC path for feedback LED to flash with the beat [default: "/config/usercontrols/assign/A/btn/5/led"]
  --info-scribble-path <PATH>    OSC path for target scribble strip to show BPM/style [default: "/ch/32/config/name"]
  --target-channels <CHANNELS>   Channels for automatic compressor envelope sync (e.g., "1,2" or "1-4")
  --compressor-subdivision <SUB> Compressor release timing multiplier [default: 1.0]
```

### Hardware Remote Mapping
You can map X32 assignable controls to interact with `x32_autobeat` over the network:
1.  **Panic Button**: Map a button on your console to send the path configured in `--panic-btn`. Pressing it triggers the panic routine.
2.  **Preset Style Encoder**: Map an assignable rotary encoder to `--preset-enc`. Turning it cycles FX preset styles (Tight, Natural, etc.).
3.  **Beat Feedback LED**: The assignable button LED at `--btn-led-path` flashes dynamically on every detected beat.
4.  **BPM / Style Display**: The target scribble strip at `--info-scribble-path` (e.g. Channel 32) displays the current BPM and active style (e.g. `120BPM TIGHT`).
5.  **Input Channel Scribble Strip**: The scribble strip of the configured input channel dynamically updates to show the selected FX Slot and subdivision (e.g. `FX1:1/8d`).

### Compressor Envelope Synchronization
Use `--target-channels` to automatically adjust the `Hold` and `Release` parameters of target channel compressors relative to detected BPM.
*   Calculates: `QuarterNotePeriod = 60000 / BPM`
*   `Hold` is scaled logarithmically between $[0.02, 2000]$ms.
*   `Release` is scaled logarithmically between $[20, 2000]$ms.
*   Syncs compression breathing cycles to the rhythm of the music automatically.
