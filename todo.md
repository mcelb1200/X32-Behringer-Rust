# TODO

- Reconcile and merge all performance, security compliance, and porting PRs into `refactor/mixer-client-abstraction` (Completed)
- Extract 'todo' items from reference C files in `c_origin/` and document them here for future work.
  - Evaluate `// TODO: How to select Master from REAPER???` from `X32Reaper.c`
  - Evaluate `// TODO: do a proper implementation of the function` for `/add` and `/load` from `X32.c` (Completed)
  - Evaluate `// TODO: implement 'libfx' and 'librout'` from `X32.c` (Completed)

## Context
These TODOs are legacy markers from the original C codebase that was ported to Rust. Future agents should review these items to verify whether the ported Rust components (like `x32_reaper` and `x32_core`) fully implement the intended functionality or if additional logic is needed to resolve the original author's comments.


## Remaining Legacy TODOs
The following are all the remaining 'todo' items found in `c_origin/`:
  - `//char* r_CS[] = { "INT", "AES50A", "AES50B", "CARD" };	//Todo: Validate "CARD"` from `X32DeskRestore.c:694`
  - `//// todo: validate if the use of the "/~~~" command would work here; is it` from `X32DeskRestore.c:794`
  - `////todo: one or 80 lines?` from `X32DeskRestore.c:981`
  - `////todo: ... Last one is a string, unknown for now... could it correspond to iQ/group???` from `X32DeskRestore.c:1057`
  - `////todo: Decay sent as a logf or as a float with 2 digits after decimal point?` from `X32DeskRestore.c:1107`
  - `// the same way as REW clicks but does not behave similarly... TODO therefore` from `X32PunchControl.c:357`
  - `//todo` from `X32Commander.c:688` (Completed: Implemented string handling and RPN/NRPN generation)
  - `// Effects defaults. The trailing zeroes are not necessary - No big deal but todo: remove them!` from `X32CopyFX.c:99`
  - `//						// TODO: 1 if any solosw is still on, 0 otherwise` from `X32.c:1186`
  - `s_len = Xsprint(s_buf, s_len, 's', ",b"); //todo - incorrect` from `X32.c:3006`
  - `s_adr += ((strlen(s_adr) + 4) & ~3); //todo - incorrect` from `X32.c:3026`
  - `return 0; // no room for new clients! (todo; another return status?)` from `X32.c:3136`
  - `// Todo?` from `X32.c:3767`
  - `strcpy(s_buf + s_len, " TODO");` from `X32.c:4516`
  - `XClientMeters[i] = *Client_ip_pt;			// remember requesting IP client TODO: not the right approach` from `X32.c:4842`
  - `// Ignored for now / Todo` from `X32.c:4956`
  - `// TODO: 'scene', 'libfx', 'librout' should be implemented` from `X32.c:5022` (Completed: Implemented `/copy` and `/delete` for these types in `x32_core`)
  - `// todo: this is not actually done here!` from `X32.c:5177` (Completed: Implemented `hasdata` updates for `/save` and `/delete` in `x32_core`)
  - `// TODO At this time the last two <string><int> parameters are ignored` from `X32.c:5213`
  - `// todo: this is not actually done here!` from `X32.c:5221` (Completed: Implemented `hasdata` updates for `/save` and `/delete` in `x32_core`)
  - `// todo: this is not actually done here!` from `X32.c:5281` (Completed: Implemented `hasdata` updates for `/save` and `/delete` in `x32_core`)
  - `// todo: this is not actually done here!` from `X32.c:5315` (Completed: Implemented `hasdata` updates for `/save` and `/delete` in `x32_core`)
  - `// !! TODO find Master track REAPER numbering` from `X32Reaper.c:1522`

## Future Enhancements
1. **Automatic DCA Spills**: Automatically spill DCA members onto a custom fader bank for quick access. Architectural Considerations: Needs an OSC listener to detect DCA select button presses and dynamically rewrite custom layer mappings (`/-prefs/custom_bank/`). Must handle latency tightly to make spills feel instantaneous to the engineer.
2. **Intelligent Feedback Suppression / Spectral Ducking (Implemented)**: Auto-detect feedback frequencies and inject surgical notch filters into the master/bus EQs.
   * *Status*: Completed via `x32_vocal_ducking` using real-time audio FFT (via USB CPAL) or meter polling (via OSC) and dynamic PEQ Band 3 updates.
3. **Crossfading Snapshots**: Smoothly interpolate faders, EQs, and dynamics parameters between two scenes. Architectural Considerations: Current scene changes are instantaneous (`/load`). This needs a background tick-loop to tween float values over time. Requires state caching of both scenes to calculate deltas without overwhelming the console with `/node` requests.
4. **Auto-Mixing NOM (Number of Open Mics) Extension**: Expand the Dugan-style automixer with dynamic background noise tracking and priority ducking. Architectural Considerations: High-frequency OSC polling (`/meters/1`) can cause network congestion. Must implement a UDP throttling mechanism and an efficient algorithm to compute gain shares on a dedicated realtime-priority thread to prevent audio pumping.
5. **Multi-Console Sync**: Mirror specific channels, busses, or master state across two or more X32s (e.g., FOH and Monitors). Architectural Considerations: Requires a robust TCP or UDP bidirectional bridge. Needs loop-prevention logic (split-horizon) so updates don't ping-pong infinitely between consoles. Must handle connection drops gracefully.
