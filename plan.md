1. **Optimize string splitting in `apps/x32_autobeat/src/lib.rs`**
   - In `parse_channels` and `parse_slots`, replace `.split('-').collect::<Vec<&str>>()` with `.split_once('-')` to avoid unnecessary heap allocations when parsing ranges.
   - This directly aligns with Bolt's `.jules/bolt.md` teachings on eliminating O(N) allocations for string parsing by avoiding `collect::<Vec<&str>>()`.
2. **Add a journal entry to `.jules/bolt.md`**
   - Record the learning if it's not already covered in exactly this context, though it looks like it is covered under `,` splitting. We don't strictly need a new entry if the rule already exists, but I will make sure I add comments explaining the optimization. Actually I will verify if I need to add one. "Avoid using `.split('-').collect::<Vec<&str>>()` for parsing ranges. Use `.split_once('-')` to avoid heap allocations."
3. **Verify the change**
   - Run `cargo fmt`, `cargo clippy`, and `cargo test`.
4. **Pre-commit step**
   - Run `pre_commit_instructions` tool to ensure all required checks are done.
5. **Submit the change**
   - PR Title: `⚡ Bolt: [performance improvement] Eliminate heap allocation in range parsing`
   - Description containing What, Why, Impact, and Measurement.
