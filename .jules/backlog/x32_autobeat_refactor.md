# TASK-005: Refactor x32_autobeat to use MixerClient

## Objective
Migrate the beat detection and synchronization engine to `MixerClient`.

## Implementation Details
1. **Network Manager**:
   - Refactor `NetworkManager` to encapsulate `MixerClient`.
   - Use `client.subscribe()` to monitor `/meters/6` (Main Bus) for beat detection.
2. **Detection Logic**:
   - Ensure `SpectralFluxDetector` and `EnergyDetector` are fed from the client's message stream.
3. **Effect Synchronization**:
   - Use `client.send_message` to update delay times based on detected BPM.

## Success Criteria
- [ ] Tool correctly syncs X32 delay slots to system audio or OSC meter inputs.
- [ ] TUI remains responsive and correctly displays connection status from `MixerClient`.
- [ ] All missing files (audio.rs, energy.rs, etc.) from the previous regressed ZIP are confirmed present and integrated.
