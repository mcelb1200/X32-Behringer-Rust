# Task: Refactor x32_usb to use MixerClient (Restore full functionality)

## Context
The previous refactor attempt for `x32_usb` significantly regressed the tool, reducing it to a simple status check. We need to restore the full "USB Shell" functionality while correctly utilizing the new `MixerClient` abstraction.

## Goal
Restore all interactive USB management features (LS, CD, LOAD, etc.) and refactor the network layer to use `x32_lib::MixerClient`.

## Required Components & Logic

### 1. Interactive Shell Features (TO RESTORE)
- **`Ls`**: List directory contents by querying `/-usb/dir/maxpos` and iterating through `/-usb/dir/XXX/name`.
- **`Cd`**: Change directory by selecting a folder index.
- **`Load`**: Load scenes (`.scn`), snippets (`.snp`), effects (`.efx`), routing (`.rou`), preference (`.prf`), or channel (`.chn`) files.
- **`Play` / `Stop` / `Pause` / `Resume`**: Full control over the USB WAV player/recorder.
- **`Umount`**: Safely unmount the USB drive.

### 2. MixerClient Integration
- **Connect** using `MixerClient::connect(ip, true)`.
- **Replace** the custom `X32Client` struct (which used `create_socket` and manual `recv`) with the standard `MixerClient`.
- **Implement** `is_usb_mounted`, `get_file_list`, and `find_file` helper methods using `client.query_value` and `client.send_message`.

### 3. File & Type Handling
- **Maintain** the `FileType` enum and its extension-based detection logic.
- **Maintain** the `FileEntry` struct for representing USB filesystem items.

## Constraints
- **Do NOT** use the regressed 27-line version from the recent ZIP.
- Ensure the user can still use both numeric IDs and string names for `cd` and `load` targets.
- Ensure `cargo test` and integration tests in `tools/x32_usb/tests/` pass.
