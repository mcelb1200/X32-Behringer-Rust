
use anyhow::Result;
use std::net::UdpSocket;
use crate::{Config, Track};
use x32_lib::cparse;

pub fn handle_reaper_message(buf: &[u8], socket: &UdpSocket, config: &Config, tracks: &mut [Track], bank_offset: usize) -> Result<()> {
    // Parse Reaper message and send to X32
    Ok(())
}
