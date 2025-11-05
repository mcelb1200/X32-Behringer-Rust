
use std::io;
use std::net::{AddrParseError, SocketAddr, UdpSocket};
use std::time::Duration;
use osc_lib::{OscMessage, OscArg, OscError};

pub mod fx_rtn;
pub use fx_rtn::*;

#[derive(Debug)]
pub enum X32Error {
    Io(io::Error),
    AddrParse(AddrParseError),
    Osc(OscError),
    UnexpectedResponse,
    Custom(String),
}

impl std::fmt::Display for X32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X32Error::Io(e) => write!(f, "I/O error: {}", e),
            X32Error::AddrParse(e) => write!(f, "Address parsing error: {}", e),
            X32Error::Osc(e) => write!(f, "OSC error: {}", e),
            X32Error::UnexpectedResponse => write!(f, "Unexpected response from X32"),
            X32Error::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for X32Error {}

impl From<io::Error> for X32Error {
    fn from(err: io::Error) -> Self {
        X32Error::Io(err)
    }
}

impl From<AddrParseError> for X32Error {
    fn from(err: AddrParseError) -> Self {
        X32Error::AddrParse(err)
    }
}

impl From<OscError> for X32Error {
    fn from(err: OscError) -> Self {
        X32Error::Osc(err)
    }
}

impl From<String> for X32Error {
    fn from(s: String) -> Self {
        X32Error::Custom(s)
    }
}

pub type Result<T> = std::result::Result<T, X32Error>;

pub fn create_socket(ip: &str, timeout_ms: u64) -> Result<UdpSocket> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;

    let x32_addr: SocketAddr = format!("{}:10023", ip).parse()?;
    socket.connect(x32_addr)?;

    Ok(socket)
}

pub fn get_fx_type(socket: &UdpSocket, slot: u8) -> Result<i32> {
    let msg = OscMessage::new(format!("/fx/{}/type", slot), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;
    if let Some(OscArg::Int(fx_type)) = response.args.get(0) {
        Ok(*fx_type)
    } else {
        Err(X32Error::UnexpectedResponse)
    }
}
