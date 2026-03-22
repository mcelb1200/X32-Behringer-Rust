with open("x32_midi2osc/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("let _ = commands_arc;", "let _ = commands;")
text = text.replace("let _ = debug;", "")
text = text.replace('use anyhow::{anyhow, Result};', '#[cfg(feature = "midi")]\nuse anyhow::anyhow;\nuse anyhow::Result;')
text = text.replace('use osc_lib::{OscArg, OscMessage};', '#[cfg(feature = "midi")]\nuse osc_lib::{OscArg, OscMessage};')
text = text.replace('fn process_osc_string', '#[cfg(feature = "midi")]\nfn process_osc_string')

with open("x32_midi2osc/src/main.rs", "w") as f:
    f.write(text)
