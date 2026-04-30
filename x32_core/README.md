# `x32_core`

A headless, stateful emulator engine for Behringer X32 and Midas M32 mixers. It serves as the foundation for the `x32_emulator` utility and the project's integration test suite.

## 🚀 Features

*   **Stateful Emulation:** Tracks the real-time value of every console parameter via a thread-safe state store.
*   **`/xremote` Broadcasting:** Implements full client tracking. When a parameter is "set", the change is automatically broadcasted to all registered subscribers.
*   **System Admin Support:** Handles `/copy`, `/load`, `/save`, and `/add` commands to simulate complete console workflows.

## 🛠️ Usage (Integration Testing)

```rust
use x32_core::Mixer;
use std::net::SocketAddr;

let mut mixer = Mixer::new();
let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

// Dispatch a raw OSC packet
let responses = mixer.dispatch(&osc_bytes, remote_addr).unwrap();

for (target, packet) in responses {
    // target = remote_addr for queries
    // target = all subscribers for broadcasts
}
```

## 🧪 Testing
Run emulator core tests with:
```bash
cargo test -p x32_core
```
