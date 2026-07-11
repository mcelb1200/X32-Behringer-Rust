1.  **Analyze `AppState` string joining in `x32_autobeat` loop:**
    *   Currently, inside the `last_ui_update.elapsed() > Duration::from_millis(50)` loop block, `AppState::source` is generated via:
        ```rust
        source: sources
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ")
        ```
    *   This is evaluated every ~50ms and allocates multiple `String` instances and a `Vec`, then joins them into another `String`, doing O(N) heap allocations.
    *   However, `sources` is initialized *before* the main event loop and is completely static during the loop.

2.  **Move computation outside the loop:**
    *   We will compute the `sources_str` once before the `loop { ... }`.
    *   Inside the loop, we simply `clone()` this pre-computed string: `source: sources_str.clone(),`.

3.  **Ensure compliance with PR formatting and tests:**
    *   We'll use `cargo check` and `cargo test --workspace` to ensure things pass.
    *   We'll run `cargo clippy --workspace -- -D warnings`.
    *   We'll document the performance win in `.jules/bolt.md` based on the system memory directives.
    *   We'll format PR as `⚡ Bolt: [performance improvement]`.

4.  **Perform pre-commit checks**
    *   Ensure proper testing, verification, review, and reflection are done.

5.  **Submit changes.**
