# PRD: Dual-Protocol Transport (OSC + USB MIDI)

## 1. Executive Summary
Extend the X32 Tool suite to support direct USB connection via the X-USB/X-LIVE expansion card. This provides a zero-network configuration option for controlling the mixer using **OSC-over-MIDI Sysex**.

## 2. Technical Specification

### 2.1 The Trait-Based Transport Model
Abstract the communication layer into a `MixerTransport` trait to allow seamless switching between Ethernet (UDP) and USB (MIDI).

```rust
#[async_trait]
pub trait MixerTransport {
    async fn send(&self, msg: OscMessage) -> Result<()>;
    async fn recv(&self) -> Result<OscMessage>;
}
```

### 2.2 USB Implementation (OSC over MIDI Sysex)
*   **Protocol:** MIDI System Exclusive (Sysex).
*   **Encapsulation:** `F0 00 20 32 32 <OSC_PAYLOAD> F7`.
*   **Library:** Use the `midir` crate for cross-platform MIDI access.

### 2.3 Hybrid Connection Strategy
*   **Auto-Discovery:** Application attempts to find an "X-USB" or "X-LIVE" MIDI port if no IP address is provided.
*   **Failover:** If network latency is high, the system can fallback to USB if connected.

## 3. Gaps & Limitations
*   **Bandwidth:** MIDI is significantly slower than Ethernet (31.25 kbit/s vs 100+ Mbit/s). High-frequency meter updates (RTA) may be throttled over USB.
*   **OSC Binary vs. ASCII:** Some X32 firmware versions prefer raw ASCII strings in Sysex rather than binary OSC packets.

## 4. Alternative Implementations & Redundancy

### Path 1: Ethernet UDP (Primary)
*   **Approach:** Standard high-speed OSC over Network.
*   **User Value:** Lowest latency; support for multiple concurrent control devices.
*   **Redundancy:** Preferred primary transport for all scenarios.

### Path 2: USB MIDI Sysex (Failover/Standalone)
*   **Approach:** Encapsulate OSC within MIDI Sysex via the X-USB/X-LIVE card.
*   **User Value:** Zero network configuration; direct physical connection to the PC.
*   **Redundancy:** Automatic failover transport if the network connection is lost during a performance.

## 5. Directory Structure
```text
libs/x32_lib/src/
├── transport/
│   ├── mod.rs          <-- Transport Trait
│   ├── udp.rs          <-- Current Ethernet logic
│   └── midi.rs         <-- New: USB MIDI Sysex logic
```

## 6. Verification Plan
*   **Loopback Test:** Send a Sysex-wrapped OSC message and verify it is correctly parsed by the `osc_lib`.
*   **Hardware Verification:** Connect to an X32 via USB and verify a fader move results in a console update.
