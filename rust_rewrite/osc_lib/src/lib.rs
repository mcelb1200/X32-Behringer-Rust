
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Read, Write};
use std::string::FromUtf8Error;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum OscError {
    Io(io::Error),
    Utf8(FromUtf8Error),
    InvalidTypeTag,
    UnsupportedTypeTag(char),
}

impl std::fmt::Display for OscError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OscError::Io(e) => write!(f, "I/O error: {}", e),
            OscError::Utf8(e) => write!(f, "UTF-8 conversion error: {}", e),
            OscError::InvalidTypeTag => write!(f, "Invalid OSC type tag string"),
            OscError::UnsupportedTypeTag(c) => write!(f, "Unsupported OSC type tag: {}", c),
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

pub type Result<T> = std::result::Result<T, OscError>;

#[derive(Debug, PartialEq, Clone)]
pub enum OscArg {
    Int(i32),
    Float(f32),
    String(String),
}

#[derive(Debug, PartialEq)]
pub struct OscMessage {
    pub path: String,
    pub args: Vec<OscArg>,
}

impl OscMessage {
    pub fn new(path: String, args: Vec<OscArg>) -> Self {
        OscMessage { path, args }
    }

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
                _ => return Err(OscError::UnsupportedTypeTag(tag)),
            }
        }

        Ok(OscMessage { path, args })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        write_osc_string(&mut bytes, &self.path)?;

        let mut type_tags = ",".to_string();
        for arg in &self.args {
            match arg {
                OscArg::Int(_) => type_tags.push('i'),
                OscArg::Float(_) => type_tags.push('f'),
                OscArg::String(_) => type_tags.push('s'),
            }
        }
        write_osc_string(&mut bytes, &type_tags)?;

        for arg in &self.args {
            match arg {
                OscArg::Int(val) => bytes.write_i32::<BigEndian>(*val)?,
                OscArg::Float(val) => bytes.write_f32::<BigEndian>(*val)?,
                OscArg::String(val) => write_osc_string(&mut bytes, val)?,
            }
        }

        Ok(bytes)
    }
}

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

fn write_osc_string(bytes: &mut Vec<u8>, s: &str) -> Result<()> {
    bytes.write_all(s.as_bytes())?;
    bytes.write_u8(0)?;
    while bytes.len() % 4 != 0 {
        bytes.write_u8(0)?;
    }
    Ok(())
}
