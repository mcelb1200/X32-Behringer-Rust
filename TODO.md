# Future Work Candidates

## x32_desk_restore
- The C version `X32DeskRestore.c` has ~1200 lines, while the Rust version is very basic (~110 lines).
- Missing logic: Needs the `XDS_parse` function to process specific node commands that can be "in any order and possibly incomplete without generating errors", enabling restoring of detailed settings from a file instead of simply relying on OSC message forwarding without validation.

## x32_reaper
- Missing logic: `X32Reaper.c` mentions several TODOs regarding Master track selection from Reaper and Solo/Mute functionality which need to be ported or solved.

## XAir Tools / Others
- Several original C tools lack a Rust counterpart: `XAirGetScene`, `XAirSetScene`, `XAir_Command`, `X32TapW`, `X32UDP`, `GetSceneName.c`, etc. Some are specific to the XAir mixer, but should be ported if the intention is a comprehensive rewrite.
