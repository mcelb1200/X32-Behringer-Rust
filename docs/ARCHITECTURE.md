# X32 Tools Architecture

This document provides a deep dive into the technical design, networking model, and security principles of the X32 Tools Rust suite.

---

## 🏗️ System Overview

The project is designed as a **decoupled workspace**. Logic is strictly separated into three layers:

1.  **Transport & Protocol (`osc_lib`):** Handles the raw byte serialization and deserialization of the Open Sound Control (OSC) protocol. It remains agnostic of the mixer's specific commands.
2.  **Domain Logic (`x32_lib`):** Defines the X32/M32/XAir command set, parameter scaling, and the **`MixerClient` abstraction** that provides unified, async, multi-transport connections and query-response tracking.
3.  **Application Layer (Binary Crates):** Specialized tools that use the libraries to perform specific tasks.

### Unified `x32-cli` Monolith Architecture
While the workspace contains many individual binary crates for development and testing isolation, the primary distribution and execution model is the **unified `x32-cli` monolith**.
*   **Subcommand Routing:** All individual tools (e.g., `x32_emulator`, `x32_autobeat`, `x32_desk_save`) expose their core logic via library functions that are aggregated as `clap` subcommands within `tools/x32_cli`.
*   **Shared Dependencies:** This monolithic approach significantly reduces final compile times and binary bloat by ensuring all tools share the exact same `tokio`, `clap`, and `osc_lib` instances statically linked into a single executable.
*   **Execution:** Users invoke tools via `x32-cli <subcommand>` (e.g., `x32-cli x32-emulator`), simplifying PATH management and deployment.

---

## 📡 Networking & Connection Model

All tools communicate with the mixer using the unified `MixerClient`.

### Multi-Transport & Fallback Hierarchy
`MixerClient` supports both UDP (network OSC and AES50 tunneled OSC) and USB MIDI System Exclusive (Sysex) transports. Connections utilize a prioritized auto-fallback mechanism:
1.  **Ethernet OSC**: Standard UDP control over the primary network IP.
2.  **AES50 Tunneled OSC**: UDP control routed through AES50 ethernet extension.
3.  **USB MIDI Sysex**: Fallback local control via USB MIDI connection.

### `/xremote` and Client Tracking
The X32 mixer only sends state updates to clients that have explicitly requested them via the `/xremote` command. These subscriptions expire every 10 seconds.
*   **Heartbeat Mechanism**: `MixerClient` runs a background task that automatically sends `/xremote` keep-alives every 9 seconds, freeing the binary tools from managing heartbeat boilerplate.
*   **Emulator Support:** The `x32_core` emulator tracks up to 4 concurrent clients and automatically broadcasts state changes to all registered subscribers, mimicking the behavior of physical hardware.

---

## 🤖 The Emulator (`x32_core`)

The `x32_core` crate provides a headless, stateful simulation of an X32 console.
*   **State Store:** Uses a thread-safe `HashMap` to track the current value of every OSC path.
*   **Dispatch Loop:** Processes incoming bytes, updates the internal state for "set" commands, and generates response packets for "queries".
*   **TDD Foundation:** The emulator allows for exhaustive integration testing without requiring a physical $3,000 console.

---

## 🛡️ Security & Reliability (Sentinel)

The project adheres to the **Sentinel** design philosophy: *All inputs from the outside world (Network & Files) are untrusted and must be bounded.*

### OOM Prevention
Early versions were vulnerable to Out-of-Memory (OOM) attacks via unbounded file reads (e.g., passing `/dev/zero` as a config file).
*   **Stream Bounding:** All file-reading utilities now use `take(MAX_BYTES)` to ensure that memory allocation is strictly capped, regardless of what the OS reported for file size.
*   **Network Buffer Limits:** OSC packet sizes are validated at the socket level to prevent buffer overflows or excessive allocation.

---

## ⚡ Performance Optimization (Bolt)

Latency is critical in live audio control. The **Bolt** initiative focuses on reducing the "OSC-to-Action" delay.
*   **Zero-Alloc Serialization:** High-frequency paths (like meter monitoring) use pre-allocated buffers and direct byte manipulation instead of expensive string formatting or `write!` macros.
*   **Async I/O:** Built on `tokio`, the suite handles concurrent network traffic and file I/O without blocking the main event loops.

---

## ⏱️ Latency & External DSP (RT-Linux `PREEMPT_RT`)

External audio processing vs. hybrid analysis comparison under RT-Linux kernel.

### Latency Comparison Matrix

| Option | Audio RTL | Control Lag | Phase Cancellation Risk | Optimal Use Case | Target Hardware / Bus |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Full External DSP** (X-USB $\rightarrow$ Host $\rightarrow$ X-USB) | ~4.8ms | N/A (Direct) | High (Comb filtering if mixed) | Reverb, Delay FX (Time-domain) | RT-patched PC, USB 2.0 (32-sample buffer) |
| **2. Hybrid Control** (X-USB $\rightarrow$ PC Analyze $\rightarrow$ OSC $\rightarrow$ X32) | ~1.6ms (Console path) | ~25–35ms | Zero (Coherent analog path) | Ducking, Automixer, FB Suppression | PC (RT/Standard), LAN/UDP |
| **3. AES50 Hardware DSP** (AES50 $\rightarrow$ FPGA Card $\rightarrow$ AES50) | ~2.5ms | N/A (Direct) | Low | Live Inserts (Channel EQ/Comp) | Waves SoundGrid / AES50 expansion card |

### Architectural Tradeoffs

#### 1. Full External DSP Loop
* **Causality**: AD/DA $\rightarrow$ X32 Internal Route $\rightarrow$ USB Tx buffer $\rightarrow$ RT-Linux ALSA $\rightarrow$ Host DSP engine $\rightarrow$ ALSA Tx $\rightarrow$ USB Rx $\rightarrow$ X32 Aux/Input $\rightarrow$ DA.
* **Math (RT-Linux @ 48kHz, 32-sample buffer)**:
  $$\text{Console AD/DA}~(1.6\text{ms}) + \text{Console DSP}~(0.83\text{ms}) + \text{USB I/O Buffer}~(2 \times 0.67\text{ms} = 1.33\text{ms}) + \text{USB Bus Overhead}~(\sim1.0\text{ms}) + \text{RT Kernel Jitter}~(<0.05\text{ms}) \approx 4.8\text{ms}$$
* **Limitation**: Comb filtering occurs if processed dry/wet signals merge on console. Latency is close to human tolerance limit for IEMs ($\sim5\text{ms}$).

#### 2. Hybrid Analysis & OSC Control
* **Causality**:
  * **Audio**: Direct analog input $\rightarrow$ X32 DSP $\rightarrow$ Output (constant $1.6\text{ms}$).
  * **Control**: X-USB $\rightarrow$ PC (RT-patched) $\rightarrow$ FFT/Spectral Analysis $\rightarrow$ OSC over UDP $\rightarrow$ X32 Parameter update.
* **Math**:
  $$\text{X-USB Capture}~(\sim2.2\text{ms}) + \text{FFT Window}~(512\text{ samples} \approx 10.6\text{ms}) + \text{OSC Network Tx}~(<0.5\text{ms}) + \text{X32 CPU Parse}~(\sim10\text{ms}) \approx 23.3\text{ms}$$
* **Advantage**: Zero phase cancellation. Clean direct sound. Perfectly suited for dynamic parameter controls (e.g. vocal ducking trigger times).
