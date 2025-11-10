//! `osc_lib` is a library for encoding and decoding Open Sound Control (OSC) 1.0 messages.
//!
//! This library provides data structures for representing OSC messages and arguments,
//! along with methods for serializing and deserializing them to and from byte streams
//! and string representations.
//!
//! # Examples
//!
//! ## Creating and serializing an OSC message
//!
//! ```
//! use osc_lib::{OscMessage, OscArg};
//!
//! let msg = OscMessage::new("/ch/01/mix/fader".to_string(), vec![OscArg::Float(0.75)]);
//! let bytes = msg.to_bytes().unwrap();
//!
//! // The resulting byte vector can be sent over a network.
//! ```
//!
//!
//! ## Parsing an OSC message from a string
//!
//! ```
//! use osc_lib::{OscMessage, OscArg};
//! use std::str::FromStr;
//!
//! let msg_str = r#"/ch/01/mix/fader ,f 0.75"#;
//! let msg = OscMessage::from_str(msg_str).unwrap();
//!
//! assert_eq!(msg.path, "/ch/01/mix/fader");
//! assert_eq!(msg.args, vec![OscArg::Float(0.75)]);
//! ```

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Read, Write};
use std::str::FromStr;
use std::string::FromUtf8Error;

#[cfg(test)]
mod tests;

/// Represents the possible errors that can occur when working with OSC messages.
#[derive(Debug)]
pub enum OscError {
    /// An I/O error occurred while reading or writing.
    Io(io::Error),
    /// A string was not valid UTF-8.
    Utf8(FromUtf8Error),
    /// The OSC type tag string was invalid (e.g., did not start with ',').
    InvalidTypeTag,
    /// An unsupported OSC type tag was encountered.
    UnsupportedTypeTag(char),
    /// A general parsing error occurred.
    ParseError(String),
    /// An unexpected response was received from the mixer.
    UnexpectedResponse,
}

impl std::fmt::Display for OscError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OscError::Io(e) => write!(f, "I/O error: {}", e),
            OscError::Utf8(e) => write!(f, "UTF-8 conversion error: {}", e),
            OscError::InvalidTypeTag => write!(f, "Invalid OSC type tag string"),
            OscError::UnsupportedTypeTag(c) => write!(f, "Unsupported OSC type tag: {}", c),
            OscError::ParseError(s) => write!(f, "Parse error: {}", s),
            OscError::UnexpectedResponse => write!(f, "Unexpected response from mixer"),
        }
    }
}

impl std::error::Error for OscError {}

impl From<io::Error> for OscError {
    fn from(err: io::Error) -> Self {
        OscError::Io(err)
    }
}

impl From<FromUtf8Error> for OscError {
    fn from(err: FromUtf8Error) -> Self {
        OscError::Utf8(err)
    }
}

/// A type alias for `Result` with the error type `OscError`.
pub type Result<T> = std::result::Result<T, OscError>;

/// Represents a single argument in an OSC message.
#[derive(Debug, PartialEq, Clone)]
pub enum OscArg {
    /// A 32-bit integer (`i` in OSC type tags).
    Int(i32),
    /// A 32-bit float (`f` in OSC type tags).
    Float(f32),
    /// A string (`s` in OSC type tags).
    String(String),
    /// A blob of binary data (`b` in OSC type tags).
    Blob(Vec<u8>),
}

/// Represents a single OSC message, containing a path and a list of arguments.
#[derive(Debug, PartialEq)]
pub struct OscMessage {
    /// The OSC address pattern (e.g., `/ch/01/mix/fader`).
    pub path: String,
    /// The arguments to the message.
    pub args: Vec<OscArg>,
}

impl OscMessage {
    /// Creates a new `OscMessage` with the given path and arguments.
    ///
    /// # Arguments
    ///
    /// * `path` - The OSC address pattern.
    /// * `args` - A vector of `OscArg` values.
    pub fn new(path: String, args: Vec<OscArg>) -> Self {
        OscMessage { path, args }
    }

    /// Deserializes an `OscMessage` from a byte slice.
    ///
    /// The byte slice should be a valid OSC 1.0 message, including the path,
    /// type tag string, and arguments, all properly padded.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let path = read_osc_string(&mut cursor)?;
        let type_tags = read_osc_string(&mut cursor)?;

        if !type_tags.starts_with(',') {
            return Err(OscError::InvalidTypeTag);
        }

        let mut args = Vec::new();
        for tag in type_tags[1..].chars() {
            match tag {
                'i' => {
                    let val = cursor.read_i32::<BigEndian>()?;
                    args.push(OscArg::Int(val));
                }
                'f' => {
                    let val = cursor.read_f32::<BigEndian>()?;
                    args.push(OscArg::Float(val));
                }
                's' => {
                    let val = read_osc_string(&mut cursor)?;
                    args.push(OscArg::String(val));
                }
                'b' => {
                    let len = cursor.read_i32::<BigEndian>()? as usize;
                    let mut buf = vec![0; len];
                    cursor.read_exact(&mut buf)?;
                    args.push(OscArg::Blob(buf));
                    let current_pos = cursor.position();
                    let next_aligned_pos = (current_pos + 3) & !3;
                    cursor.set_position(next_aligned_pos);
                }
                _ => return Err(OscError::UnsupportedTypeTag(tag)),
            }
        }

        Ok(OscMessage { path, args })
    }

    /// Serializes the `OscMessage` to a `Vec<u8>`.
    ///
    /// The resulting byte vector will be a valid OSC 1.0 message, ready to be
    /// sent over a network.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        write_osc_string(&mut bytes, &self.path)?;

        let mut type_tags = ",".to_string();
        for arg in &self.args {
            match arg {
                OscArg::Int(_) => type_tags.push('i'),
                OscArg::Float(_) => type_tags.push('f'),
                OscArg::String(_) => type_tags.push('s'),
                OscArg::Blob(_) => type_tags.push('b'),
            }
        }
        write_osc_string(&mut bytes, &type_tags)?;

        for arg in &self.args {
            match arg {
                OscArg::Int(val) => bytes.write_i32::<BigEndian>(*val)?,
                OscArg::Float(val) => bytes.write_f32::<BigEndian>(*val)?,
                OscArg::String(val) => write_osc_string(&mut bytes, val)?,
                OscArg::Blob(val) => {
                    bytes.write_i32::<BigEndian>(val.len() as i32)?;
                    bytes.write_all(val)?;
                    while !bytes.len().is_multiple_of(4) {
                        bytes.write_u8(0)?;
                    }
                }
            }
        }

        Ok(bytes)
    }
}

impl FromStr for OscMessage {
    type Err = OscError;
    /// Creates an `OscMessage` from a string representation.
    ///
    /// The string format is the OSC path followed by a space, then the type tag string,
    /// and then a space-separated list of arguments. String arguments with spaces
    /// should be enclosed in double quotes.
    ///
    /// # Example
    ///
    /// ```
    /// use osc_lib::{OscMessage, OscArg};
    /// use std::str::FromStr;
    ///
    /// let msg_str = r#"/ch/01/mix/fader ,f 0.75"#;
    /// let msg = OscMessage::from_str(msg_str).unwrap();
    /// assert_eq!(msg.path, "/ch/01/mix/fader");
    /// assert_eq!(msg.args, vec![OscArg::Float(0.75)]);
    /// ```
    fn from_str(s: &str) -> Result<Self> {
        let tokens = tokenize(s)?;
        let mut it = tokens.iter();
        let path = it
            .next()
            .ok_or(OscError::ParseError("Empty command string".to_string()))?
            .to_string();
        let mut args = Vec::new();

        if let Some(type_tags) = it.next() {
            if !type_tags.starts_with(',') {
                return Err(OscError::InvalidTypeTag);
            }

            for tag in type_tags[1..].chars() {
                let val_str = it.next().ok_or(OscError::ParseError(format!(
                    "Missing value for type tag '{}'",
                    tag
                )))?;
                match tag {
                    'i' => {
                        let val = i32::from_str(val_str)
                            .map_err(|e| OscError::ParseError(e.to_string()))?;
                        args.push(OscArg::Int(val));
                    }
                    'f' => {
                        let val = f32::from_str(val_str)
                            .map_err(|e| OscError::ParseError(e.to_string()))?;
                        args.push(OscArg::Float(val));
                    }
                    's' => {
                        args.push(OscArg::String(val_str.to_string()));
                    }
                    _ => return Err(OscError::UnsupportedTypeTag(tag)),
                }
            }
            if it.next().is_some() {
                return Err(OscError::ParseError(
                    "Extra arguments at end of command string".to_string(),
                ));
            }
        }

        Ok(OscMessage { path, args })
    }
}

impl std::fmt::Display for OscMessage {
    /// Converts the `OscMessage` to a string representation.
    ///
    /// # Example
    ///
    /// ```
    /// use osc_lib::{OscMessage, OscArg};
    ///
    /// let msg = OscMessage::new("/ch/01/mix/fader".to_string(), vec![OscArg::Float(0.75)]);
    /// let msg_str = msg.to_string();
    ///
    /// assert_eq!(msg_str, "/ch/01/mix/fader ,f 0.75");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)?;
        if !self.args.is_empty() {
            write!(f, " ,")?;
            for arg in &self.args {
                match arg {
                    OscArg::Int(_) => write!(f, "i")?,
                    OscArg::Float(_) => write!(f, "f")?,
                    OscArg::String(_) => write!(f, "s")?,
                    OscArg::Blob(_) => write!(f, "b")?,
                }
            }
            for arg in &self.args {
                write!(f, " ")?;
                match arg {
                    OscArg::Int(val) => write!(f, "{}", val)?,
                    OscArg::Float(val) => write!(f, "{}", val)?,
                    OscArg::String(val) => write!(f, "\"{}\"", val.replace("\"", "\\\""))?,
                    OscArg::Blob(_) => write!(f, "[blob]")?,
                }
            }
        }
        Ok(())
    }
}

/// Tokenizes a string for OSC message parsing, handling quoted strings.
///
/// This function splits a string into tokens by whitespace, but treats text
/// enclosed in double quotes as a single token.
pub fn tokenize(s: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quote = false;
    let mut it = s.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => {
                if let Some(next_c) = it.next() {
                    current_token.push(next_c);
                }
            }
            '"' => {
                if in_quote {
                    // Closing quote
                    tokens.push(current_token);
                    current_token = String::new();
                }
                in_quote = !in_quote;
            }
            ' ' if !in_quote => {
                if !current_token.is_empty() {
                    tokens.push(current_token);
                    current_token = String::new();
                }
            }
            _ => {
                current_token.push(c);
            }
        }
    }
    if !current_token.is_empty() {
        tokens.push(current_token);
    }
    Ok(tokens)
}

/// Reads a null-terminated and 4-byte padded OSC string from a cursor.
fn read_osc_string(cursor: &mut Cursor<&[u8]>) -> Result<String> {
    let mut bytes = Vec::new();
    loop {
        let byte = cursor.read_u8()?;
        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }
    let string = String::from_utf8(bytes)?;

    let current_pos = cursor.position();
    let next_aligned_pos = (current_pos + 3) & !3;
    cursor.set_position(next_aligned_pos);

    Ok(string)
}

/// Writes a null-terminated and 4-byte padded OSC string to a byte vector.
fn write_osc_string(bytes: &mut Vec<u8>, s: &str) -> Result<()> {
    bytes.write_all(s.as_bytes())?;
    bytes.write_u8(0)?;
    while !bytes.len().is_multiple_of(4) {
        bytes.write_u8(0)?;
    }
    Ok(())
}
