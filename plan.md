1. **Fix `libs/osc_lib/src/lib.rs` vulnerability**
   - The `from_bytes` function parses `b` (blob) arguments by reading an `i32` length, casting it to `usize`, and then doing `current_pos + len`.
   - Since `i32` can be negative, an attacker can supply a negative number (e.g. `-1`). When cast to `usize`, this becomes a very large number (e.g. `18446744073709551615`).
   - The addition `current_pos + len` then panics with "attempt to add with overflow", crashing the server (DoS).
   - Fix: check `if len < 0 { return Err(...); }` before casting to `usize`, similar to what's done in `apps/x32_reaper/src/reaper_handler.rs`.

2. **Review other `read_i32` and similar parsing functions**
   - Wait, `cursor.read_i32` is used in a few places in `tools/x32_dump/src/lib.rs` for blobs, but those are internal tools, so maybe not critical, but let's check if there's any panic potential. Actually, `cursor.read_i32::<BigEndian>()` is also used for the `i` (int) argument in `osc_lib`, but that's just pushed to args, no length calculations, so it's fine.

3. **Pre-commit checks**
   - Complete pre-commit checks as requested.
