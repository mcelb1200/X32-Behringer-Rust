# TASK: x32-cli-migration

## Objective
Consolidate all remaining binary targets in the workspace into the single unified `x32-cli` binary. Standalone binaries must be preserved throughout.

## Guidelines
- Extract `Args` and the main logic from `src/main.rs` of each target into `src/lib.rs`.
- Do not remove the standalone target. Cargo automatically compiles a binary target if `src/main.rs` is present alongside `src/lib.rs`.
- Thin out `src/main.rs` to only parse arguments and invoke `run(args).await` or equivalent from `lib.rs`.
- Add each migrated crate as a path dependency in `tools/x32_cli/Cargo.toml`.
- Register each tool as a subcommand in `tools/x32_cli/src/main.rs` using the `clap` parser.
- Keep execution cross-platform. No symlinks.
- Run `cargo test` and verify that the build succeeds and tests pass after each migration step.

## Scope of Migration
The following crates must be refactored:

### Phase 1: Simple Single-File Tools
1. `x32_udp`
2. `xair_set_scene`
3. `xair_get_scene`
4. `x32_set_scene`
5. `x32_get_scene`
6. `x32_get_scene_name`
7. `x32_cpxlivemarkers`
8. `x32_tcp`
9. `x32_geq2_cpy`
10. `x32_jog4xlive`
11. `x32_dump`
12. `x32_xlive_wav`
13. `xair_command`
14. `x32_copy_fx` (has local module `fx_defaults.rs` - move that logic/module into library as well)
15. `x32_get_lib`

### Phase 2: Multi-Module Tools
1. `x32_desk_restore` (module `parse.rs` - move to library)
2. `x32_desk_save` (module `nodes.rs` - move to library)
3. `x32_ssavergw` (module `state.rs` - move to library)
4. `x32_set_lib`
5. `x32_punch_control` (modules `config.rs`, `format.rs`, `state.rs` - move to library)
6. `x32_set_preset`
7. `x32_replay` (delete `src/main.rs.orig` leftover file)
8. `x32_fade`

### Phase 3: Complex Tools
1. `x32_custom_layer`
2. `x32_wav_xlive` (hound, byteorder, chrono)
3. `x32_commander` (rpn.rs, midir dep)
4. `x32_loudness` (serde_json config)

### Phase 4: Apps & Heavy Tools
1. `x32_automix`
2. `x32_sync` (ensure you fix any non-workspace dependencies)
3. `x32_tap`
4. `x32_tapw` (app.rs)
5. `x32_autobeat` (multiple subfolders: detection/, effects/)
6. `x32_reaper` (modules: config.rs, reaper_handler.rs, state.rs, x32_handler.rs)
7. `x32_vocal_ducking`
8. `x32_usb`
9. `x32_midi2osc` (config.rs, rpn.rs, rpn_calc.rs)
10. `x32_emulator` (already has `lib.rs`, just register subcommand)

## Final Tasks
- Delete `tools/x32_replay/src/main.rs.orig` if not already done.
- Update `docs/ROADMAP.md` to show that Phase 4 "Unified x32-cli" is complete.
- Verify `cargo build --release` produces the correct set of targets.
