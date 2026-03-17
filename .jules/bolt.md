## 2024-05-23 - [Pre-calculating Vec capacity and avoiding intermediate strings in OSC serialization]
**Learning:** In hot loops like serialization, calculating the exact size and allocating once (`Vec::with_capacity`) is significantly faster than relying on `Vec`'s automatic growth. Also, avoiding intermediate allocations (like building a `String` for type tags) further reduces overhead.
**Action:** Always look for opportunities to pre-calculate buffer sizes when serialization logic is deterministic. Avoid creating temporary heap-allocated structures (Strings, Vecs) inside serialization methods if the data can be written directly to the output buffer.

## 2026-03-15 - [Efficient parsing of null-terminated strings from binary cursors]
**Learning:** In hot loops where binary streams or cursors are parsed into null-terminated `String` values, repeatedly calling `read_u8()` in a loop and pushing to a dynamic `Vec` incurs severe functional overhead and unnecessary allocations. Operating directly on the underlying buffer slice `cursor.get_ref()[pos..]`, finding the null byte using `slice.iter().position(|&b| b == 0)`, and decoding using `String::from_utf8(string_bytes.to_vec())` yields a 4x+ performance improvement.
**Action:** Always prefer slice manipulations and built-in search functions when dealing with contiguous byte arrays `Cursor<&[u8]>` in parsers, rather than byte-by-byte looping.

## 2024-05-23 - [Optimizing formatting through `Write` and `Display`]
**Learning:** Using macro `write!(f, "...")` to append static string literals through the `Write` trait in `fmt::Display` implementations carries unnecessary formatting overhead in Rust.
**Action:** Always replace `write!(f, "literal")` with `f.write_str("literal")` for significant performance improvements in serialization or stringification paths.

## 2024-05-23 - [Avoiding generic Write trait abstractions in hot-paths]
**Learning:** In hot loops like serialization, appending to a `Vec` via `Write` trait methods (e.g., `write_u8` or `write_all`) is slower than using native `Vec` methods (`push`, `extend_from_slice`) due to trait abstraction overhead.
**Action:** In serialization logic where writing directly to an underlying `Vec<u8>` is possible (without the generic `Write` abstraction), always prefer `Vec::push` and `Vec::extend_from_slice` over `write_u8` and `write_all`.
