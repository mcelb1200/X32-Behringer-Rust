//! `x32_testparams` provides a library for parsing INI-style test parameter files.
//!
//! This is a Rust port of `Testparams.c` from the original C codebase. It supports parsing
//! lines in the format `key=value` into various strongly-typed parameter values.

use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Represents the type of a parameter to parse.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamType {
    /// A single character (e.g., `key=A`)
    Char,
    /// An integer (e.g., `key=10`)
    Int,
    /// A floating-point number (e.g., `key=10.23`)
    Float,
    /// A string (e.g., `key=hello world`)
    String,
    /// A sequence of hex bytes separated by spaces (e.g., `key=f0 f1 f2`)
    HexString,
    /// A yes/no boolean (e.g., `key=Y` or `key=no`)
    YesNo,
}

/// Holds the parsed value of a parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    /// A single character value
    Char(char),
    /// An integer value
    Int(i32),
    /// A floating-point value
    Float(f32),
    /// A string value
    String(String),
    /// A vector of hex bytes
    HexString(Vec<u8>),
    /// A boolean value representing Yes or No
    YesNo(bool),
}

/// Defines a parameter to be parsed, including its expected name and type,
/// and eventually holding its parsed value.
#[derive(Debug, Clone)]
pub struct ParamDefinition {
    /// The expected key name for this parameter
    pub name: String,
    /// The expected type for this parameter
    pub param_type: ParamType,
    /// The parsed value, if successfully read
    pub value: Option<ParamValue>,
}

impl ParamDefinition {
    /// Creates a new `ParamDefinition` with the given name and type.
    pub fn new(name: &str, param_type: ParamType) -> Self {
        Self {
            name: name.to_string(),
            param_type,
            value: None,
        }
    }
}

/// Parses a single line in the format `key=value`.
///
/// If the key matches a definition in `defs`, its value is parsed according to the defined type
/// and stored in the definition. Returns an error if the key is not found or parsing fails.
pub fn parse_line(line: &str, defs: &mut [ParamDefinition]) -> Result<()> {
    let line = line.trim_end_matches('\n');
    let Some(eq_idx) = line.find('=') else {
        return Err(anyhow!("Invalid line format, missing '=': {}", line));
    };

    let key = &line[..eq_idx];
    let val_str = &line[eq_idx + 1..];

    for def in defs.iter_mut() {
        if def.name == key {
            def.value = Some(match def.param_type {
                ParamType::Char => {
                    let c = val_str.chars().next().unwrap_or('\0');
                    ParamValue::Char(c)
                }
                ParamType::Int => {
                    let i = val_str
                        .parse::<i32>()
                        .map_err(|e| anyhow!("Failed to parse int '{}': {}", val_str, e))?;
                    ParamValue::Int(i)
                }
                ParamType::Float => {
                    let f = val_str
                        .parse::<f32>()
                        .map_err(|e| anyhow!("Failed to parse float '{}': {}", val_str, e))?;
                    ParamValue::Float(f)
                }
                ParamType::String => ParamValue::String(val_str.to_string()),
                ParamType::HexString => {
                    let mut bytes = Vec::new();
                    for chunk in val_str.split_whitespace() {
                        let byte = u8::from_str_radix(chunk, 16)
                            .map_err(|e| anyhow!("Failed to parse hex '{}': {}", chunk, e))?;
                        bytes.push(byte);
                    }
                    ParamValue::HexString(bytes)
                }
                ParamType::YesNo => {
                    let c = val_str.chars().next().unwrap_or('\0');
                    let is_yes = c == 'y' || c == 'Y';
                    ParamValue::YesNo(is_yes)
                }
            });
            return Ok(());
        }
    }

    Err(anyhow!("Did not find param definition for: {}", key))
}

/// Parses the contents of a configuration string, applying values to the provided definitions.
///
/// Lines starting with `#` or empty lines are ignored.
pub fn parse_file_content(content: &str, defs: &mut [ParamDefinition]) -> Result<()> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Keep original line format without trimming spaces from key/value unless needed,
        // to match C implementation behavior which searched for exact match on 'key=...'.
        // We'll use the original line for `parse_line`.
        let _ = parse_line(line, defs); // Ignore errors for unknown lines, similar to C
    }
    Ok(())
}

/// Parses a file, line by line, applying values to the provided definitions.
pub fn parse_file<P: AsRef<Path>>(path: P, defs: &mut [ParamDefinition]) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let _ = parse_line(&line, defs);
    }
    Ok(())
}
