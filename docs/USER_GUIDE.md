# X32 Tools User Guide

Welcome to the X32 Tools User Guide. This document categorizes the workspace utilities by workflow to help you find the right tool for your specific needs.

---

## 🛡️ Volunteer & Safety Apps

High-level applications in `apps/` designed for non-expert operators, live safety guardrails, and automated setup:

### `x32_ui` / `x32-cli x32-ui` (Consolidated Web UI & Digital Twin Suite)
Local browser-based control suite (`http://127.0.0.1:8080`, zero external HTTP crates) unifying every X32/M32/Wing/X-Air tool alongside live mixing, Console-First Studio Mode, full OSC event recording, and automated **REAPER Virtual Soundcheck (Digital Twin)**. See [apps/x32_ui/README.md](../apps/x32_ui/README.md).

#### Launching & CLI Flags
```bash
# Start local dashboard and open browser
x32-cli x32-ui --open

# Auto-discover mixer on LAN and expose dashboard to tablets on port 8080
x32-cli x32-ui --bind-ip 0.0.0.0 --port 8080 --ip auto

# Unattended Turn-On & Go boot supervisor using saved profile
x32-cli x32-ui --profile x32_studio_profile.json --auto-boot
```
- `--bind-ip` (`127.0.0.1`), `--port` (`8080`), `--ip <IP|auto>`, `--transport <auto|osc|aes50|usb>`, `--aes50-ip`, `--usb-port`, `--open`, `--profile` (`x32_studio_profile.json`), `--auto-boot`.

#### Live Console Control
- Multi-bank faders, mutes, pans, scribble-strip names/colors, and live meters across **Channels 1–32**, **Aux In 1–8**, **Mix Buses 1–16**, **Matrices 1–6**, **DCAs 1–8**, and **Main LR**.
- Tracks stereo link state across channels (`/config/chlink/*`), auxes (`/config/auxlink/*`), buses (`/config/buslink/*`), and matrices (`/config/mtxlink/*`).
- Visual Routing Matrix (`routswitch`, `IN`, `PLAY`, `CARD`, `OUT`, `AES50-A`, `AES50-B`), raw typed OSC console with live stream, searchable launcher for all 48 `x32-cli` tools, snapshot backup/restore, and 2-step smooth panic mute.

#### Console-First Turn-On & Go & Digital Split Workflow
1. Configure and save your studio profile once in the **Studio Mode** tab (`x32_studio_profile.json`: mixer IP/transport, REAPER paths & OSC ports, `recordings_dir`, AES50 split mirror option, pop-guard mute duration, User Assign bank, `preserve_card_uout`, `auto_generate_rpp_on_connect`, `auto_connect_on_boot`).
2. Start `x32-cli x32-ui --auto-boot` at computer login (or enable `auto_connect_on_boot`). When the console comes online, the supervisor connects automatically, applies the safe digital split (if enabled), generates `<recordings_dir>/AutoSession.rpp` + `.X32Reaper.ini` (when `auto_generate_rpp_on_connect` is enabled), optionally launches REAPER and `x32-reaper`, and programs the X32 **User Assign Bank** (`A`, `B`, or `C`, buttons **5–12**) with synced LEDs:
   - **Button 5 (`LIVE/DAW`):** Pop-guarded `routswitch` switch between **LIVE (REC)** and **DAW (PLAY)** (temporarily engages Main LR/Matrix/Monitor pop-guard mute during clock/routing transition).
   - **Button 6 (`REC`):** Starts/stops synchronized REAPER audio recording + full console OSC event recording.
   - **Button 7 (`PLAY`):** Starts/stops REAPER DAW playback (with optional synced console event replay).
   - **Button 8 (`STOP`):** Stops active recording/playback.
   - **Button 9 (`MARKER`):** Drops a synchronized marker in both REAPER and the event recording log.
   - **Button 10 (`EVT SYNC`):** Toggles synchronized console event replay during DAW playback.
   - **Button 11 (`SNAPSHOT`):** Saves a quick timestamped console snapshot backup.
   - **Button 12 (`DAW BRIDGE`):** Starts/stops the bidirectional `x32-reaper` bridge process.
3. **Digital Split & AES50 / `UOUT1-48` Protection:**
   - Mirrors physical stage inputs (`IN`) to `CARD` Out `1–32` (automatically avoiding `CARD`-to-`CARD` feedback loops) and pre-stages `PLAY` return on `CARD 1–32` while preserving existing `PLAY/AUX` `CARD1-2/4/6` routing (`10..=12`).
   - Supports X32 v4 **`UOUT1-48`** (`26..=31`) on `CARD` routing blocks: when `preserve_card_uout` is enabled, `UOUT` feeds remain intact with an advisory warning.
   - Protects active AES50 ports: AES50-A or AES50-B split mirroring is skipped with a warning whenever that port already carries stagebox inputs (`4..=9` for AES50-A, `10..=15` for AES50-B) or active outputs (`>= 16`, such as `OUT*`, `P16*`, `CARD*`, or `UOUT*`) on blocks `1–32`.

#### Full Console Event Recorder
- Captures an initial full-console keyframe snapshot followed by timestamped OSC parameter deltas and markers into `.x32events.jsonl` files inside `recordings_dir`.
- Replays timed console changes back to the X32 alongside REAPER multitrack audio playback.

#### REAPER Virtual Soundcheck & Digital Twin Setup
- **Live Capture or Offline `.scn` Import:** Capture live state directly from the console (`Capture`), or import any saved X32 `.scn` scene file offline in the Web UI (Sentinel-bounded to 4 MiB) to inspect routing and build a REAPER session before showtime.
- **Source-Aware Patching & Shared-Source Deduplication:** Reads each channel's physical `/ch/XX/config/source` (`1..=32`) to assign `card_in`/`card_out`, deduplicates channels sharing the same physical source (`deduplicate_shared_sources`) so multiple mixer channels tapping one input do not double-drive a single `HWOUT`, and defaults `include_empty_channels` to `false` when custom channel names are present.
- **Digital Twin Synchronization:** When `digital_twin_mode` is active, `.rpp` track generation writes converted console fader (`VOLPAN`), pan, and mute (`MUTESOLO`) values; optionally includes Aux In 1–8, Mix Bus 1–16, Matrix 1–6, and DCA 1–8 tracks; and `Push to REAPER OSC` transmits live `/track/{n}/name`, `/volume`, `/mute`, and `/pan`.
- **Template Adaptation & Routing Backup:** Non-destructively adapts existing `.rpp` templates (such as [docs/X32Template.rpp](X32Template.rpp) and [docs/Template 48 tracks.rpp](Template%2048%20tracks.rpp)) while preserving recorded items, FX chains, and markers, generates a matching `.X32Reaper.ini`, and saves/restores pre-VSC console snapshots.

### `x32_volunteer` (Interactive)
Simplified Terminal UI dashboard for volunteer sound operators across X32, Wing, and X-Air (`XR18`/`XR16`/`XR12`). Supports `--ip auto` LAN discovery, `x32_config.json` ([sample config](examples/x32_config.volunteer.json)), per-channel mute/fader nudging clamped to `--max-fader-limit-db`, Main LR & connection health monitoring, `[A]` spectrum-aware mixbus Dugan automixing with dynamic vocal-pocket EQ, broadcast `stream_monitor`, `[C]` pre-service check & backup, `[U]` undo rollback, and a 2-step smooth-fade `[P]anic` button. See [apps/x32_volunteer/README.md](../apps/x32_volunteer/README.md) and the [RTA Automix Spec](specs/rta_spectrum_automix.md).

### `x32_safe_mute`
"Panic Button" utility that silences the system using a rapid exponential fade instead of a hard mute to prevent speaker pops and thumps.

### `x32_scene_checker`
Pre-flight scene diffing and risk-classification tool (`--safe-load` skips critical/high-risk changes, `--locked-paths` protects routing/main buses, and `--backup` saves the pre-load state).

### `x32_speech_mode`
Model-aware one-shot macro that configures spoken-word channels (HPF, compression, EQ) safely across X32, Wing, and X-Air consoles.

### `x32_auto_gain` / `x32_auto_ringout` / `x32_feedback_detect` / `x32_vocal_ducking`
Automated gain staging, monitor ringout notch filtering, live feedback detection, and music-under-speech ducking.

### `x32_dca_spill` / `x32_sync` / `x32_system_tune`
DCA spill controller, bidirectional multi-console state sync, and guided system tuning TUI (`x32_system_tune --list-mics` / `--mic-profile m5` supports 16 common venue microphones including SM58, e835, AKG C747, Rode M5, Behringer C-2/C5, and uncalibrated ECM8000 with cut-only RTA calibration and `[b]`/`[t]`/`[u]` A/B undo). See [Guided Room Tuning Spec](specs/guided_room_tuning_any_mic.md).

---

## 🎭 Live Performance & Mixing

These tools are designed for real-time use during a performance.

### `x32-cli x32-autobeat`
Automatically detects the BPM of an incoming audio source (or via OSC meters) and synchronizes the delay time of X32 effects slots. Supports Energy and Spectral Flux detection algorithms.

### `x32-cli x32-tapw` (Interactive)
A Terminal UI (TUI) tool for manually tapping tempo or using an automatic threshold. Displays levels and BPM in real-time.

### `x32-cli x32-automix`
A Rust implementation of the "Dugan-style" automixer. It monitors input levels and automatically adjusts faders to prevent feedback and manage gain across multiple active microphones.

### `x32-cli x32-crossfade`
Smoothly interpolates faders, EQs, and dynamics parameters between two scenes (`.scn`). Continuous values fade over time, while discrete values snap at a specified progress threshold.

### `x32-cli x32-fade`
Triggers smooth, timed fader transitions. Useful for automated scene changes or gradual fade-outs.

### `x32-cli x32-jog4xlive`
Maps user-assignable encoders on the X32 to act as a "jog wheel" for the X-Live! card, allowing you to seek through recordings with hardware knobs.

---

## 🎧 Studio & DAW Integration

Tools for bridging the X32 with your recording environment.

### `x32-cli x32-reaper`
A bidirectional bridge for the Reaper DAW. Synchronizes faders, mutes, and pans between the physical console and the DAW mixer.

### `x32-cli x32-punch-control`
Records and plays back automation "punches" synchronized via MIDI Time Code (MTC). Uses the `.xpc` binary format for precise automation data.

### `x32-cli x32-midi2osc`
A flexible translation engine that converts MIDI messages (Note On/Off, CC, Program Change) into X32 OSC commands using a template-based configuration.

---

## 💾 Utility, Backup & Migration

Tools for managing console state, libraries, and files.

### `x32-cli x32-desk-save` / `x32-cli x32-desk-restore`
Saves the entire state of the mixer (Scenes, Routing, Setup) to a human-readable text file and restores it. Optimized with `BufWriter` for high-speed I/O.

### `x32-cli x32-get-lib` / `x32-cli x32-set-lib`
Retrieves and uploads library presets (.chn, .efx, .rou) to the console's internal library slots.

### `x32-cli x32-copy-fx` / `x32-cli x32-geq2-cpy`
Specialized utilities for copying settings between FX slots or Graphic EQ bands.

---

## 📽️ Recording & Media

Utilities for working with X-Live! sessions and WAV files.

### `x32-cli x32-wav-xlive`
Merges multiple mono WAV files into a single multi-channel session compatible with the X-Live! expansion card.

### `x32-cli x32-xlive-wav`
Extracts individual mono WAV files from an X-Live! multi-channel recording.

### `x32-cli x32-cpxlivemarkers`
Extracts markers from X-Live! session files and converts them to Reaper or Audition marker formats.

---

## 🛠️ Full CLI Subcommand Index

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
| `x32_ui` | App | Consolidated Web UI, Matrices, Digital Split & REAPER Digital Twin suite. |
| `x32_tcp` | Bin | TCP-to-UDP OSC bridge. |
| `x32_udp` | Bin | UDP-to-UDP OSC bridge. |
| `x32_usb` | Bin | Console USB drive manager. |
| `x32_wav_xlive` | Bin | Mono-to-Multichannel WAV merger. |
| `x32_xlive_wav` | Bin | Multichannel-to-Mono WAV splitter. |
| `xair_command` | Bin | One-shot OSC sender for XAir. |
| `xair_get_scene` | Bin | XAir scene data downloader. |
| `xair_set_scene` | Bin | XAir bulk OSC command sender. |
