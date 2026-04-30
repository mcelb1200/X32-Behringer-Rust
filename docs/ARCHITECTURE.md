# X32 Tools Architecture

This document provides a deep dive into the technical design, networking model, and security principles of the X32 Tools Rust suite.

---

## 🏗️ System Overview

The project is designed as a **decoupled workspace**. Logic is strictly separated into three layers:

1.  **Transport & Protocol (`osc_lib`):** Handles the raw byte serialization and deserialization of the Open Sound Control (OSC) protocol. It remains agnostic of the mixer's specific commands.
2.  **Domain Logic (`x32_lib`):** Defines the X32/M32/XAir command set. It handles parameter scaling (e.g., converting a 0.0-1.0 float to a logarithmic fader dB value) and command generation.
3.  **Application Layer (Binary Crates):** Specialized tools that use the libraries to perform specific tasks.

---

## 📡 Networking Model

All tools communicate with the mixer via **UDP (User Datagram Protocol)**, which is the native transport for X32 OSC.

### `/xremote` and Client Tracking
The X32 mixer only sends state updates to clients that have explicitly requested them via the `/xremote` command. These subscriptions expire every 10 seconds.
*   **Heartbeat Mechanism:** Tools that require real-time updates (like `x32_reaper` or `x32_tapw`) implement an async heartbeat loop to renew the `/xremote` subscription.
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
