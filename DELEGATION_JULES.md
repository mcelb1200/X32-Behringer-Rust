# Task Delegation: Comprehensive MixerClient Refactor

## 🎯 Objective
Complete the migration of all remaining binary crates in `apps/` and `tools/` to the new `MixerClient` abstraction located in `libs/x32_lib/src/client.rs`. This refactor is essential to eliminate socket-handling boilerplate, centralize security bounding (Sentinel), and ensure performance optimizations (Bolt) are applied globally.

## 📐 Current State
- **Branch**: `refactor/mixer-client-abstraction`
- **Infrastructure**: `MixerClient` is implemented in `libs/x32_lib` and exported via `lib.rs`.
- **Completed**: `x32_command`, `x32_get_scene`, and `x32_commander` have already been migrated to use `MixerClient`.
- **Workspace**: Root `Cargo.toml` uses workspace inheritance for all shared dependencies.

## 🛠️ Instructions for Jules AI
1. **Iterative Migration**: For each crate in `apps/` and `tools/` (excluding those already finished):
   - Replace manual `UdpSocket` creation and `osc_lib` serialization boilerplate with `MixerClient` methods.
   - Use `client.send_message(path, args)` for sets and `client.query_value(path)` for queries.
   - For real-time tools, utilize `client.subscribe()` to handle asynchronous updates from the mixer.
   - Update the crate's `Cargo.toml` to remove redundant local dependencies and use `workspace = true` where applicable.
2. **Security Compliance**: Ensure all file interactions within these crates use the `Sentinel` bounding logic (already integrated into the library's client module).
3. **Performance Optimization**: Ensure `Bolt` optimized serialization is leveraged by defaulting to the client's internal buffering.
4. **Validation**: Run `cargo test --workspace` frequently to ensure no regressions in tool behavior or integration tests with the `x32_core` emulator.
5. **Conclusion**: When the refactor is complete and verified, **move this `DELEGATION_JULES.md` file to the `docs/archive/` directory** and commit the change.

## 🛡️ Constraints
- **Zero Interface Change**: Do not modify the CLI arguments or external behavior of the tools.
- **Async Priority**: Prefer the `MixerClient`'s async methods. Convert synchronous tools to `tokio` runtimes if they aren't already.
- **Clean Code**: Adhere strictly to the workspace's linting rules (no unsafe code, minimal unwraps).

---
**Status**: IN_PROGRESS
**Owner**: Jules AI
