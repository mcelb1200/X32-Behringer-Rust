1.  **Create the `x32_auto_ringout` App crate:**
    *   Initialize the crate in `apps/x32_auto_ringout`.
    *   Set up `Cargo.toml` with dependencies: `x32_lib`, `osc_lib`, `tokio`, `anyhow`, `clap`, `ratatui`, `crossterm`.
    *   Create a library entry point `src/lib.rs` and standalone wrapper `src/main.rs`.

2.  **Implement the Core Logic in `src/lib.rs`:**
    *   Parse CLI arguments (`ip`, `buses`, `target_dbfs`, `max_notches`).
    *   Implement TUI setup/teardown using `crossterm` and `ratatui`.
    *   Set up the main application loop responding to key events and OSC messages.
    *   Define an `AppState` to track buses (armed, disarmed, active), current levels, and identified notches.
    *   Implement the auto-ringout sequence:
        1.  Gradually raise bus levels.
        2.  Identify feedback (simulated via high sustained signal levels for now, or using a rudimentary peak detection on `/meters` if feasible without FFT, although full FFT is complex for a TUI without a dedicated audio stream. The spec says "Listen for feedback onset using the Automatic Feedback Detection function... Fallback: meter data from `/meters` OSC subscriptions"). We will implement a peak detection over meter blocks.
        3.  Apply notch filters on the corresponding `/bus/XX/eq/N/*` OSC paths.
        4.  Cap out at `max_notches` per bus.
    *   Integrate with the Dugan automixer if speech buses are configured.
    *   Maintain "gain before feedback" headroom.

3.  **Integrate `x32_auto_ringout` into the `x32-cli` Monolith:**
    *   Add `x32_auto_ringout = { path = "../../apps/x32_auto_ringout" }` to `tools/x32_cli/Cargo.toml`.
    *   Add the `X32AutoRingout(x32_auto_ringout::Args)` subcommand in `tools/x32_cli/src/main.rs`.
    *   Wire up the execution logic in the `main` match block.

4.  **Write Tests:**
    *   Add integration tests in `apps/x32_auto_ringout/tests/integration_test.rs` using a mock UDP server to verify EQ notch applications.

5.  **Pre-Commit Check:**
    *   Run `cargo test`, `cargo fmt`, and `cargo clippy`.
    *   Ensure all project standards and `.jules/x32_cli.md` constraints are met.
