# TODO

## Future Enhancements
1. **Automatic DCA Spills**: Automatically spill DCA members onto a custom fader bank for quick access. Architectural Considerations: Needs an OSC listener to detect DCA select button presses and dynamically rewrite custom layer mappings (`/-prefs/custom_bank/`). Must handle latency tightly to make spills feel instantaneous to the engineer.
2. **Crossfading Snapshots**: Smoothly interpolate faders, EQs, and dynamics parameters between two scenes. Architectural Considerations: Current scene changes are instantaneous (`/load`). This needs a background tick-loop to tween float values over time. Requires state caching of both scenes to calculate deltas without overwhelming the console with `/node` requests.
3. **Auto-Mixing NOM (Number of Open Mics) Extension**: Expand the Dugan-style automixer with dynamic background noise tracking and priority ducking. Architectural Considerations: High-frequency OSC polling (`/meters/1`) can cause network congestion. Must implement a UDP throttling mechanism and an efficient algorithm to compute gain shares on a dedicated realtime-priority thread to prevent audio pumping.
