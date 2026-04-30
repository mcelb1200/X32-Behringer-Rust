//! `osc_lib` is a library for encoding and decoding Open Sound Control (OSC) 1.0 messages.
//!
//! This library provides data structures for representing OSC messages and arguments,
//! along with methods for serializing and deserializing them to and from byte streams
//! and string representations.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200
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

use byteorder::{BigEndian, ReadBytesExt};
use std::fmt::Write;
use std::io::{self, Cursor};
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
            OscError::InvalidTypeTag => f.write_str("Invalid OSC type tag string"),
            OscError::UnsupportedTypeTag(c) => write!(f, "Unsupported OSC type tag: {}", c),
            OscError::ParseError(s) => {
                f.write_str("Parse error: ")?;
                f.write_str(s)
            }
            OscError::UnexpectedResponse => f.write_str("Unexpected response from mixer"),
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

/// Helper function to calculate padded size.
fn padded_size(len: usize) -> usize {
    (len + 3) & !3
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
    ///
    /// # Returns
    ///
    /// A new `OscMessage` instance.
    pub fn new(path: String, args: Vec<OscArg>) -> Self {
        OscMessage { path, args }
    }

    /// Deserializes an `OscMessage` from a byte slice.
    ///
    /// The byte slice should be a valid OSC 1.0 message, including the path,
    /// type tag string, and arguments, all properly padded.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The byte slice containing the OSC message data.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized `OscMessage` or an `OscError`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let path = read_osc_string(&mut cursor)?;
        // OPTIMIZATION: Parse type tags as raw bytes instead of allocating a String
        // and validating UTF-8, as type tags are guaranteed to be ASCII.
        // This eliminates a Vec allocation and string conversion per OSC message parsed.
        let type_tags = read_osc_string_bytes(&mut cursor)?;

        if type_tags.is_empty() || type_tags[0] != b',' {
            return Err(OscError::InvalidTypeTag);
        }

        let mut args = Vec::with_capacity(type_tags.len().saturating_sub(1));
        for &tag_byte in &type_tags[1..] {
            match tag_byte as char {
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

                    // OPTIMIZATION: Instead of allocating a zero-initialized buffer `vec![0; len]`
                    // and calling `cursor.read_exact(&mut buf)`, directly slice the underlying buffer
                    // and copy it using `.to_vec()`. This skips the zero-initialization overhead,
                    // which is significant for large binary blobs.
                    let current_pos = cursor.position() as usize;
                    let buf_ref = cursor.get_ref();

                    let end_pos = current_pos + len;
                    if end_pos > buf_ref.len() {
                        return Err(OscError::ParseError("Unexpected end of buffer".to_string()));
                    }

                    let buf = buf_ref[current_pos..end_pos].to_vec();
                    args.push(OscArg::Blob(buf));

                    let next_aligned_pos = (end_pos + 3) & !3;
                    cursor.set_position(next_aligned_pos as u64);
                }
                _ => return Err(OscError::UnsupportedTypeTag(tag_byte as char)),
            }
        }

        Ok(OscMessage { path, args })
    }

    /// Serializes an OSC message directly from a path and an iterator of argument references.
    ///
    /// This bypasses the overhead of heap-allocating an owned `OscMessage` struct
    /// during hot paths like response generation.
    ///
    /// # Arguments
    ///
    /// * `path` - The OSC address pattern.
    /// * `args` - An iterator of references to `OscArg` values.
    ///
    /// # Returns
    ///
    /// A `Result` containing the serialized byte vector or an `OscError`.
    pub fn serialize_to_bytes<'a, I>(path: &str, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = &'a OscArg> + Clone,
    {
        // First pass: Calculate the total size required and collect type tags
        let path_size = padded_size(path.len() + 1);

        let mut type_tags = Vec::with_capacity(8);
        type_tags.push(b',');

        let mut args_size = 0;
        for arg in args.clone() {
            match arg {
                OscArg::Int(_) => {
                    args_size += 4;
                    type_tags.push(b'i');
                }
                OscArg::Float(_) => {
                    args_size += 4;
                    type_tags.push(b'f');
                }
                OscArg::String(s) => {
                    args_size += padded_size(s.len() + 1);
                    type_tags.push(b's');
                }
                OscArg::Blob(b) => {
                    args_size += 4 + padded_size(b.len());
                    type_tags.push(b'b');
                }
            }
        }
        type_tags.push(0); // Null terminator
        let type_tags_size = padded_size(type_tags.len());

        let total_size = path_size + type_tags_size + args_size;
        let mut bytes = Vec::with_capacity(total_size);

        // Write path
        write_osc_string(&mut bytes, path)?;

        // Write type tags
        bytes.extend_from_slice(&type_tags);

        // OPTIMIZATION: Calculate exact padding required instead of a while loop.
        let rem = bytes.len() % 4;
        if rem != 0 {
            let pad_len = 4 - rem;
            bytes.extend_from_slice(&[0, 0, 0][..pad_len]);
        }

        // Second pass: Write args
        for arg in args {
            match arg {
                OscArg::Int(val) => bytes.extend_from_slice(&val.to_be_bytes()),
                OscArg::Float(val) => bytes.extend_from_slice(&val.to_be_bytes()),
                OscArg::String(val) => write_osc_string(&mut bytes, val)?,
                OscArg::Blob(val) => {
                    bytes.extend_from_slice(&(val.len() as i32).to_be_bytes());
                    bytes.extend_from_slice(val);

                    // OPTIMIZATION: Calculate exact padding required instead of a while loop.
                    let rem = bytes.len() % 4;
                    if rem != 0 {
                        let pad_len = 4 - rem;
                        bytes.extend_from_slice(&[0, 0, 0][..pad_len]);
                    }
                }
            }
        }

        Ok(bytes)
    }

    /// Serializes the `OscMessage` to a `Vec<u8>`.
    ///
    /// The resulting byte vector will be a valid OSC 1.0 message, ready to be
    /// sent over a network.
    ///
    /// # Returns
    ///
    /// A `Result` containing the serialized byte vector or an `OscError`.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Self::serialize_to_bytes(&self.path, self.args.iter())
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
    /// # Arguments
    ///
    /// * `s` - The string representation of the OSC message.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `OscMessage` or an `OscError`.
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
        let mut args = Vec::new(); // Capacity will be reserved later

        if let Some(type_tags) = it.next() {
            if !type_tags.starts_with(',') {
                return Err(OscError::InvalidTypeTag);
            }

            args.reserve_exact(type_tags.len().saturating_sub(1));

            // OPTIMIZATION: Use .bytes() instead of .chars() to bypass UTF-8 decoding
            // overhead since OSC type tags are guaranteed to be ASCII.
            for tag in type_tags[1..].bytes() {
                let val_str = it.next().ok_or(OscError::ParseError(format!(
                    "Missing value for type tag '{}'",
                    tag as char
                )))?;
                match tag {
                    b'i' => {
                        let val = i32::from_str(val_str)
                            .map_err(|e| OscError::ParseError(e.to_string()))?;
                        args.push(OscArg::Int(val));
                    }
                    b'f' => {
                        let val = f32::from_str(val_str)
                            .map_err(|e| OscError::ParseError(e.to_string()))?;
                        args.push(OscArg::Float(val));
                    }
                    b's' => {
                        args.push(OscArg::String(val_str.to_string()));
                    }
                    b'b' => {
                        if val_str.len() % 2 != 0 {
                            return Err(OscError::ParseError(format!(
                                "Invalid hex string length for blob: {}",
                                val_str
                            )));
                        }
                        let mut blob = Vec::with_capacity(val_str.len() / 2);
                        let bytes = val_str.as_bytes();

                        // OPTIMIZATION: Manually parse hex bytes instead of using `u8::from_str_radix`
                        // on string slices. This bypasses the string slice parsing overhead and is
                        // significantly faster in hot loops.
                        for i in (0..bytes.len()).step_by(2) {
                            let high = match bytes[i] {
                                b'0'..=b'9' => bytes[i] - b'0',
                                b'a'..=b'f' => bytes[i] - b'a' + 10,
                                b'A'..=b'F' => bytes[i] - b'A' + 10,
                                _ => {
                                    return Err(OscError::ParseError(format!(
                                        "Invalid hex character in blob: {}",
                                        bytes[i] as char
                                    )));
                                }
                            };
                            let low = match bytes[i + 1] {
                                b'0'..=b'9' => bytes[i + 1] - b'0',
                                b'a'..=b'f' => bytes[i + 1] - b'a' + 10,
                                b'A'..=b'F' => bytes[i + 1] - b'A' + 10,
                                _ => {
                                    return Err(OscError::ParseError(format!(
                                        "Invalid hex character in blob: {}",
                                        bytes[i + 1] as char
                                    )));
                                }
                            };
                            blob.push((high << 4) | low);
                        }
                        args.push(OscArg::Blob(blob));
                    }
                    _ => return Err(OscError::UnsupportedTypeTag(tag as char)),
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
    /// # Arguments
    ///
    /// * `f` - The formatter to write the string to.
    ///
    /// # Returns
    ///
    /// A `std::fmt::Result` indicating success or failure.
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
        f.write_str(&self.path)?;
        if !self.args.is_empty() {
            f.write_str(" ,")?;
            for arg in &self.args {
                match arg {
                    OscArg::Int(_) => f.write_str("i")?,
                    OscArg::Float(_) => f.write_str("f")?,
                    OscArg::String(_) => f.write_str("s")?,
                    OscArg::Blob(_) => f.write_str("b")?,
                }
            }
            for arg in &self.args {
                f.write_str(" ")?;
                match arg {
                    OscArg::Int(val) => write!(f, "{}", val)?,
                    OscArg::Float(val) => write!(f, "{}", val)?,
                    OscArg::String(val) => {
                        f.write_str("\"")?;
                        f.write_str(val)?;
                        f.write_str("\"")?;
                    }
                    OscArg::Blob(val) => {
                        // OPTIMIZATION: Manually write hex characters instead of using the `write!` macro
                        // with formatting `{:02x}`. This avoids the machinery of std::fmt and is
                        // significantly faster for large binary blobs in hot paths.
                        static HEX: &[u8; 16] = b"0123456789abcdef";
                        for byte in val {
                            f.write_char(HEX[(byte >> 4) as usize] as char)?;
                            f.write_char(HEX[(byte & 0x0f) as usize] as char)?;
                        }
                        Ok(())
                    }?,
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
///
/// # Arguments
///
/// * `s` - The string to tokenize.
///
/// # Returns
///
/// A `Result` containing a vector of tokens (strings) or an `OscError`.
pub fn tokenize(s: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quote = false;
    let mut escaped = false;
    for c in s.chars() {
        if escaped {
            current_token.push(c);
            escaped = false;
            continue;
        }

        match c {
            '\\' => {
                escaped = true;
            }
            '"' => {
                if !in_quote {
                    // Opening quote
                    if !current_token.is_empty() {
                        tokens.push(current_token);
                        current_token = String::new();
                    }
                } else {
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
    if in_quote {
        return Err(OscError::ParseError(
            "Unmatched quote in command string".to_string(),
        ));
    }
    if !current_token.is_empty() {
        tokens.push(current_token);
    }
    Ok(tokens)
}

/// Reads a null-terminated and 4-byte padded OSC string from a cursor, returning raw bytes.
///
/// # Arguments
///
/// * `cursor` - A mutable reference to a cursor over the byte slice.
///
/// # Returns
///
/// A `Result` containing the parsed string bytes or an `OscError`.
fn read_osc_string_bytes<'a>(cursor: &mut Cursor<&'a [u8]>) -> Result<&'a [u8]> {
    let pos = cursor.position() as usize;
    let buf = cursor.get_ref();

    if pos >= buf.len() {
        return Err(OscError::ParseError("Unexpected end of buffer".to_string()));
    }

    let remainder = &buf[pos..];

    // Find the null terminator byte (0)
    let null_pos = match remainder.iter().position(|&b| b == 0) {
        Some(p) => p,
        None => {
            return Err(OscError::ParseError(
                "Missing null terminator in string".to_string(),
            ));
        }
    };

    // Extract the string bytes
    let string_bytes = &remainder[..null_pos];

    // Calculate the new position after the null terminator and padding
    let new_pos = pos + null_pos + 1; // +1 for the null terminator
    let next_aligned_pos = (new_pos + 3) & !3;

    // Ensure we don't set the cursor past the end of the buffer
    let final_pos = std::cmp::min(next_aligned_pos, buf.len());
    cursor.set_position(final_pos as u64);

    Ok(string_bytes)
}

/// Reads a null-terminated and 4-byte padded OSC string from a cursor.
///
/// # Arguments
///
/// * `cursor` - A mutable reference to a cursor over the byte slice.
///
/// # Returns
///
/// A `Result` containing the parsed string or an `OscError`.
fn read_osc_string(cursor: &mut Cursor<&[u8]>) -> Result<String> {
    let string_bytes = read_osc_string_bytes(cursor)?;
    // Extract the string bytes and convert to String
    let string = String::from_utf8(string_bytes.to_vec())?;
    Ok(string)
}

/// Writes a null-terminated and 4-byte padded OSC string to a byte vector.
///
/// # Arguments
///
/// * `bytes` - A mutable reference to the byte vector.
/// * `s` - The string to write.
///
/// # Returns
///
/// A `Result` indicating success or failure.
fn write_osc_string(bytes: &mut Vec<u8>, s: &str) -> Result<()> {
    bytes.extend_from_slice(s.as_bytes());
    bytes.push(0);

    // OPTIMIZATION: Calculate exact padding required instead of a while loop.
    // This allows rustc/LLVM to optimize away repeated bounds checks and
    // branch predictions when writing the 0..3 trailing null bytes.
    let rem = bytes.len() % 4;
    if rem != 0 {
        let pad_len = 4 - rem;
        bytes.extend_from_slice(&[0, 0, 0][..pad_len]);
    }

    Ok(())
}
