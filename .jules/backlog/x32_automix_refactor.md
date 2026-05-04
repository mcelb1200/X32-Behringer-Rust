# Task: Refactor x32_automix to use MixerClient

## Context
`x32_automix` implements a Dugan-style gain-sharing automixer for the X32/M32. It requires high-speed polling of input levels (meters) and rapid updates to fader/gain levels.

## Goal
Migrate the network and polling logic to `x32_lib::MixerClient`.

## Required Components & Logic

### 1. Network & Polling
- **Replace** manual `UdpSocket` with `MixerClient::connect(ip, true)`.
- **Implement** a high-frequency polling loop that:
    - Subscribes to `/meters/0` or `/meters/1` for input channel levels.
    - Uses `client.subscribe()` to receive these meter blobs.
    - Decodes the meter data to extract raw input levels.

### 2. Automix Algorithm
- **Preserve** the core gain-sharing logic:
    - Calculation of total power across all active automix channels.
    - Application of weights (if applicable).
    - Smoothing/Attack/Release parameters for gain adjustments.
- **Ensure** the algorithm correctly handles "priority" or "weight" settings per channel.

### 3. Fader Updates
- **Use** `client.send_message` (or a more optimized batch update if supported) to send gain corrections back to the mixer.
- **Maintain** the logic that prevents the automixer from fighting with physical fader moves by the user.

### 4. Configuration
- **Preserve** command-line arguments for channel selection, threshold, and algorithm sensitivity.

## Constraints
- **Timing is critical**: The automixer must process meter data and send updates with minimal jitter.
- Avoid excessive network congestion; ensure updates are only sent when gain changes exceed a small delta.
