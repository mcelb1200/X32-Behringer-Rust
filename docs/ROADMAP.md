# Architectural Roadmap: X32-Rust Evolution

## 📋 Recommended Workplan (Phase 1: Foundation)

| Task | Priority | Complexity | Description |
| :--- | :--- | :--- | :--- |
| **1. Dependency Unification** | Critical | **Small** | Move shared dependencies to root `Cargo.toml` workspace inheritance. |
| **2. Structural Reorganization**| High | **Medium**| Move crates into `libs/`, `apps/`, and `tools/` directories. |
| **3. `MixerClient` Abstraction** | High | **Medium**| Consolidate boilerplate (Heartbeat, Bounding, Bolt) into `x32_lib`. |
| **4. Unified `x32-cli`** | Medium | **Large** | Merge 20+ utility crates into a single subcommand-based binary. |
| **5. Rust-Native Testing** | Medium | **Large** | Port `.sh`/`.ps1` tests to native `assert_cmd` integration tests. |

---

## 🚀 Strategic Roadmap (Phase 2: Intelligent Automation)

The following advanced features are planned for future development. Detailed specifications (PRDs) can be found in `docs/specs/`.

### [01. Gain Sharing Automixing](specs/01-gain-sharing-automix.md)
*   **Goal**: Transition from binary gating to Dugan-style gain sharing.
*   **Status**: Planning.

### [02. SPL-Aware EQ Normalization](specs/02-spl-eq-normalization.md)
*   **Goal**: Apply Fletcher-Munson reciprocity based on Master Fader proxy SPL.
*   **Status**: Planning.

### [03. Phase-Synced Time Effects](specs/03-phase-synced-fx.md)
*   **Goal**: Rhythmic transient alignment for Delays and LFOs via PLL.
*   **Status**: Planning.

### [04. Probabilistic Beat Tracking](specs/04-ml-beat-tracking.md)
*   **Goal**: Comb Filter Bank analyzer for syncopation-robust BPM detection.
*   **Status**: Planning.

### [05. Smart Proxy Digital Twin](specs/05-smart-proxy-digital-twin.md)
*   **Goal**: High-fidelity shadow state mirroring and request coalescing in `x32_core`.
*   **Status**: Planning.

---

## 🔌 Phase 3: Hybrid Connectivity & Hardware Synergy

### [06. Dual-Protocol Transport](specs/06-dual-protocol-transport.md)
*   **Goal**: Support direct USB connection via MIDI System Exclusive (Sysex) as an alternative to Ethernet.
*   **Status**: Planning.

### [07. DSP-Assisted Intelligence](specs/07-dsp-assisted-intelligence.md)
*   **Goal**: Harvest high-resolution RTA and meter data directly from X32 internal DSP to reduce host CPU load.
*   **Status**: Planning.

---

## 🔬 Complexity Analysis & Estimates (Phase 1)

### 1. Dependency Unification (Est: 1-2 hours)
*   **Status**: ✅ COMPLETED (Integrated into main).

### 2. Structural Reorganization (Est: 2-4 hours)
*   **Status**: ✅ COMPLETED (Crates moved to libs/, apps/, tools/).

### 3. `MixerClient` Abstraction (Est: 1-2 days)
*   **Status**: ✅ COMPLETED (Consolidated boilerplate, heartbeat, and transport fallback into `x32_lib`).

### 4. Unified `x32-cli` (Est: 3-5 days)
*   **Status**: BACKLOG.

### 5. Rust-Native Testing (Est: Ongoing/Incremental)
*   **Status**: BACKLOG.

---

## 📦 Upstream C Library Parity Backlog (`upstream/master`)

Tracked items identified from comparing `pmaillot/X32-Behringer` (`upstream/master`) against this Rust workspace:

| Task | Upstream Source | Priority | Status | Description |
| :--- | :--- | :--- | :--- | :--- |
| **1. Consolidate `Xcparse` into `osc_lib`** | `X32lib/Xcparse.c` | Medium | ✅ COMPLETED | Consolidated single/double-quote tokenization and `OscMessage::parse_to_bytes` into `libs/osc_lib`; removed empty `libs/x32_xcparse` crate. |
| **2. X-Air FX & Scene Parsers** | `XAir_fxparse.c`, `XAir_fxparse1.c`, `XAirSetSceneParse.c` | High | BACKLOG | Port X-Air (XR12/XR16/XR18) FX and scene line parsers for full X-Air scene/FX restoration parity. |
| **3. Emulator `/meters` Parity Fixes** | `X32.c` (`84fb370`, `0178461`) | High | BACKLOG | Port 4-byte float padding and `time_factor` handling for `/meters/5` and `/meters/6` into `tools/x32_emulator` to improve offline RTA/meter testing. |
| **4. `X32ReaperW.c` New OSC/Reaper Mappings** | `X32ReaperW.c` (`783fe42`) | Medium | BACKLOG | `apps/x32_reaper/src/lib.rs` already covers core X32↔Reaper sync; remaining new `X32ReaperW.c` mappings to port: ReaEQ/ReaComp FX sync (`eq/on`↔`fx/1/bypass`, `dyn/on`↔`fx/2/bypass`, `fxparam` bands, `/-stat/screen/CHAN/page`↔`openui`), User-Bank C buttons `33..36`, and extended Reaper transport/marker actions (`40073`, `40172`/`40173`, `40294`, `40318`/`40319`, `40839`–`40842`, `41743`, `53808`). |
