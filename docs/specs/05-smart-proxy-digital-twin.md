# PRD: Smart Proxy & Digital Twin Architecture

## 1. Executive Summary
Transform `x32_core` into a **Stateful Smart Proxy**. Instead of individual tools competing for network bandwidth and synchronization, they connect to a local proxy that maintains a high-fidelity "Digital Twin" of the console's state.

## 2. Technical Specification

### 2.1 The "Twin" Model
*   **Shadow State:** The proxy maintains a 1:1 mirror of all console parameters in memory.
*   **Delta-Push:** Only changed parameters (deltas) are sent to the hardware, reducing network overhead.
*   **Pre-flight Checks:** Complex automations (like Gain Sharing) are calculated against the Shadow State first; only validated changes are pushed to the physical console.

### 2.2 Virtualization Layer
*   Expose a virtual OSC server to local tools.
*   Implement "Request Coalescing": If 5 tools ask for `/ch/01/mix/fader`, the proxy sends only 1 request to the mixer and broadcasts the result to all 5.

## 3. Gaps & Limitations
*   **Latency Jitter:** The proxy adds a "hop" between tools and hardware (~1-2ms).
*   **State Drift:** If a human moves a fader on the physical console, the proxy must detect it instantly via `/xremote` or the "Twin" becomes invalid.

## 4. Alternative Implementations & Redundancy

### Interface 1: Local OSC Relay (High Compatibility)
*   **Approach:** Proxy acts as a standard OSC server.
*   **User Value:** Any standard OSC app (TouchOSC, Lemur) can connect to the proxy and benefit from advanced logic.
*   **Redundancy:** Provides a broad interface for multi-device control surfaces.

### Interface 2: Unix Domain Socket / IPC (Low Latency)
*   **Approach:** Local tools talk to the proxy via high-speed Inter-Process Communication.
*   **User Value:** Bypasses the network stack entirely; sub-millisecond local latency.
*   **Redundancy:** Dedicated interface for local "headless" automation scripts.

## 5. Directory Structure
```text
libs/x32_core/src/
├── proxy/
│   ├── twin.rs         <-- State mirroring logic
│   ├── coalescer.rs    <-- Request merging
│   └── validator.rs    <-- Safety/Rule engine
└── server.rs           <-- Virtual OSC listener
```

## 6. Verification Plan
*   **Drift Test:** Verify that manual fader moves on the physical console update the Shadow State in <10ms.
*   **Stress Test:** Connect 20 simultaneous CLI tools to the proxy and verify that only 1 OSC query per unique path is sent to the mixer.
