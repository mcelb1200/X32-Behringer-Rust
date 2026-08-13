//! `x32_testparams` provides a library for parsing INI-style test parameter files.
//!
//! This is a Rust port of `Testparams.c` from the original C codebase. It supports parsing
//! lines in the format `key=value` into various strongly-typed parameter values.

use anyhow::{anyhow, Result};
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
                    let i = val_str.parse::<i32>().map_err(|e| anyhow!("Failed to parse int '{}': {}", val_str, e))?;
                    ParamValue::Int(i)
                }
                ParamType::Float => {
                    let f = val_str.parse::<f32>().map_err(|e| anyhow!("Failed to parse float '{}': {}", val_str, e))?;
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
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Represents the expected type of a parameter to parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    /// A single character
    Char,
    /// An integer value
    Int,
    /// A floating point value
    Float,
    /// A string value
    String,
    /// A space-separated list of hexadecimal bytes (e.g., "f0 f1 f2")
    HexString,
    /// A boolean yes/no value ("y" or "Y" becomes true)
    YesNo,
}

/// Represents the parsed value of a parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Char(char),
    Int(i32),
    Float(f32),
    String(String),
    HexString(Vec<u8>),
    YesNo(bool),
}

/// A parser for INI-style parameter files (`Testparams.c` equivalent).
pub struct TestParamsParser {
    schema: HashMap<String, ParamType>,
}

impl TestParamsParser {
    /// Creates a new parser with the given schema.
    pub fn new(schema: HashMap<String, ParamType>) -> Self {
        Self { schema }
    }

    /// Parses a file into a map of parsed parameter values.
    /// Returns an error if the file cannot be read, but ignores unparseable lines or missing parameters.
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<HashMap<String, ParamValue>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut results = HashMap::new();
        for line in reader.lines() {
            let line = line?;
            if let Some((k, v)) = self.parse_line(&line) {
                results.insert(k, v);
            }
        }

        Ok(results)
    }

    /// Parses a single line.
    /// Lines starting with '#' are ignored.
    /// Expected format is "key=value".
    /// Returns `Some((key, value))` if successful, otherwise `None`.
    pub fn parse_line(&self, line: &str) -> Option<(String, ParamValue)> {
        let line = line.trim_end(); // Strip trailing newline/whitespace
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        let (key, value_str) = line.split_once('=')?;

        let ptype = self.schema.get(key)?;

        let parsed_val = match ptype {
            ParamType::Char => {
                let c = value_str.chars().next()?;
                ParamValue::Char(c)
            }
            ParamType::Int => {
                let v = value_str.parse::<i32>().ok()?;
                ParamValue::Int(v)
            }
            ParamType::Float => {
                let v = value_str.parse::<f32>().ok()?;
                ParamValue::Float(v)
            }
            ParamType::String => ParamValue::String(value_str.to_string()),
            ParamType::HexString => {
                let mut bytes = Vec::new();
                for hex_part in value_str.split_whitespace() {
                    let byte = u8::from_str_radix(hex_part, 16).ok()?;
                    bytes.push(byte);
                }
                ParamValue::HexString(bytes)
            }
            ParamType::YesNo => {
                // According to C impl: line[i+1] == 'y' || line[i+1] == 'Y'
                let is_yes = value_str.starts_with('y') || value_str.starts_with('Y');
                ParamValue::YesNo(is_yes)
            }
        };

        Some((key.to_string(), parsed_val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn get_schema() -> HashMap<String, ParamType> {
        let mut schema = HashMap::new();
        schema.insert("aaa".to_string(), ParamType::Int);
        schema.insert("bbb".to_string(), ParamType::Float);
        schema.insert("ccc".to_string(), ParamType::Char);
        schema.insert("sss".to_string(), ParamType::String);
        schema.insert("hhh".to_string(), ParamType::HexString);
        schema.insert("yn_yes".to_string(), ParamType::YesNo);
        schema.insert("yn_no".to_string(), ParamType::YesNo);
        schema
    }

    #[test]
    fn test_parse_line() {
        let parser = TestParamsParser::new(get_schema());

        // Int
        assert_eq!(
            parser.parse_line("aaa=123"),
            Some(("aaa".to_string(), ParamValue::Int(123)))
        );

        // Float
        assert_eq!(
            parser.parse_line("bbb=3.14"),
            Some(("bbb".to_string(), ParamValue::Float(3.14)))
        );

        // Char
        assert_eq!(
            parser.parse_line("ccc=X"),
            Some(("ccc".to_string(), ParamValue::Char('X')))
        );

        // String
        assert_eq!(
            parser.parse_line("sss=hello world"),
            Some((
                "sss".to_string(),
                ParamValue::String("hello world".to_string())
            ))
        );

        // HexString
        assert_eq!(
            parser.parse_line("hhh=f0 0a 1b"),
            Some((
                "hhh".to_string(),
                ParamValue::HexString(vec![0xf0, 0x0a, 0x1b])
            ))
        );

        // YesNo
        assert_eq!(
            parser.parse_line("yn_yes=y"),
            Some(("yn_yes".to_string(), ParamValue::YesNo(true)))
        );
        assert_eq!(
            parser.parse_line("yn_yes=Yes"),
            Some(("yn_yes".to_string(), ParamValue::YesNo(true)))
        );
        assert_eq!(
            parser.parse_line("yn_no=n"),
            Some(("yn_no".to_string(), ParamValue::YesNo(false)))
        );
        assert_eq!(
            parser.parse_line("yn_no=Nope"),
            Some(("yn_no".to_string(), ParamValue::YesNo(false)))
        );

        // Comments
        assert_eq!(parser.parse_line("# aaa=123"), None);

        // Empty line
        assert_eq!(parser.parse_line(""), None);

        // Invalid key
        assert_eq!(parser.parse_line("unknown=123"), None);

        // Invalid data type formatting
        assert_eq!(parser.parse_line("aaa=abc"), None); // Invalid int
    }

    #[test]
    fn test_parse_file() {
        let parser = TestParamsParser::new(get_schema());

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# This is a test file").unwrap();
        writeln!(file, "aaa=42").unwrap();
        writeln!(file, "bbb=-1.5").unwrap();
        writeln!(file, "ccc=A").unwrap();
        writeln!(file, "sss=test string").unwrap();
        writeln!(file, "hhh=01 02 03").unwrap();
        writeln!(file, "yn_yes=y").unwrap();
        writeln!(file, "yn_no=n").unwrap();
        writeln!(file, "unknown=ignored").unwrap();

        let path = file.path();
        let results = parser.parse_file(path).unwrap();

        assert_eq!(results.get("aaa"), Some(&ParamValue::Int(42)));
        assert_eq!(results.get("bbb"), Some(&ParamValue::Float(-1.5)));
        assert_eq!(results.get("ccc"), Some(&ParamValue::Char('A')));
        assert_eq!(
            results.get("sss"),
            Some(&ParamValue::String("test string".to_string()))
        );
        assert_eq!(
            results.get("hhh"),
            Some(&ParamValue::HexString(vec![1, 2, 3]))
        );
        assert_eq!(results.get("yn_yes"), Some(&ParamValue::YesNo(true)));
        assert_eq!(results.get("yn_no"), Some(&ParamValue::YesNo(false)));
        assert_eq!(results.get("unknown"), None);
    }
}
