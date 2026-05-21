# TASK-002: Refactor x32_usb to use MixerClient (Restore full functionality)

## Objective
Restore the "USB Shell" interactive functionality (missing in recent regressed versions) using the `MixerClient` API.

## Implementation Details
1. **Interactive Commands**:
   - `Ls`: Query `/-usb/dir/maxpos` then loop `/-usb/dir/%03d/name`.
   - `Cd <target>`: Support both numeric index and string name matching.
   - `Load <target>`: Execute `/-action/recselect` for valid scene/preset files.
2. **Transport Control**:
   - Map `Play/Stop/Pause/Resume` to `/-stat/tape/state` (0=Stop, 1=Pause, 2=Play).
3. **Network Layer**:
   - Initialize with `MixerClient::connect(ip, true)`.
   - Use `client.query_value` for all synchronous filesystem metadata requests.

## Success Criteria
- [ ] Binary `x32_usb` supports all subcommands: `ls`, `cd`, `load`, `play`, `stop`, `pause`, `resume`, `umount`.
- [ ] Integration tests in `tools/x32_usb/tests/integration_test.rs` pass using the emulator.
- [ ] No raw `UdpSocket` usage remains in `tools/x32_usb/src/main.rs`.
