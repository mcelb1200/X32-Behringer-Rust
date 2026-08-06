1. **Refactor `print_report_summary` in `apps/x32_scene_checker/src/lib.rs`**
   - The current implementation iterates over `issues` 5 separate times using `.filter(..).collect::<Vec<_>>()`, which causes 5 O(N) heap allocations.
   - Refactor it to use a single pass over `issues`, keeping vectors only for the elements we need to show details for (`criticals` and `highs`), and just integer counters for the rest, or just `Vec<&RiskIssue>` for all but collected in one pass without redundant filtering. Actually, we need details for criticals and highs (up to 3 items and the count). We only need the counts for moderates, lows, and infos.
   - We will replace the 5 `collect::<Vec<_>>()` calls with a single loop that bins the references into two `Vec<&RiskIssue>` for `criticals` and `highs`, and counters for `moderates`, `lows`, and `infos`.
2. **Update `print_full_details` to use `issues.to_vec()` or similar, avoiding `.collect::<Vec<_>>()` if possible.** Actually `issues.iter().collect::<Vec<_>>()` is ok for an array of references, but we can do `let mut sorted_issues: Vec<&RiskIssue> = issues.iter().collect();` which is just what it does. The main optimization is in `print_report_summary`.
3. **Run `pnpm test` and formatting checks.**
4. **Document critical learnings in `.jules/bolt.md`.**
5. **Pre-commit verification.**
6. **Submit PR.**
