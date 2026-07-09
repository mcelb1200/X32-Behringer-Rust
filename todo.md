# TODO

## Future Enhancements
1. (Completed) **Automatic DCA Spills**: Automatically spill DCA members onto a custom fader bank for quick access. Architectural Considerations: Needs an OSC listener to detect DCA select button presses and dynamically rewrite custom layer mappings (`/-prefs/custom_bank/`). Must handle latency tightly to make spills feel instantaneous to the engineer.
2. (Completed) **Crossfading Snapshots**: Smoothly interpolate faders, EQs, and dynamics parameters between two scenes. Architectural Considerations: Current scene changes are instantaneous (`/load`). This needs a background tick-loop to tween float values over time. Requires state caching of both scenes to calculate deltas without overwhelming the console with `/node` requests.
3. (Completed) **Auto-Mixing NOM (Number of Open Mics) Extension**: Expand the Dugan-style automixer with dynamic background noise tracking and priority ducking. Architectural Considerations: High-frequency OSC polling (`/meters/1`) can cause network congestion. Must implement a UDP throttling mechanism and an efficient algorithm to compute gain shares on a dedicated realtime-priority thread to prevent audio pumping.

## Volunteer & Inexperienced User Improvements
1. **Automatic Feedback Detection and Management**: Dynamically identify and notch out ringing frequencies on vocal/instrument buses before they become audible, using high-precision FFT on the USB mode fallback and surgical PEQ notch EQ insertion.
2. **Auto-Gain / Smart Gain Staging**: Provide a macro that listens to peak levels on selected channels and automatically sets HA gain to an optimal target (e.g., -18dBFS), minimizing clipping for new operators.
3. **One-Touch "Speech Mode" Macro**: A simplified interface or command that automatically engages optimal EQ presets (high-pass filters), Dugan automixing, and mild compression for speech microphones.
4. **"Safe Mute" / Panic Button**: A global feature that instantly mutes all outputs or selected groups with a gradual but fast fade-out to prevent pops while protecting the PA system.
5. **Intelligent Scene Pre-flight Checker**: A pre-load diagnostic tool that compares current routing/outputs with an incoming scene and warns the user if major patch changes might mute the PA or cause feedback loops.
6. **Simplified TUI Dashboard (Volunteer Mode)**: A clean, minimal Terminal User Interface (TUI) overlay that abstracts away complex channel strips, showing only large faders, essential mutes, and clear visual indicators for clipping/feedback.
