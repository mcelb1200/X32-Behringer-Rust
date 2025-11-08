# X32 Tools - Rust Rewrite

This project is a complete rewrite of the X32 command-line tools in Rust, focusing on simplicity, stability, and reliability. The goal is to provide a modern, high-quality, and easy-to-maintain suite of tools for controlling Behringer X32 and X-Air digital mixers.

## Original Author

This project is a fork and rewrite of the original X32 libraries. All credit for the original design, functionality, and years of work goes to the original author:

*   **Author:** Patrick-Gilles Maillot
*   **Original Work:** [https://sites.google.com/site/patrickmaillot/x32](https://sites.google.com/site/patrickmaillot/x32)

The link above is to the original author's website and is not affiliated with this fork.

## License

This Rust rewrite is licensed under the **GNU General Public License v3.0**. A full copy of the license is available in the `LICENSE` file in this directory.

## Current Status

This project is under active development. The following tools have been rewritten in Rust and are available in this workspace:

*   `x32_get_scene`: Retrieves and prints the current scene number and name from the mixer.
*   `x32_set_scene`: Loads a specific scene number on the mixer.
*   `x32_desk_restore`: Restores the mixer's state from a file containing OSC commands.

## Building and Running the Tools

This project is structured as a Rust workspace. To build all the tools, you will need to have a recent version of the Rust toolchain installed.

1.  **Clone the repository.**
2.  **Navigate to the `rust_rewrite` directory.**
3.  **Build the entire workspace:**

    ```bash
    cargo build --release
    ```

The compiled binaries will be located in the `rust_rewrite/target/release/` directory.

## Tools

This section provides details on the function and usage of each tool.

### `x32_desk_restore`

Restores the state of an X32 mixer from a file containing OSC commands. This tool is useful for loading a previously saved mixer state, scene, or snippet.

**Function:**

The tool reads a specified file line by line. Each line is interpreted as an OSC command and sent to the mixer. Lines starting with `#` are treated as comments and ignored.

**Arguments:**

*   `-i, --ip <IP>`: The IP address of the X32 mixer. Defaults to `192.168.1.64`.
*   `<FILE>`: The path to the file containing the OSC commands to be sent to the mixer.

**Example Usage:**

To restore the mixer state from a file named `myscene.scn` on a mixer at IP address `192.168.1.32`:

```bash
./rust_rewrite/target/release/x32_desk_restore -i 192.168.1.32 /path/to/myscene.scn
```

### `x32_get_scene`

Retrieves and prints the current scene number and name from the mixer.

**Arguments:**

*   `-i, --ip <IP>`: The IP address of the X32 mixer. Defaults to `192.168.1.64`.

**Example Usage:**

```bash
./rust_rewrite/target/release/x32_get_scene -i 192.168.1.32
```

### `x32_set_scene`

Loads a specific scene on the mixer.

**Arguments:**

*   `-i, --ip <IP>`: The IP address of the X32 mixer. Defaults to `192.168.1.64`.
*   `<SCENE>`: The scene number to load (0-99).

**Example Usage:**

To load scene number 5 on the mixer:

```bash
./rust_rewrite/target/release/x32_set_scene -i 192.168.1.32 5
```
