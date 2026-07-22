1. **Refactor `apps/x32_volunteer/src/ui.rs` to cache layout constraints**:
   - Add `cached_constraints: Vec<Constraint>` and `cached_channels_len: usize` to the `Tui` struct in `apps/x32_volunteer/src/ui.rs`.
   - Update `Tui::new()` to initialize these fields.
   - In `Tui::draw()`, check if `state.channels.len() != self.cached_channels_len`. If so, regenerate the `cached_constraints` in `self` and update `cached_channels_len`.
   - Pass `&self.cached_constraints` to `.constraints()` instead of dynamically mapping and collecting, preventing per-frame heap allocations.
2. **Verify functionality**:
   - Run `cargo check --manifest-path apps/x32_volunteer/Cargo.toml` and `cargo test --manifest-path apps/x32_volunteer/Cargo.toml` using `run_in_bash_session`.
3. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done**.
4. **Submit the PR**:
   - Use the `submit` tool to commit the change to a new branch, using the Bolt persona PR format "⚡ Bolt: Cache layout constraints to prevent per-frame heap allocations".
