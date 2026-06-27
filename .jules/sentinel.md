## 2024-06-27 - Integer Overflow in OSC Bundle Size Parsing
**Vulnerability:** A Denial of Service (DoS) vulnerability existed in `x32_reaper` where negative integer values sent as the size parameter in OSC bundles were parsed as unsigned 32-bit integers (`u32`) and cast to `usize`. On 64-bit systems, this bypassed length validation due to the size wrapping to a huge positive number, resulting in an out-of-bounds slice read panic.
**Learning:** OSC explicitly defines size prefixes in bundles as signed 32-bit integers (`int32`), but they were mistakenly parsed as unsigned integers. This is a common network packet parsing trap in Rust.
**Prevention:** Always parse size bounds as `i32` when required by the protocol, and explicitly check if `size < 0` to reject maliciously crafted negative offsets before casting to `usize`.

## 2024-06-27 - Integer Overflow in OSC Bundle Size Parsing
**Vulnerability:** A Denial of Service (DoS) vulnerability existed in `x32_reaper` where negative integer values sent as the size parameter in OSC bundles were parsed as unsigned 32-bit integers (`u32`) and cast to `usize`. On 64-bit systems, this bypassed length validation due to the size wrapping to a huge positive number, resulting in an out-of-bounds slice read panic.
**Learning:** OSC explicitly defines size prefixes in bundles as signed 32-bit integers (`int32`), but they were mistakenly parsed as unsigned integers. This is a common network packet parsing trap in Rust.
**Prevention:** Always parse size bounds as `i32` when required by the protocol, and explicitly check if `size < 0` to reject maliciously crafted negative offsets before casting to `usize`.

## 2024-06-27 - Integer Overflow in OSC Bundle Size Parsing
**Vulnerability:** A Denial of Service (DoS) vulnerability existed in `x32_reaper` where negative integer values sent as the size parameter in OSC bundles were parsed as unsigned 32-bit integers (`u32`) and cast to `usize`. On 64-bit systems, this bypassed length validation due to the size wrapping to a huge positive number, resulting in an out-of-bounds slice read panic.
**Learning:** OSC explicitly defines size prefixes in bundles as signed 32-bit integers (`int32`), but they were mistakenly parsed as unsigned integers. This is a common network packet parsing trap in Rust.
**Prevention:** Always parse size bounds as `i32` when required by the protocol, and explicitly check if `size < 0` to reject maliciously crafted negative offsets before casting to `usize`.
