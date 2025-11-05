# X32 Emulator

This project is a Rust rewrite of the X32 Emulator originally created by Patrick-Gilles Maillot. The original project can be found at https://sites.google.com/site/patrickmaillot/x32.

## `cfg_main.rs` Module

The `cfg_main.rs` module provides a Rust implementation of the OSC command definitions found in the C header file `X32CfgMain.h`. This module defines the data structures and constants needed to represent and interact with the X32's configuration and main commands in a safe and idiomatic way.

### Data Structures

- **`CommandType`**: An enum representing the different types of OSC commands.
- **`CommandValue`**: An enum representing the value of a command, which can be an integer, float, string, or blob of data.
- **`Format`**: An enum that can be either a `CommandType` or a `String`.
- **`Command`**: A struct that represents a single OSC command, containing its path, format, flags, value, and an optional node.

### Constants

- **`XCONFIG`**: A constant array of `Command` structs representing the `/config` commands.
- **`XMAIN`**: A constant array of `Command` structs representing the `/main` commands.

### Example Usage

Here is an example of how to access and print the commands from the `XCONFIG` and `XMAIN` arrays:

```rust
use x32_lib::cfg_main::{XCONFIG, XMAIN};

fn main() {
    println!("XCONFIG Commands:");
    for command in XCONFIG.iter() {
        println!("  {}", command.command);
    }

    println!("\nXMAIN Commands:");
    for command in XMAIN.iter() {
        println!("  {}", command.command);
    }
}
```
