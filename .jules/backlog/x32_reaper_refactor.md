# TASK-004: Refactor x32_reaper to use MixerClient

## Objective
Modernize the X32-Reaper bidirectional bridge by migrating the X32 network layer to `MixerClient`.

## Implementation Details
1. **X32 Connection**:
   - Migrate from `UdpSocket` to `MixerClient::connect(x32_ip, true)`.
   - Use `client.subscribe()` to drive the main event loop for X32-to-Reaper updates.
2. **Reaper Connection**:
   - Maintain the existing `UdpSocket` for Reaper (Port 8000 default).
3. **State Management**:
   - Update `AppState` to handle async updates from both sources using `tokio::select!`.
   - Ensure `update_bk_ch` utilizes `client.send_message` for bulk bank updates.

## Success Criteria
- [ ] Bidirectional sync of faders, mutes, and transport works without regression.
- [ ] Bank switching correctly updates the X32 scribble strips and fader positions.
- [ ] `cargo test` integration tests for Reaper bridge pass.
