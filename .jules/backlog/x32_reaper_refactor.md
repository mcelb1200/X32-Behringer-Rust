# Task: Refactor x32_reaper to use MixerClient

## Context
`x32_reaper` is a complex bridge between the Behringer X32 and Reaper DAW. It manages bidirectional synchronization of faders, mutes, pans, and transport controls. It currently handles raw `UdpSocket` for both X32 and Reaper communications.

## Goal
Refactor the X32 communication layer to use `x32_lib::MixerClient`. Reaper communication (on a separate port/IP) should remain as a standard `tokio::net::UdpSocket` but could be abstracted if beneficial.

## Required Components & Logic

### 1. X32 Integration (`main.rs`, `x32_handler.rs`)
- **Replace** the X32-specific `UdpSocket` with `MixerClient::connect(config.x32_ip, true)`.
- **Update** `connect_x32` to utilize the `MixerClient` connection lifecycle.
- **Modernize** the main loop to use `client.subscribe()` for incoming X32 messages.
- **Ensure** `send_to_x` uses `client.send_message`.

### 2. Bidirectional Mapping Logic
- **Preserve** all mapping logic between X32 OSC paths (e.g., `/ch/01/mix/fader`) and Reaper OSC paths (e.g., `/track/1/volume`).
- **Maintain** the "bank" logic (`update_bk_ch`) which allows the X32 to control arbitrary sets of Reaper tracks.
- **Preserve** handling of:
    - Fader, Pan, Mute, Solo.
    - Scribble strip updates (Name, Color, Icon).
    - Transport controls (Play, Stop, Record, etc.).
    - User-defined controls (Encoders and Buttons).

### 3. State & Configuration
- **Preserve** `AppState` (`state.rs`) for tracking the current state of both systems.
- **Maintain** `Config` (`config.rs`) parsing from `.ini` files.
- **Ensure** `init_user_ctrl` correctly initializes the X32 User Control section.

### 4. Reaper Communication
- **Maintain** the `reaper_handler.rs` logic for processing incoming messages from Reaper.
- **Ensure** Reaper messages are correctly translated and forwarded to the `MixerClient`.

## Constraints
- Must maintain the high-performance, low-latency nature of the bridge.
- Do **not** break the existing logic for DCA and Master fader synchronization.
- Ensure all integration tests in `main.rs` pass or are updated to work with `MixerClient`.
