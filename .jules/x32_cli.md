# Unified `x32-cli` Architecture and Constraints

This document provides crucial guidance for AI agents and automated code generators working within the `x32-rust` repository. It outlines the mandatory architectural pattern for adding or modifying tools.

## The `x32-cli` Monolith Binary

The `x32-rust` project employs a unified monolithic binary architecture (`tools/x32_cli`). This approach is designed to drastically reduce compilation times and final binary footprint by statically linking shared libraries (`tokio`, `clap`, `osc_lib`) into a single executable, rather than duplicating them across 30+ separate binaries.

### Core Constraint: Subcommand Registration
**All new tools, applications, and utilities MUST be integrated as subcommands of the `x32-cli` binary.**

You must never create a new tool that *only* exists as an isolated binary target.

### Implementation Rules

When creating a new tool (e.g., `x32_new_tool`), you must follow this dual-target pattern:

1.  **Library Core (`src/lib.rs`)**:
    *   The core logic and the `Args` (or `Cli`) struct for command-line parsing must reside in `src/lib.rs`.
    *   Export a primary entry point, typically an asynchronous `run` function: `pub async fn run(args: Args) -> Result<()>`.

2.  **Standalone Wrapper (`src/main.rs`)**:
    *   You **must** preserve a thin standalone wrapper. Cargo will automatically compile this if `src/main.rs` is present alongside `src/lib.rs`.
    *   This file should simply parse arguments and invoke the library's `run` function:
        ```rust
        // src/main.rs
        use clap::Parser;
        use x32_new_tool::{run, Args};

        #[tokio::main]
        async fn main() -> anyhow::Result<()> {
            let args = Args::parse();
            run(args).await
        }
        ```
    *   *Reasoning:* Standalone binaries are preserved for isolated testing, specialized Docker containers, and legacy scripts that have not yet migrated to the unified CLI syntax.

3.  **`x32-cli` Integration**:
    *   Add your new crate as a local path dependency in `tools/x32_cli/Cargo.toml`.
    *   In `tools/x32_cli/src/main.rs`, register the tool as a new variant in the `Commands` enum using `clap::Subcommand`.
    *   Add the corresponding `match` arm in the `main` function of `tools/x32_cli/src/main.rs` to route execution to your tool's `run` function.

### Example Integration

**`tools/x32_cli/src/main.rs` modifications:**
```rust
#[derive(Subcommand, Debug)]
enum Commands {
    // ... existing commands
    /// Description of the new tool
    X32NewTool(x32_new_tool::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        // ... existing arms
        Commands::X32NewTool(args) => x32_new_tool::run(args).await.map_err(anyhow::Error::msg),
    }
}
```

By adhering strictly to this pattern, you ensure the project remains coherent, compile times remain manageable, and the unified `x32-cli` experience remains comprehensive.
