# `osc_lib`

A foundational Rust library for high-speed, zero-allocation-ready Open Sound Control (OSC) packet processing.

## 🚀 Features

*   **Zero-Alloc Reading:** Leverages byte slices and smart pointer logic to minimize heap allocations during packet parsing.
*   **Bolt Optimized:** Serialization paths are optimized for low-latency audio control environments.
*   **Robust Tokenization:** Correctly handles quoted strings and binary blobs in OSC paths.

## 🛠️ Usage

### Creating a Message
```rust
use osc_lib::{OscMessage, OscArg};

let msg = OscMessage::new(
    "/ch/01/mix/fader".to_string(),
    vec![OscArg::Float(0.75)]
);
let bytes = msg.to_bytes().unwrap();
```

### Parsing a Message
```rust
let msg = OscMessage::from_bytes(&bytes).unwrap();
println!("Path: {}, Arg: {:?}", msg.path, msg.args[0]);
```

## 🧪 Testing
Run unit tests with:
```bash
cargo test -p osc_lib
```
