# Task: Refactor x32_autobeat to use MixerClient

## Context
The workspace is migrating to a centralized, async `MixerClient` abstraction for all X32/M32 communications. `x32_autobeat` currently uses manual `tokio::net::UdpSocket` management in its `NetworkManager`.

## Goal
Rewrite the network layer of `x32_autobeat` to utilize `x32_lib::MixerClient`. The core logic of beat detection and UI must remain intact.

## Required Components & Logic

### 1. Network Layer (`network.rs`)
- **Replace** manual socket management with `MixerClient`.
- **Implement** `NetworkManager` as an async task that:
    - Connects using `MixerClient::connect(ip, true)` (heartbeat enabled).
    - Uses `client.subscribe()` to listen for meter data (`/meters/6`).
    - Subscribes to necessary OSC paths for effect monitoring.
    - Sends heartbeat/keepalive commands via the client's internal mechanism.
- **Maintain** the `NetworkEvent` and `UIEvent` channel-based communication.

### 2. Audio & Detection Logic
- **Preserve** `AudioEngine` (`audio.rs`) integration for system audio capture.
- **Preserve** all detection algorithms in `detection/`:
    - `EnergyDetector`
    - `SpectralFluxDetector`
    - `OscLevelDetector` (updated to receive messages from the client subscriber).
- **Ensure** `BeatDetector` state machine correctly processes timing from both audio and OSC sources.

### 3. Effects Handling (`effects/` and `compressor.rs`)
- **Preserve** the `EffectHandler` trait and its implementations.
- **Update** `EffectHandler::apply` to be `async` (if not already) and take a `&MixerClient`.

### 4. UI Layer (`ui.rs` and `main.rs`)
- **Maintain** the `ratatui` TUI.
- **Ensure** the main loop correctly selects between UI events and network events using `tokio::select!`.

## Constraints
- Do **not** remove support for any existing command-line arguments.
- Maintain compatibility with the existing subdivision and musical theory logic in `musical_theory.rs`.
- Ensure `cargo test` and `cargo clippy` pass after refactoring.
