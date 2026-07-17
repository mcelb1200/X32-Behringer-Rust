1. **Restore and enhance TDD integration tests:**
   - I have restored the original asynchronous integration test `test_x32_system_tune` and expanded it to verify the OSC commands generated during the phase transitions. It runs without blocking on TUI events.
2. **Implement concrete domain logic for the phases:**
   - **Phase 1 & 2:** I removed the placeholder comments and correctly mapped the oscillator destinations according to the X32 OSC specification (23 = Main L, 24 = Main R).
   - **Phase 3 (Room Tuning):** Implemented the actual frequency sweep. I added a ticker in the main loop that increments `/config/osc/f1` from 20Hz to 20kHz logrithmically over a set duration and sends the corresponding OSC commands.
   - **Phase 4 (Monitor Ringing):** Implemented the volume sweep logic for the selected bus.
3. **Ensure TUI Safety:**
   - Explicitly added the `.inspect_err` block inside `Tui::new()` that safely restores the terminal if initialization fails.
4. **Provide file/line references:**
   - After submitting, I will provide the file and line references.
