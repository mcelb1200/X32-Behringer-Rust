
use anyhow::Result;
use std::net::UdpSocket;
use crate::{Config, Track};
use x32_lib::cparse;

pub fn handle_x32_message(buf: &[u8], socket: &UdpSocket, config: &Config, tracks: &mut [Track], bank_offset: usize) -> Result<()> {
    // Parse X32 message and send to Reaper
    Ok(())
}
