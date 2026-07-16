1. **Analyze existing components and the feature roadmap:**
   - `todo.md` requests a `x32_system_tune` app with 4 phases (Output Verification, Assisted Gain Staging, Room Tuning, Monitor Ringing).
   - Currently, `apps/x32_system_tune` implements a basic version that just ramps the oscillator up.
   - I have added `crossterm` and `ratatui` as dependencies, and set up a basic `Tui` rendering with states matching the phases.
2. **Refine `x32_system_tune` to meet all `todo.md` requirements:**
   - Update `lib.rs` to loop through all `main_outputs`, test for `Pink Noise` or `Sine`, listen for operator confirmations.
   - We have implemented a state machine for the 4 phases and interactive TUI prompts.
   - Restored the integration test for `x32_system_tune` allowing us to verify the background client interaction and TDD process correctly.
3. **Run tests & clippy.**
   - All tests and clippy checks pass.
4. **Ensure `AGENTS.md` and `.jules/bolt.md` rules are followed** (like hoisting invariant string operations, and adding the Bolt performance PR format).
   - The TUI state rendering is simple and avoids excessive allocation.
5. **Perform pre-commit checks**
   - Ensure proper testing, verification, review, and reflection are done.
6. **Submit changes.**
