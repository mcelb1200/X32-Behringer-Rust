# TODO

- Extract 'todo' items from reference C files in `c_origin/` and document them here for future work.
  - Evaluate `// TODO: How to select Master from REAPER???` from `X32Reaper.c`
  - Evaluate `// TODO: do a proper implementation of the function` for `/add` and `/load` from `X32.c`
  - Evaluate `// TODO: implement 'libfx' and 'librout'` from `X32.c`
  - Evaluate `//char* r_CS[] = { "INT", "AES50A", "AES50B", "CARD" };	//Todo: Validate "CARD"` from `X32DeskRestore.c`
  - Evaluate `//// todo: validate if the use of the "/~~~" command would work here` from `X32DeskRestore.c`
  - Evaluate `////todo: one or 80 lines?` from `X32DeskRestore.c`
  - Evaluate `////todo: ... Last one is a string, unknown for now... could it correspond to iQ/group???` from `X32DeskRestore.c`
  - Evaluate `////todo: Decay sent as a logf or as a float with 2 digits after decimal point?` from `X32DeskRestore.c`
  - Evaluate `// the same way as REW clicks but does not behave similarly... TODO therefore` from `X32PunchControl.c`
  - Evaluate `//todo` from `X32Commander.c`
  - Evaluate `// Effects defaults. The trailing zeroes are not necessary - No big deal but todo: remove them!` from `X32CopyFX.c`
  - Evaluate `// TODO: 1 if any solosw is still on, 0 otherwise` from `X32.c`
  - Evaluate `s_len = Xsprint(s_buf, s_len, 's', ",b"); //todo - incorrect` from `X32.c`
  - Evaluate `s_adr += ((strlen(s_adr) + 4) & ~3); //todo - incorrect` from `X32.c`
  - Evaluate `return 0; // no room for new clients! (todo; another return status?)` from `X32.c`
  - Evaluate `// Todo?` from `X32.c`
  - Evaluate `strcpy(s_buf + s_len, " TODO");` from `X32.c`
  - Evaluate `XClientMeters[i] = *Client_ip_pt;			// remember requesting IP client TODO: not the right approach` from `X32.c`
  - Evaluate `// Ignored for now / Todo` (in `function_renew`) from `X32.c`
  - Evaluate `// todo: this is not actually done here!` from `X32.c`
  - Evaluate `// TODO At this time the last two <string><int> parameters are ignored` from `X32.c`

## Unported C Utilities
The following original C utilities have not been explicitly ported into standalone Rust binaries:
- `Testparams` (testing/utility logic in `X32lib`)
- `X32Connect` (testing/utility logic in `X32lib`)
- `X32RpnCalc` (testing/utility logic in `X32lib`)
- `X32logf` (testing/utility logic in `X32lib`)
- `Xcparse` (testing/utility logic in `X32lib`)
- `Xdump` (testing/utility logic in `X32lib`)
- `Xscene2X` (testing/utility logic in `X32lib`)
- `Xsprint` (testing/utility logic in `X32lib`)
- `validateIP4Dotted` (testing/utility logic in `X32lib`)

*Note:* The auxiliary parsing files (`SetSceneParse`, `XAirSetSceneParse`, `XAir_fxparse`, `XAir_fxparse1`, `fxparse`, `fxparse1`, `fxparse5`) have had their logic integrated into `x32_lib::scene_parse`.

## Context
These TODOs are legacy markers from the original C codebase that was ported to Rust. Future agents should review these items to verify whether the ported Rust components (like `x32_reaper` and `x32_core`) fully implement the intended functionality or if additional logic is needed to resolve the original author's comments.


## Missing Test/Utility Functions
The following C files from `X32lib/` have not been directly ported into isolated Rust structures, although some of their logic might be replicated in `osc_lib` and `x32_lib`:
- `Testparams.c`
- `X32Connect.c`
- `X32RpnCalc.c` (Partially addressed by `x32_commander/src/rpn.rs`)
- `X32logf.c`
- `Xcparse.c`
- `Xdump.c`
- `Xscene2X.c` (Partially addressed by `x32_lib::scene_parse`)
- `Xsprint.c`
- `validateIP4Dotted.c`
