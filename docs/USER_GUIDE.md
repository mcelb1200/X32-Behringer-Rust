# X32 Tools User Guide

Welcome to the X32 Tools User Guide. This document categorizes the workspace utilities by workflow to help you find the right tool for your specific needs.

---

## 🎭 Live Performance & Mixing

These tools are designed for real-time use during a performance.

### `x32_autobeat`
Automatically detects the BPM of an incoming audio source (or via OSC meters) and synchronizes the delay time of X32 effects slots. Supports Energy and Spectral Flux detection algorithms.

### `x32_tapw` (Interactive)
A Terminal UI (TUI) tool for manually tapping tempo or using an automatic threshold. Displays levels and BPM in real-time.

### `x32_automix`
A Rust implementation of the "Dugan-style" automixer. It monitors input levels and automatically adjusts faders to prevent feedback and manage gain across multiple active microphones.

### `x32_fade`
Triggers smooth, timed fader transitions. Useful for automated scene changes or gradual fade-outs.

### `x32_jog4xlive`
Maps user-assignable encoders on the X32 to act as a "jog wheel" for the X-Live! card, allowing you to seek through recordings with hardware knobs.

---

## 🎧 Studio & DAW Integration

Tools for bridging the X32 with your recording environment.

### `x32_reaper`
A bidirectional bridge for the Reaper DAW. Synchronizes faders, mutes, and pans between the physical console and the DAW mixer.

### `x32_punch_control`
Records and plays back automation "punches" synchronized via MIDI Time Code (MTC). Uses the `.xpc` binary format for precise automation data.

### `x32_midi2osc`
A flexible translation engine that converts MIDI messages (Note On/Off, CC, Program Change) into X32 OSC commands using a template-based configuration.

---

## 💾 Utility, Backup & Migration

Tools for managing console state, libraries, and files.

### `x32_desk_save` / `x32_desk_restore`
Saves the entire state of the mixer (Scenes, Routing, Setup) to a human-readable text file and restores it. Optimized with `BufWriter` for high-speed I/O.

### `x32_get_lib` / `x32_set_lib`
Retrieves and uploads library presets (.chn, .efx, .rou) to the console's internal library slots.

### `x32_copy_fx` / `x32_geq2_cpy`
Specialized utilities for copying settings between FX slots or Graphic EQ bands.

---

## 📽️ Recording & Media

Utilities for working with X-Live! sessions and WAV files.

### `x32_wav_xlive`
Merges multiple mono WAV files into a single multi-channel session compatible with the X-Live! expansion card.

### `x32_xlive_wav`
Extracts individual mono WAV files from an X-Live! multi-channel recording.

### `x32_cpxlivemarkers`
Extracts markers from X-Live! session files and converts them to Reaper or Audition marker formats.

---

## 🛠️ Full Crate Index

| Crate | Logic | Description |
| :--- | :--- | :--- |
| `osc_lib` | Lib | Foundation for OSC packet serialization. |
| `x32_lib` | Lib | X32 Command DSL and parameter scaling. |
| `x32_core` | Lib | Stateful emulator loop and client tracking. |
| `x32_autobeat` | Bin | Automatic BPM effect synchronization. |
| `x32_automix` | Bin | Multi-channel automixing engine. |
| `x32_command` | Bin | Simple one-shot OSC command sender. |
| `x32_commander` | Bin | OSC/MIDI event-driven trigger system. |
| `x32_copy_fx` | Bin | FX slot parameter copier. |
| `x32_cpxlivemarkers`| Bin | X-Live! marker converter. |
| `x32_custom_layer` | Bin | Custom fader layer manager. |
| `x32_desk_restore` | Bin | Full console state restorer. |
| `x32_desk_save` | Bin | Full console state backup tool. |
| `x32_emulator` | Bin | Console simulator (Headless). |
| `x32_fade` | Bin | Timed fader transition utility. |
| `x32_geq2_cpy` | Bin | Graphic EQ setting copier. |
| `x32_get_lib` | Bin | Library preset downloader. |
| `x32_get_scene` | Bin | Scene data downloader. |
| `x32_get_scene_name`| Bin | Active scene name query. |
| `x32_jog4xlive` | Bin | X-Live! hardware jog wheel mapper. |
| `x32_midi2osc` | Bin | MIDI-to-OSC translation engine. |
| `x32_punch_control`| Bin | MIDI-sync'd automation tool. |
| `x32_reaper` | Bin | Reaper DAW bidirectional bridge. |
| `x32_replay` | Bin | OSC traffic recorder and player. |
| `x32_set_lib` | Bin | Library preset uploader. |
| `x32_set_preset` | Bin | Console preset loader. |
| `x32_set_scene` | Bin | Bulk OSC command sender (STDIN). |
| `x32_ssavergw` | Bin | Console screen saver / dimmer. |
| `x32_tap` | Bin | Delay tap tempo tool (CLI). |
| `x32_tapw` | Bin | Delay tap tempo tool (TUI). |
| `x32_tcp` | Bin | TCP-to-UDP OSC bridge. |
| `x32_udp` | Bin | UDP-to-UDP OSC bridge. |
| `x32_usb` | Bin | Console USB drive manager. |
| `x32_wav_xlive` | Bin | Mono-to-Multichannel WAV merger. |
| `x32_xlive_wav` | Bin | Multichannel-to-Mono WAV splitter. |
| `xair_command` | Bin | One-shot OSC sender for XAir. |
| `xair_get_scene` | Bin | XAir scene data downloader. |
| `xair_set_scene` | Bin | XAir bulk OSC command sender. |
