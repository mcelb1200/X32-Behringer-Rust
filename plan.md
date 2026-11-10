1. **Identify Performance Bottleneck**: `apps/x32_scene_checker/src/lib.rs` has `backup_content.push_str(&format!("{} {}\n", path, format_arg(val)));` in a loop.
2. **Analysis**: This uses `push_str(&format!(...))` which causes intermediate heap allocations as it loops.
3. **Proposed Change**: Replace with `writeln!(backup_content, "{} {}", path, format_arg(val)).unwrap();` or use the `write!` macro directly. Wait, `std::fmt::Write` trait is needed. We should import `std::fmt::Write`. Wait, in `std::io::Write` or `std::fmt::Write`. The file has `use std::io::{BufRead, Read, Write};` - but `backup_content` is a `String`, so it needs `std::fmt::Write`.
4. **Implement**:
```rust
<<<<<<< SEARCH
        let mut backup_content = String::new();
        for (path, val) in &current_map {
            backup_content.push_str(&format!("{} {}\n", path, format_arg(val)));
        }
=======
        let mut backup_content = String::new();
        for (path, val) in &current_map {
            use std::fmt::Write;
            let _ = writeln!(backup_content, "{} {}", path, format_arg(val));
        }
>>>>>>> REPLACE
```
5. **Verify**: Ensure the test passes by running `pnpm test` or equivalent.
