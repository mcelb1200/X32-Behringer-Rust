1. Modify `AppTui` struct in `apps/x32_feedback_detect/src/tui.rs` to cache the `notch_text` String buffer instead of reallocating it with `String::new()` on every `draw` call (which is part of the render loop).
2. Clear the cached `notch_text` string buffer before appending text to it in the `draw` method, effectively reusing the heap allocation.
3. Call pre commit tools.
4. Submit the PR.
