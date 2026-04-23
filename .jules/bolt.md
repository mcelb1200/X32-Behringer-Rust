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

## 2024-05-31 - [Avoiding `write!` macro for string concatenation]
**Learning:** Using the `write!(f, "{}", s)` macro to append a dynamic string variable to a formatter or a `String` buffer incurs the overhead of `std::fmt` machinery. It is significantly faster to directly use string manipulation methods like `f.write_str(s)` or `buffer.push_str(s)`.
**Action:** Always replace `write!(f, "{}", s)` with `f.write_str(s)` (for formatters) or `s.push_str(val)` (for Strings) when appending string variables. Similarly, avoid `write!(&mut s, "\"{}\"", val)` by using `s.push('"'); s.push_str(val); s.push('"');`.
## 2024-06-05 - [Avoiding std::fmt machinery in hot loops]
**Learning:** The `write!` macro invokes the `std::fmt` machinery, which involves hidden parsing and allocation overhead even for simple strings or values. In hot loops or `fmt::Display` implementations, relying on `write!(f, "{}", val)` or `write!(f, "literal")` causes measurable degradation compared to direct manipulation.
**Action:** To eliminate formatting machinery overhead in Rust hot loops, replace the `write!` macro (e.g., `write!(f, "{}", val)` or `write!(&mut s, "\"{}\"", val)`) with direct string manipulation methods like `f.write_str(val)`, `s.push()`, `s.push_str()`, or manual static character array mappings for hex values.
## 2026-04-14 - [Handling Error Silencing with BufWriter]
**Learning:** When using `std::io::BufWriter` in Rust to optimize file I/O, relying on its `Drop` implementation to flush the buffer causes any underlying I/O errors during that final flush to be silently ignored.
**Action:** Always explicitly call `flush()?` (or `.into_inner()?`) before the `BufWriter` goes out of scope to ensure any write errors are properly caught and bubbled up.
## 2024-06-10 - [Avoiding string parsing overhead for purely hex data]
**Learning:** In performance-critical hot loops (e.g., parsing OSC blobs), using `u8::from_str_radix` on string slices to parse hexadecimal data incurs measurable overhead due to slice creation, UTF-8 checks, and generic parsing machinery. Replacing this with a manual loop that matches on raw ASCII bytes (`b'0'..=b'9'`, `b'a'..=b'f'`, `b'A'..=b'F'`) and uses bitwise operations (`(high << 4) | low`) significantly improves execution time for purely hex data parsing.
**Action:** When parsing purely hex strings into bytes in hot paths, avoid `u8::from_str_radix`. Work directly with byte slices and map ASCII characters to values using simple arithmetic and bitwise combinations.

## 2024-05-18 - [Missing Async BufWriter Flush]
**Learning:** In asynchronous Rust using Tokio, `tokio::io::BufWriter` cannot be flushed within its `Drop` implementation because dropping is a synchronous operation. Thus, merely letting the `BufWriter` fall out of scope (e.g., by reassigning `file_writer = None`) will silently discard any buffered data that has not been flushed to disk.
**Action:** Always extract the inner `BufWriter` (e.g., using `Option::take()`) and explicitly await `.flush().await` before discarding the writer, especially during state transitions or exit paths.

## 2024-06-15 - [Efficient broadcasting of shared data using Arc]
**Learning:** When broadcasting the same payload (e.g. serialized OSC responses) to multiple clients, cloning a `Vec<u8>` for each recipient incurs unnecessary `O(N)` heap allocations and memory copying. Replacing `Vec<u8>` with `std::sync::Arc<[u8]>` allows cheap pointer cloning and zero-copy sharing across all outgoing messages.
**Action:** When returning multiple identical byte buffers intended for different network addresses, use `Arc<[u8]>` instead of `Vec<u8>` to eliminate redundant allocations in the broadcast loop.
