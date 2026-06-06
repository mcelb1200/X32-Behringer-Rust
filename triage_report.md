# 🔍 Semantic Diff Triage & SemVer Report
Generated on: `2026-05-21 23:28:26`  
Repository: `X32-Behringer-Rust`  
Comparing Target: `origin/main` ➔ Branch: `refactor/mixer-client-abstraction`

## 📊 Branch Status
| Attribute | Status |
| :--- | :--- |
| **Ahead Commits** | 14 |
| **Behind Commits** | 0 |
| **Divergence State** | AHEAD |

## 🚨 Risk & Regression Analysis
> [!CRITICAL]
> **🔴 API Breaking Change (Static Detection)**  
> Detected **7 deleted public API items** (functions or structs) which will cause downstream compilation breaks.

> [!WARNING]
> **🟡 File Deletion Alert**  
> The following files were completely deleted (not renamed): `x32_automix/Cargo.toml`, `x32_command/Cargo.toml`, `x32_command/src/main.rs`, `x32_commander/Cargo.toml`, `x32_copy_fx/Cargo.toml`, `x32_custom_layer/Cargo.toml`, `x32_emulator/Cargo.toml`, `x32_fade/Cargo.toml`, `x32_geq2_cpy/Cargo.toml`, `x32_get_lib/Cargo.toml`, `x32_get_scene/Cargo.toml`, `x32_jog4xlive/Cargo.toml`, `x32_punch_control/Cargo.toml`, `x32_reaper/Cargo.toml`, `x32_replay/Cargo.toml`, `x32_set_lib/Cargo.toml`, `x32_set_preset/Cargo.toml`, `x32_set_scene/Cargo.toml`, `x32_ssavergw/Cargo.toml`, `x32_tcp/Cargo.toml`, `x32_udp/Cargo.toml`, `x32_usb/Cargo.toml`, `xair_get_scene/src/main.rs`, `xair_set_scene/src/main.rs`

> [!CRITICAL]
> **🔴 SemVer Compatibility Failure**  
> Workspace fails semver check against baseline! Downstream integration breaking.

## 🛠️ API & Public Symbol Changes
### 🔴 Deleted Public APIs (Breaking changes)
- `fn update() in `apps/x32_autobeat/src/compressor.rs``
- `fn new() in `apps/x32_autobeat/src/network.rs``
- `fn send() in `apps/x32_autobeat/src/network.rs``
- `fn set_scribble_text() in `apps/x32_autobeat/src/network.rs``
- `fn set_scribble_target() in `apps/x32_autobeat/src/network.rs``
- `fn set_effect_param() in `apps/x32_autobeat/src/network.rs``
- `fn send_osc_float() in `apps/x32_autobeat/src/network.rs``

### 🟢 Added Public APIs
- `fn update() in `apps/x32_autobeat/src/compressor.rs``
- `fn new() in `apps/x32_autobeat/src/network.rs``
- `fn send() in `apps/x32_autobeat/src/network.rs``
- `fn set_scribble_text() in `apps/x32_autobeat/src/network.rs``
- `fn set_scribble_target() in `apps/x32_autobeat/src/network.rs``
- `fn set_effect_param() in `apps/x32_autobeat/src/network.rs``
- `fn send_osc_float() in `apps/x32_autobeat/src/network.rs``
- `struct MixerClient in `libs/x32_lib/src/client.rs``
- `fn connect() in `libs/x32_lib/src/client.rs``
- `fn start_heartbeat() in `libs/x32_lib/src/client.rs``
- `fn stop_heartbeat() in `libs/x32_lib/src/client.rs``
- `fn send_message() in `libs/x32_lib/src/client.rs``
- `fn query_value() in `libs/x32_lib/src/client.rs``
- `fn subscribe() in `libs/x32_lib/src/client.rs``
- `fn set_parameter_async() in `libs/x32_lib/src/lib.rs``
- `fn get_parameter_async() in `libs/x32_lib/src/lib.rs``
- `fn query_value_async() in `libs/x32_lib/src/lib.rs``

## 💡 Remediation & Git Process Guidelines
### 📍 Deleted public API signatures (Compile Breakage)
- **Preferred Process**: `API Depreciation pattern (add `#[deprecated]` attribute rather than hard deletion of functions)`
- **Suggested Remediation**: Restore the signature to maintain backwards compatibility, use `#[deprecated]` to warn downstream clients, or increment the package's major version according to SemVer rules.

### 📍 File deletion verification
- **Preferred Process**: `Explicit Git tracking verification (`git diff --summary` to confirm moves/renames)`
- **Suggested Remediation**: Confirm deletions are intentional workspace cleanups. If files were relocated, ensure the workspace `Cargo.toml` targets and dependency references are updated.

## 📦 Cargo SemVer & API Check
### SemVer Check Output
```text
error: no such command: `semver-checks`

help: view all installed commands with `cargo --list`
help: find a package to install `semver-checks` with `cargo search cargo-semver-checks`
```

### Public API Diff
```text
error: no such command: `public-api`

help: view all installed commands with `cargo --list`
help: find a package to install `public-api` with `cargo search cargo-public-api`
```

## 📄 File-by-File Semantic Triage
| File | Added APIs | Deleted APIs | Logical Lines | Trivial Edits | Risk Level |
| :--- | :--- | :--- | :--- | :--- | :--- |
| .jules/backlog/TASK-001.md | 0 | 0 | +20/-0 | +3/-0 | 🟢 LOW |
| .jules/backlog/TASK-002.md | 0 | 0 | +17/-0 | +3/-0 | 🟢 LOW |
| .jules/backlog/TASK-003.md | 0 | 0 | +17/-0 | +3/-0 | 🟢 LOW |
| .jules/backlog/TASK-004.md | 0 | 0 | +16/-0 | +3/-0 | 🟢 LOW |
| .jules/backlog/TASK-005.md | 0 | 0 | +15/-0 | +3/-0 | 🟢 LOW |
| .jules/jclaw.db | 0 | 0 | +0/-0 | +0/-0 | 🟢 LOW |
| .jules/state.json | 0 | 0 | +5/-0 | +0/-0 | 🟢 LOW |
| Cargo.lock | 0 | 0 | +23/-0 | +1/-0 | 🟢 LOW |
| Cargo.toml | 0 | 0 | +10/-37 | +6/-0 | 🟡 MEDIUM |
| DELEGATION_JULES.md | 0 | 0 | +23/-0 | +7/-0 | 🟢 LOW |
| `x32_autobeat/Cargo.toml` ➔ `apps/x32_autobeat/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/README.md` ➔ `apps/x32_autobeat/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/audio.rs` ➔ `apps/x32_autobeat/src/audio.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/compressor.rs` ➔ `apps/x32_autobeat/src/compressor.rs` [RENAME] | 1 | 1 | +2/-2 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/detection/energy.rs` ➔ `apps/x32_autobeat/src/detection/energy.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/detection/mod.rs` ➔ `apps/x32_autobeat/src/detection/mod.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/detection/osc_level.rs` ➔ `apps/x32_autobeat/src/detection/osc_level.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/detection/research.md` ➔ `apps/x32_autobeat/src/detection/research.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/detection/spectral.rs` ➔ `apps/x32_autobeat/src/detection/spectral.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/effects/delay.rs` ➔ `apps/x32_autobeat/src/effects/delay.rs` [RENAME] | 4 | 4 | +9/-7 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/effects/fx_data.rs` ➔ `apps/x32_autobeat/src/effects/fx_data.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/effects/mod.rs` ➔ `apps/x32_autobeat/src/effects/mod.rs` [RENAME] | 2 | 2 | +1/-0 | +0/-3 | 🟡 MEDIUM |
| `x32_autobeat/src/effects/modulation.rs` ➔ `apps/x32_autobeat/src/effects/modulation.rs` [RENAME] | 2 | 2 | +4/-3 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/effects/reverb.rs` ➔ `apps/x32_autobeat/src/effects/reverb.rs` [RENAME] | 2 | 2 | +5/-4 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/main.rs` ➔ `apps/x32_autobeat/src/main.rs` [RENAME] | 1 | 1 | +17/-13 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/musical_theory.rs` ➔ `apps/x32_autobeat/src/musical_theory.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/network.rs` ➔ `apps/x32_autobeat/src/network.rs` [RENAME] | 6 | 6 | +32/-59 | +2/-10 | 🟡 MEDIUM |
| `x32_autobeat/src/scaling.rs` ➔ `apps/x32_autobeat/src/scaling.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/tests_scaling.rs` ➔ `apps/x32_autobeat/src/tests_scaling.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_autobeat/src/ui.rs` ➔ `apps/x32_autobeat/src/ui.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| apps/x32_automix/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_automix/README.md` ➔ `apps/x32_automix/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_automix/src/main.rs` ➔ `apps/x32_automix/src/main.rs` [RENAME] | 4 | 4 | +37/-50 | +2/-2 | 🟡 MEDIUM |
| apps/x32_reaper/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_reaper/README.md` ➔ `apps/x32_reaper/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_reaper/src/config.rs` ➔ `apps/x32_reaper/src/config.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_reaper/src/main.rs` ➔ `apps/x32_reaper/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_reaper/src/reaper_handler.rs` ➔ `apps/x32_reaper/src/reaper_handler.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_reaper/src/state.rs` ➔ `apps/x32_reaper/src/state.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_reaper/src/x32_handler.rs` ➔ `apps/x32_reaper/src/x32_handler.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tapw/Cargo.toml` ➔ `apps/x32_tapw/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tapw/README.md` ➔ `apps/x32_tapw/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tapw/src/app.rs` ➔ `apps/x32_tapw/src/app.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tapw/src/main.rs` ➔ `apps/x32_tapw/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tapw/tests/cli.rs` ➔ `apps/x32_tapw/tests/cli.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tapw/tests/emulator_test.rs` ➔ `apps/x32_tapw/tests/emulator_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| docs/ROADMAP.md | 0 | 0 | +28/-0 | +40/-0 | 🟢 LOW |
| docs/specs/01-gain-sharing-automix.md | 0 | 0 | +25/-0 | +25/-0 | 🟢 LOW |
| docs/specs/02-spl-eq-normalization.md | 0 | 0 | +21/-0 | +24/-0 | 🟢 LOW |
| docs/specs/03-phase-synced-fx.md | 0 | 0 | +21/-0 | +25/-0 | 🟢 LOW |
| docs/specs/04-ml-beat-tracking.md | 0 | 0 | +21/-0 | +26/-0 | 🟢 LOW |
| docs/specs/05-smart-proxy-digital-twin.md | 0 | 0 | +20/-0 | +25/-0 | 🟢 LOW |
| docs/specs/06-dual-protocol-transport.md | 0 | 0 | +28/-0 | +27/-0 | 🟢 LOW |
| docs/specs/07-dsp-assisted-intelligence.md | 0 | 0 | +20/-0 | +25/-0 | 🟢 LOW |
| git-publish.sh | 0 | 0 | +160/-0 | +19/-0 | 🟢 LOW |
| `osc_lib/Cargo.toml` ➔ `libs/osc_lib/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `osc_lib/README.md` ➔ `libs/osc_lib/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `osc_lib/src/lib.rs` ➔ `libs/osc_lib/src/lib.rs` [RENAME] | 0 | 0 | +1/-1 | +0/-0 | 🟡 MEDIUM |
| `osc_lib/src/tests.rs` ➔ `libs/osc_lib/src/tests.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_core/Cargo.toml` ➔ `libs/x32_core/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_core/README.md` ➔ `libs/x32_core/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_core/src/lib.rs` ➔ `libs/x32_core/src/lib.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_core/src/tests.rs` ➔ `libs/x32_core/src/tests.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/Cargo.toml` ➔ `libs/x32_lib/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/README.md` ➔ `libs/x32_lib/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| libs/x32_lib/src/client.rs | 8 | 0 | +85/-0 | +29/-0 | 🟢 LOW |
| `x32_lib/src/command/auxin.rs` ➔ `libs/x32_lib/src/command/auxin.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/bus.rs` ➔ `libs/x32_lib/src/command/bus.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/cfg_main.rs` ➔ `libs/x32_lib/src/command/cfg_main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/channel.rs` ➔ `libs/x32_lib/src/command/channel.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/dca.rs` ➔ `libs/x32_lib/src/command/dca.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/fx.rs` ➔ `libs/x32_lib/src/command/fx.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/fxrtn.rs` ➔ `libs/x32_lib/src/command/fxrtn.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/headamp.rs` ➔ `libs/x32_lib/src/command/headamp.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/libs.rs` ➔ `libs/x32_lib/src/command/libs.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/misc.rs` ➔ `libs/x32_lib/src/command/misc.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/mod.rs` ➔ `libs/x32_lib/src/command/mod.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/mtx.rs` ➔ `libs/x32_lib/src/command/mtx.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/output.rs` ➔ `libs/x32_lib/src/command/output.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/prefstat.rs` ➔ `libs/x32_lib/src/command/prefstat.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/command/show.rs` ➔ `libs/x32_lib/src/command/show.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/common.rs` ➔ `libs/x32_lib/src/common.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/error.rs` ➔ `libs/x32_lib/src/error.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/lib.rs` ➔ `libs/x32_lib/src/lib.rs` [RENAME] | 3 | 0 | +23/-0 | +34/-0 | 🟡 MEDIUM |
| `x32_lib/src/main_bus.rs` ➔ `libs/x32_lib/src/main_bus.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/scene_parse.rs` ➔ `libs/x32_lib/src/scene_parse.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/tests.rs` ➔ `libs/x32_lib/src/tests.rs` [RENAME] | 0 | 0 | +2/-0 | +0/-0 | 🟡 MEDIUM |
| libs/x32_lib/src/tests/async_methods.rs | 1 | 0 | +16/-0 | +12/-0 | 🟢 LOW |
| `x32_lib/src/tests/common.rs` ➔ `libs/x32_lib/src/tests/common.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/tests/error.rs` ➔ `libs/x32_lib/src/tests/error.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/tests/main_bus.rs` ➔ `libs/x32_lib/src/tests/main_bus.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/src/tests/output.rs` ➔ `libs/x32_lib/src/tests/output.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/tests/fx_tests.rs` ➔ `libs/x32_lib/tests/fx_tests.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_lib/tests/socket.rs` ➔ `libs/x32_lib/tests/socket.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tasks.json | 0 | 0 | +48/-0 | +0/-0 | 🟢 LOW |
| tools/x32_command/Cargo.toml | 0 | 0 | +3/-0 | +2/-0 | 🟡 MEDIUM |
| tools/x32_command/src/main.rs | 2 | 0 | +139/-0 | +42/-0 | 🟢 LOW |
| `x32_command/tests/cli_tests.rs` ➔ `tools/x32_command/tests/cli_tests.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_commander/Cargo.toml | 0 | 0 | +3/-0 | +2/-0 | 🟡 MEDIUM |
| `x32_commander/README.md` ➔ `tools/x32_commander/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_commander/src/main.rs` ➔ `tools/x32_commander/src/main.rs` [RENAME] | 1 | 2 | +32/-106 | +0/-40 | 🟡 MEDIUM |
| tools/x32_copy_fx/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_copy_fx/README.md` ➔ `tools/x32_copy_fx/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_copy_fx/src/fx_defaults.rs` ➔ `tools/x32_copy_fx/src/fx_defaults.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_copy_fx/src/main.rs` ➔ `tools/x32_copy_fx/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_cpxlivemarkers/Cargo.toml` ➔ `tools/x32_cpxlivemarkers/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_cpxlivemarkers/src/main.rs` ➔ `tools/x32_cpxlivemarkers/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_cpxlivemarkers/tests/cli.rs` ➔ `tools/x32_cpxlivemarkers/tests/cli.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_custom_layer/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_custom_layer/README.md` ➔ `tools/x32_custom_layer/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_custom_layer/src/main.rs` ➔ `tools/x32_custom_layer/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_restore/Cargo.toml` ➔ `tools/x32_desk_restore/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_restore/README.md` ➔ `tools/x32_desk_restore/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_restore/src/main.rs` ➔ `tools/x32_desk_restore/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_restore/src/parse.rs` ➔ `tools/x32_desk_restore/src/parse.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_restore/tests/integration_test.rs` ➔ `tools/x32_desk_restore/tests/integration_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_save/Cargo.toml` ➔ `tools/x32_desk_save/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_save/README.md` ➔ `tools/x32_desk_save/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_save/src/main.rs` ➔ `tools/x32_desk_save/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_save/src/nodes.rs` ➔ `tools/x32_desk_save/src/nodes.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_desk_save/tests/integration_test.rs` ➔ `tools/x32_desk_save/tests/integration_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_emulator/Cargo.toml | 0 | 0 | +3/-0 | +2/-0 | 🟡 MEDIUM |
| `x32_emulator/README.md` ➔ `tools/x32_emulator/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_emulator/src/lib.rs` ➔ `tools/x32_emulator/src/lib.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_emulator/src/main.rs` ➔ `tools/x32_emulator/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_fade/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_fade/README.md` ➔ `tools/x32_fade/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_fade/src/main.rs` ➔ `tools/x32_fade/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_geq2_cpy/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_geq2_cpy/README.md` ➔ `tools/x32_geq2_cpy/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_geq2_cpy/src/main.rs` ➔ `tools/x32_geq2_cpy/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_get_lib/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_get_lib/src/main.rs` ➔ `tools/x32_get_lib/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_get_scene/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_get_scene/README.md` ➔ `tools/x32_get_scene/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_get_scene/src/main.rs` ➔ `tools/x32_get_scene/src/main.rs` [RENAME] | 1 | 1 | +15/-11 | +2/-0 | 🟡 MEDIUM |
| `x32_get_scene_name/Cargo.toml` ➔ `tools/x32_get_scene_name/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_get_scene_name/README.md` ➔ `tools/x32_get_scene_name/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_get_scene_name/src/main.rs` ➔ `tools/x32_get_scene_name/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_get_scene_name/tests/integration_test.rs` ➔ `tools/x32_get_scene_name/tests/integration_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_jog4xlive/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_jog4xlive/README.md` ➔ `tools/x32_jog4xlive/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_jog4xlive/src/main.rs` ➔ `tools/x32_jog4xlive/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_midi2osc/Cargo.toml` ➔ `tools/x32_midi2osc/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_midi2osc/README.md` ➔ `tools/x32_midi2osc/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_midi2osc/src/config.rs` ➔ `tools/x32_midi2osc/src/config.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_midi2osc/src/main.rs` ➔ `tools/x32_midi2osc/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_midi2osc/src/rpn.rs` ➔ `tools/x32_midi2osc/src/rpn.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_midi2osc/src/rpn_calc.rs` ➔ `tools/x32_midi2osc/src/rpn_calc.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_punch_control/Cargo.toml | 0 | 0 | +3/-0 | +2/-0 | 🟡 MEDIUM |
| `x32_punch_control/README.md` ➔ `tools/x32_punch_control/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_punch_control/src/config.rs` ➔ `tools/x32_punch_control/src/config.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_punch_control/src/format.rs` ➔ `tools/x32_punch_control/src/format.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_punch_control/src/main.rs` ➔ `tools/x32_punch_control/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_punch_control/src/state.rs` ➔ `tools/x32_punch_control/src/state.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_replay/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_replay/src/main.rs` ➔ `tools/x32_replay/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_replay/src/main.rs.orig` ➔ `tools/x32_replay/src/main.rs.orig` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_set_lib/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_set_lib/src/main.rs` ➔ `tools/x32_set_lib/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_set_preset/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_set_preset/README.md` ➔ `tools/x32_set_preset/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_set_preset/src/main.rs` ➔ `tools/x32_set_preset/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_set_scene/Cargo.toml | 0 | 0 | +2/-0 | +1/-0 | 🟡 MEDIUM |
| `x32_set_scene/README.md` ➔ `tools/x32_set_scene/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_set_scene/src/main.rs` ➔ `tools/x32_set_scene/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_ssavergw/Cargo.toml | 0 | 0 | +3/-0 | +2/-0 | 🟡 MEDIUM |
| `x32_ssavergw/README.md` ➔ `tools/x32_ssavergw/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_ssavergw/src/main.rs` ➔ `tools/x32_ssavergw/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_ssavergw/src/state.rs` ➔ `tools/x32_ssavergw/src/state.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_ssavergw/tests/integration_test.rs` ➔ `tools/x32_ssavergw/tests/integration_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tap/Cargo.toml` ➔ `tools/x32_tap/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tap/README.md` ➔ `tools/x32_tap/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tap/src/main.rs` ➔ `tools/x32_tap/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tap/tests/cli.rs` ➔ `tools/x32_tap/tests/cli.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tap/tests/emulator_test.rs` ➔ `tools/x32_tap/tests/emulator_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_tcp/Cargo.toml | 0 | 0 | +3/-0 | +2/-0 | 🟡 MEDIUM |
| `x32_tcp/README.md` ➔ `tools/x32_tcp/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_tcp/src/main.rs` ➔ `tools/x32_tcp/src/main.rs` [RENAME] | 2 | 2 | +19/-18 | +1/-1 | 🟡 MEDIUM |
| `x32_tcp/tests/integration_test.rs` ➔ `tools/x32_tcp/tests/integration_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_udp/Cargo.toml | 0 | 0 | +3/-0 | +2/-0 | 🟡 MEDIUM |
| `x32_udp/src/main.rs` ➔ `tools/x32_udp/src/main.rs` [RENAME] | 1 | 1 | +8/-22 | +1/-1 | 🟡 MEDIUM |
| `x32_udp/tests/integration.rs` ➔ `tools/x32_udp/tests/integration.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/x32_usb/Cargo.toml | 0 | 0 | +3/-0 | +2/-0 | 🟡 MEDIUM |
| `x32_usb/README.md` ➔ `tools/x32_usb/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_usb/src/main.rs` ➔ `tools/x32_usb/src/main.rs` [RENAME] | 9 | 11 | +43/-57 | +2/-20 | 🟡 MEDIUM |
| `x32_usb/tests/integration_test.rs` ➔ `tools/x32_usb/tests/integration_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_wav_xlive/Cargo.toml` ➔ `tools/x32_wav_xlive/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_wav_xlive/README.md` ➔ `tools/x32_wav_xlive/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_wav_xlive/src/main.rs` ➔ `tools/x32_wav_xlive/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_wav_xlive/tests/integration_test.rs` ➔ `tools/x32_wav_xlive/tests/integration_test.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_xlive_wav/Cargo.toml` ➔ `tools/x32_xlive_wav/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_xlive_wav/src/main.rs` ➔ `tools/x32_xlive_wav/src/main.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `x32_xlive_wav/tests/integration.rs` ➔ `tools/x32_xlive_wav/tests/integration.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `xair_command/Cargo.toml` ➔ `tools/xair_command/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `xair_command/src/main.rs` ➔ `tools/xair_command/src/main.rs` [RENAME] | 0 | 0 | +14/-67 | +0/-15 | 🟡 MEDIUM |
| `xair_command/tests/cli.rs` ➔ `tools/xair_command/tests/cli.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `xair_get_scene/Cargo.toml` ➔ `tools/xair_get_scene/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/xair_get_scene/src/main.rs | 2 | 0 | +34/-0 | +6/-0 | 🟢 LOW |
| `xair_get_scene/tests/integration.rs` ➔ `tools/xair_get_scene/tests/integration.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `xair_set_scene/Cargo.toml` ➔ `tools/xair_set_scene/Cargo.toml` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| `xair_set_scene/README.md` ➔ `tools/xair_set_scene/README.md` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| tools/xair_set_scene/src/main.rs | 2 | 0 | +37/-0 | +7/-0 | 🟢 LOW |
| `xair_set_scene/tests/integration.rs` ➔ `tools/xair_set_scene/tests/integration.rs` [RENAME] | 0 | 0 | +0/-0 | +0/-0 | 🟡 MEDIUM |
| x32_automix/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_command/Cargo.toml | 0 | 0 | +0/-3 | +0/-2 | 🔴 HIGH |
| x32_command/src/main.rs | 0 | 2 | +0/-224 | +0/-69 | 🔴 HIGH |
| x32_commander/Cargo.toml | 0 | 0 | +0/-3 | +0/-2 | 🔴 HIGH |
| x32_copy_fx/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_custom_layer/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_emulator/Cargo.toml | 0 | 0 | +0/-3 | +0/-2 | 🔴 HIGH |
| x32_fade/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_geq2_cpy/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_get_lib/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_get_scene/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_jog4xlive/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_punch_control/Cargo.toml | 0 | 0 | +0/-3 | +0/-2 | 🔴 HIGH |
| x32_reaper/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_replay/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_set_lib/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_set_preset/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_set_scene/Cargo.toml | 0 | 0 | +0/-2 | +0/-1 | 🔴 HIGH |
| x32_ssavergw/Cargo.toml | 0 | 0 | +0/-3 | +0/-2 | 🔴 HIGH |
| x32_tcp/Cargo.toml | 0 | 0 | +0/-3 | +0/-2 | 🔴 HIGH |
| x32_udp/Cargo.toml | 0 | 0 | +0/-3 | +0/-2 | 🔴 HIGH |
| x32_usb/Cargo.toml | 0 | 0 | +0/-3 | +0/-2 | 🔴 HIGH |
| xair_get_scene/src/main.rs | 0 | 2 | +0/-53 | +0/-33 | 🔴 HIGH |
| xair_set_scene/src/main.rs | 0 | 2 | +0/-43 | +0/-29 | 🔴 HIGH |
