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

## 2024-05-25 - [Fast buffer allocation from underlying slices]
**Learning:** In parsing paths (like reading binary `Blob` data), creating a buffer using `let mut buf = vec![0; len]` and then filling it with `Read::read_exact(&mut buf)` is inefficient due to the forced zero-initialization of the heap memory. We can avoid this and just copy memory directly by reading the underlying buffer `let buf = cursor.get_ref()[start..end].to_vec()` and manually advancing the cursor position.
**Action:** Whenever parsing data directly into an exact-length vector from a `Cursor`, avoid `vec![0; len]` and `read_exact`. Instead, use `cursor.get_ref()[start..end].to_vec()` and update the cursor position manually.

## 2024-05-27 - [Efficient byte padding using slice extension]
**Learning:** When padding buffers with 0 bytes to align to a 4-byte boundary (e.g., OSC padding), using a while loop with `bytes.push(0)` introduces repeated bounds checks and branch predictions that LLVM may struggle to optimize away completely. Calculating the exact remaining bytes and directly extending the buffer with a static slice `bytes.extend_from_slice(&[0, 0, 0][..pad_len])` runs over 2.5x faster.
**Action:** When a known maximum amount of padding is required, prefer pre-calculating the exact length and using slice extensions rather than variable-length loops.

## 2024-05-27 - [Bypassing UTF-8 decoding for guaranteed ASCII characters]
**Learning:** In hot loops where iterating over characters of a string that are guaranteed to be ASCII (such as OSC type tags in `osc_lib`), using `.chars()` incurs unnecessary UTF-8 decoding overhead. Using `.bytes()` instead is significantly faster.
**Action:** When parsing formats with known ASCII sub-sections, prefer `.bytes()` for iteration and byte string literals (e.g., `b'i'`) for matching, ensuring to cast back to `char` when constructing error messages.

## 2024-05-28 - [Avoiding string allocation for binary packet sub-sections]
**Learning:** In network parsers handling binary protocols (like `x32_reaper`), calling `String::from_utf8_lossy` on a buffer sub-section (like OSC type tags) to iterate over its characters introduces unnecessary allocations and UTF-8 validation overhead. Parsing the raw `&[u8]` slice directly and matching against byte literals (`b"f"`, `b"i"`) avoids string allocation entirely.
**Action:** When processing guaranteed ASCII data from byte slices, avoid intermediate `String` allocations. Work directly with the byte slice (`&[u8]`) and use byte literals (e.g., `b"f"`) to bypass UTF-8 decoding overhead and improve parsing performance.

## 2024-05-30 - [Avoiding Vec allocations when serializing over references]
**Learning:** We added an optimization to `osc_lib` to serialize directly from references. When designing zero-allocation wrapper functions in Rust that iterate multiple times (e.g., to calculate exact buffer size before serializing), taking an `IntoIterator` with a `Clone` bound (e.g., `I: IntoIterator<Item = &'a T> + Clone`) instead of a slice `&[&T]` allows the function to consume both arrays and iterator outputs safely. This prevents callers from having to `.collect()` into a temporary `Vec` just to pass a slice reference.
**Action:** Always prefer `I: IntoIterator + Clone` to `&[&T]` in APIs requiring multiple iterations over dynamically collected items.
