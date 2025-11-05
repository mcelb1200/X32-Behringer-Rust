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

*   `x32_command`: A general-purpose tool for sending OSC commands to the mixer.
*   `x32_geq2cpy`: A utility for copying graphic EQ settings between FX slots.
*   `x32_tap`: An interactive tool for setting the tempo of a delay effect by tapping.
*   `get_scene_name`: A tool to monitor for scene changes and print the name of the new scene.

## Building and a Running the Tools

This project is structured as a Rust workspace. To build all the tools, you will need to have a recent version of the Rust toolchain installed.

1.  **Clone the repository.**
2.  **Navigate to the `rust_rewrite` directory.**
3.  **Build the entire workspace:**

    ```bash
    cargo build --release
    ```

The compiled binaries will be located in the `rust_rewrite/target/release/` directory. You can run any of the tools from there. For example:

```bash
./rust_rewrite/target/release/x32_command -i 192.168.1.10 "/info"
```
