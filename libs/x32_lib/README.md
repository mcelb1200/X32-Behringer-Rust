# `x32_lib`

The primary domain logic library for Behringer X32, Midas M32, and XAir consoles. It provides a high-level DSL for command generation and handles console-specific parameter scaling.

## 🚀 Features

*   **Command Generators:** Programmatic generation of OSC paths for Channels, Auxins, Buses, DCAs, and FX.
*   **Scene Parsing:** Includes a robust parser for `.scn` and snippet files, supporting shorthand commands and complex scaling (e.g., Logarithmic Frequencies).
*   **Network Utils:** Helper functions for establishing verified UDP connections to consoles.

## 🛠️ Usage

### Scaling Parameters
```rust
use x32_lib::scaling::fader_to_db;

let db_value = fader_to_db(0.75); // Converts 0.0-1.0 to dB string
```

### Generating Commands
```rust
use x32_lib::command::channel::set_fader;

let (path, args) = set_fader(1, 0.8);
// path = "/ch/01/mix/fader", args = [OscArg::Float(0.8)]
```

## 🧪 Testing
Run domain logic tests with:
```bash
cargo test -p x32_lib
```
