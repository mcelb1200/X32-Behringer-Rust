# `x32_ui` — Consolidated Web UI & REAPER Digital Twin Suite

A zero-external-HTTP-dependency local Web UI (`tokio` HTTP/1.1 + Server-Sent Events) that unifies all 48 `x32-cli` functions alongside live mixing, **Console-First Digital Split (Turn-On & Go)**, **Full Console OSC Event Recording**, and source-aware **REAPER Virtual Soundcheck (Digital Twin)** automation.

## Features

- **Live Mixer Console:**
  - Multi-bank faders, mutes, pans, scribble-strip names/colors, and live meters for **Channels 1–32**, **Aux In 1–8**, **Mix Buses 1–16**, **Matrices 1–6**, **DCAs 1–8**, and **Main LR**.
  - Stereo-link tracking across Channels (`/config/chlink/*`), Auxes (`/config/auxlink/*`), Buses (`/config/buslink/*`), and Matrices (`/config/mtxlink/*`).
  - Console snapshot backup/restore and 2-step smooth panic mute.
- **REAPER Virtual Soundcheck (Digital Twin):**
  - Captures live console state over OSC **or imports an offline `.scn` scene file** (bounded to 4 MiB).
  - **Source-Aware Patching & Shared-Source Deduplication:** Reads `/ch/XX/config/source` (`1..=32`) to populate `card_in` / `card_out`, prevents double-driving a single `HWOUT` when two channels share one physical input (`deduplicate_shared_sources`), and defaults `include_empty_channels` to `false` when custom names are present.
  - **Digital Twin `.rpp` & Live OSC Sync:** Writes converted console faders (`VOLPAN`), pans, and mutes (`MUTESOLO`) into generated `.rpp` tracks; optionally generates Aux In 1–8, Bus 1–16, Matrix 1–6, and DCA 1–8 tracks; adapts existing `.rpp` templates non-destructively (preserving items/FX/markers); writes matching `.X32Reaper.ini`; and pushes `/track/{n}/name`, `/volume`, `/mute`, `/pan` + transport to REAPER over OSC.
  - **X32 v4 `UOUT1-48` & `PLAY/AUX` Preservation:** Recognizes Card routing indices `26..=31` (`UOUT1-8`..`UOUT41-48`), optionally keeps `UOUT` blocks intact (`preserve_card_uout`) with clear warnings, and preserves existing `PLAY/AUX` `CARD1-2/4/6` routing (`10..=12`).
- **Console-First Digital Split & Full Event Recorder (Turn-On & Go):**
  - Programs X32 **User Assign Bank** (`A`/`B`/`C`, hardware buttons 5–12) so an operator can boot the desk and PC and control **Live ↔ DAW** switching, synced audio + console OSC event recording (`.x32events.jsonl`), playback, markers, snapshots, and `x32-reaper` directly from the console.
  - **Active AES50 & Loop Protection:** Mirrors stage inputs (`IN`) to `CARD` Out `1–32` and optionally mirrors onto AES50-A or AES50-B, automatically skipping AES50 mirroring (with a warning) whenever stagebox inputs (`4..=9` / `10..=15`) or active outputs `>= 16` on blocks `1–32` (`OUT*`, `P16*`, `CARD*`, `UOUT*`) are already in use.
  - **Auto-Session Generation on Connect:** When `auto_generate_rpp_on_connect` is enabled, automatically writes `<recordings_dir>/AutoSession.rpp` + `.X32Reaper.ini` before auto-launching REAPER.
- **Routing Matrix, All 48 Tools, & OSC Console:**
  - Visual routing editor for `routswitch`, `IN`, `PLAY`, `CARD`, `OUT`, `AES50-A`, and `AES50-B`.
  - Searchable, categorized launcher for every `x32-cli` subcommand with live stdout/stderr streaming.
  - Raw typed OSC sender and live incoming/outgoing OSC stream.

## CLI Usage

```bash
# Start on default http://127.0.0.1:8080 and open browser
cargo run -p x32-cli -- x32-ui --open

# Auto-discover mixer on LAN and bind all interfaces for tablet control
cargo run -p x32-cli -- x32-ui --bind-ip 0.0.0.0 --port 8080 --ip auto

# Unattended Turn-On & Go supervisor using saved profile
cargo run -p x32-cli -- x32-ui --profile x32_studio_profile.json --auto-boot
```

### CLI Options

| Flag | Default | Description |
| :--- | :--- | :--- |
| `--bind-ip` | `127.0.0.1` | HTTP bind IP for the Web UI server |
| `-p, --port` | `8080` | TCP port for the Web UI server |
| `-i, --ip` | *(none)* | Mixer IP address (or `auto` for LAN discovery) |
| `--transport` | `auto` | Transport mode (`auto`, `osc`, `aes50`, `usb`) |
| `--aes50-ip` | `""` | AES50 bridge IP address |
| `--usb-port` | `""` | USB MIDI port name |
| `--open` | `false` | Open Web UI in default browser on startup |
| `--profile` | `x32_studio_profile.json` | Path to persisted Studio Mode profile JSON |
| `--auto-boot` | `false` | Enable unattended Turn-On & Go connect & init supervisor |

## X32 User Assign Hardware Deck (Buttons 5–12)

| Button | Label | Action |
| :--- | :--- | :--- |
| **5** | `5: LIVE/DAW` | Pop-guarded `routswitch` switch between Live (`REC`) and DAW (`PLAY`) |
| **6** | `6: REC` | Toggle synchronized REAPER audio + console OSC event recording |
| **7** | `7: PLAY` | Toggle REAPER DAW playback (and synced event replay if enabled) |
| **8** | `8: STOP` | Stop active recording and playback |
| **9** | `9: MARKER` | Drop synchronized marker in REAPER and `.x32events.jsonl` log |
| **10** | `10: EVT SYNC` | Toggle synchronized console event replay on DAW playback |
| **11** | `11: SNAPSHOT` | Save quick console snapshot backup |
| **12** | `12: DAW BRIDGE` | Start/stop bidirectional `x32-reaper` bridge |

## HTTP / SSE API Endpoints

- **Realtime & Connection:** `GET /api/health`, `GET /api/discover`, `GET /api/state`, `GET /api/events` (SSE), `POST /api/connect`, `POST /api/disconnect`, `POST /api/refresh`
- **Mixer & Routing:** `POST /api/mixer/fader`, `POST /api/mixer/mute`, `POST /api/mixer/pan`, `POST /api/mixer/channel-config`, `POST /api/mixer/routing`, `POST /api/mixer/panic-mute`, `POST /api/mixer/snapshot-save`, `POST /api/mixer/snapshot-restore`, `POST /api/osc`
- **Virtual Soundcheck / Scene Import:** `POST /api/vsc/capture`, `POST /api/vsc/import-scene`, `POST /api/vsc/apply-routing`, `POST /api/vsc/mode`, `POST /api/vsc/restore-backup`, `POST /api/vsc/generate-rpp`, `POST /api/vsc/reaper-osc`
- **Studio Mode & Event Recorder:** `GET /api/studio/status`, `POST /api/studio/profile`, `POST /api/studio/apply-split`, `POST /api/studio/mode`, `POST /api/studio/record/start`, `POST /api/studio/record/stop`, `POST /api/studio/play`, `POST /api/studio/stop`, `POST /api/studio/marker`, `POST /api/studio/program-buttons`, `POST /api/studio/button`, `GET /api/studio/recordings`
- **Tools Launcher:** `GET /api/tools`, `GET /api/tools/jobs`, `POST /api/tools/run`, `POST /api/tools/stop`

## Crate Modules

- `mixer.rs` — Multi-transport mixer client wrapper, full snapshot state (including Matrices 1–6 & stereo links), and snapshot backup/restore
- `vsc.rs` — Source-aware Digital Twin `.rpp` generator/adapter, `UOUT1-48` routing planner, `.X32Reaper.ini` writer, and REAPER OSC sender
- `scene_import.rs` — Bounded (4 MiB) offline `.scn` parser into `MixerSnapshotState`
- `studio_mode.rs` — Turn-On & Go supervisor, loop/active-output-safe digital split, pop-guard transitions, and hardware button/LED deck
- `event_recorder.rs` — Keyframe + timestamped delta `.x32events.jsonl` recorder & replayer
- `server.rs`, `runner.rs`, `tools_catalog.rs` — Embedded HTTP/1.1 + SSE server, process manager, and 48-tool catalog

## Testing

```bash
cargo test -p x32_ui
```
