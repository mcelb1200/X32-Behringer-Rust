# Guided Room Tuning Using "What We Have" Microphones (Implemented)

## 1. Overview
Implemented in [`libs/x32_lib/src/room_tune.rs`](../../libs/x32_lib/src/room_tune.rs) and [`apps/x32_system_tune`](../../apps/x32_system_tune/src/lib.rs). Volunteers can tune a room safely using whichever microphone is already on stage, the console's built-in Pink Noise generator (capped at `MAX_EBU_R128_LEVEL` = `-18 dBFS`), X32 `/meters/4` 100-band RTA, and `ConsoleSnapshot` undo.

## 2. Built-In 16-Model Microphone Compensation Library

| ID | Model | Category | Valid Band | Max Cut | `+48V` | Notes |
| :--- | :--- | :--- | :---: | :---: | :---: | :--- |
| `sm58` | Shure SM58 | Cardioid Dynamic | `90 Hz – 7 kHz` | `-5.0 dB` | No | Cancels `+4 dB @ 4.5 kHz` presence hump |
| `beta58` | Shure Beta 58A | Supercardioid Dynamic | `90 Hz – 7 kHz` | `-5.0 dB` | No | Cancels `+5 dB @ 4 kHz` & `+4 dB @ 9 kHz` |
| `sm57` | Shure SM57 | Cardioid Dynamic | `110 Hz – 7 kHz` | `-5.0 dB` | No | Cancels `+5.5 dB @ 5.5 kHz` |
| `e835` | Sennheiser e835 | Cardioid Dynamic | `85 Hz – 8 kHz` | `-5.0 dB` | No | Cancels gentle `+3.5 dB @ 4.5 kHz` lift |
| `e935` | Sennheiser e935 / e945 | Cardioid/Supercardioid | `85 Hz – 8 kHz` | `-5.0 dB` | No | Cancels `+4 dB @ 5 kHz` |
| `d5` | AKG D5 | Supercardioid Dynamic | `90 Hz – 7 kHz` | `-5.0 dB` | No | Cancels `+4 dB @ 4.5 kHz` |
| `om2` | Audix OM2 / OM5 | Hypercardioid Dynamic | `120 Hz – 7 kHz` | `-4.5 dB` | No | Steep `< 120 Hz` roll-off ignored |
| `c747` | AKG C747 | Hypercardioid Condenser | `80 Hz – 10 kHz` | `-6.0 dB` | **Yes** | Flat mids, cancels `+4.5 dB @ 9 kHz` |
| `mx418` | Shure MX412 / MX418 | Gooseneck Condenser | `120 Hz – 8 kHz` | `-5.0 dB` | **Yes** | Built-in `< 120 Hz` roll-off ignored |
| `m5` | **Rode M5** | Small-Diaphragm Cardioid Condenser | `60 Hz – 12 kHz` | `-6.0 dB` | **Yes** | Very flat, mild `+2 dB @ 9 kHz` compensation |
| `nt5` | Rode NT5 / Shure SM81 | Small-Diaphragm Cardioid Condenser | `60 Hz – 12 kHz` | `-6.0 dB` | **Yes** | Flat, mild `+2 dB @ 9 kHz` compensation |
| `c5` | **Behringer C-2 / C5** | Small-Diaphragm Cardioid Condenser | `75 Hz – 10 kHz` | `-5.5 dB` | **Yes** | Aliases: `c-2`, `c2`; cancels `+3.5 dB @ 8.5 kHz` |
| `at2020` | Audio-Technica AT2020 | Large-Diaphragm Condenser | `60 Hz – 10 kHz` | `-5.5 dB` | **Yes** | Cancels `+2.5 dB @ 8 kHz` brightness |
| `ecm8000` | Behringer ECM8000 (uncalibrated) | Omni Electret | `40 Hz – 10 kHz` | `-6.0 dB` | **Yes** | Flat `40–4kHz`, compensates `+2.5 dB @ 8 kHz` |
| `laptop` | Laptop / Phone Mic | Consumer MEMS | `160 Hz – 5.5 kHz` | `-4.0 dB` | No | Conservative boom/honk cuts only |
| `flat` | Calibrated Measurement Mic | Calibrated Omni | `40 Hz – 12 kHz` | `-6.0 dB` | **Yes** | `0 dB` flat offset |

## 3. Guided Workflow in `x32_system_tune`
1. `cargo run -p x32_system_tune -- --list-mics` prints all 16 models, valid ranges, `+48V` requirements, and placement advice.
2. Launching with `--ip auto` discovers the console and opens the interactive mic picker (`[↑/↓]` select mic, `[` / `]` adjust MaxCut guardrail, `[Enter]` confirm).
3. Safety snapshot (`ConsoleSnapshot`) is captured before pink noise or EQ changes; `--mic-channel` is automatically unrouted from Main LR.
4. During Room Tuning, `[Space]` captures 3–5 seat RTA positions (`average_rta_captures`), `[y]` applies **cut-only** PEQ notches (`suggest_cut_only_peq`), `[b]`/`[t]` toggles Bypass vs Tuned A/B auditioning, and `[u]` restores the pre-tune snapshot.
