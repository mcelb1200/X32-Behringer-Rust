//! This library module provides the server implementation for the X32 emulator.
//! It is exposed as a library to allow integration tests to run the emulator in-process.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

pub mod server {
    use anyhow::Result;
    use std::net::{SocketAddr, UdpSocket};
    use std::sync::mpsc::Receiver;
    use x32_core::Mixer;

    /// A type alias for a closure that can be used to initialize the mixer's state.
    type Seeder = Option<Box<dyn FnOnce(&mut Mixer) + Send>>;

    /// Runs the X32 emulator server.
    ///
    /// This function binds to the specified UDP address and enters a loop where it
    /// receives OSC messages, dispatches them to the `Mixer` instance, and sends
    /// back any responses.
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - The address to bind the UDP socket to (e.g., "0.0.0.0:10023").
    /// * `seeder` - An optional closure to initialize the mixer's state before starting.
    /// * `shutdown` - An optional channel receiver to signal the server to stop.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn run(bind_addr: &str, seeder: Seeder, shutdown: Option<Receiver<()>>) -> Result<()> {
        let addr: SocketAddr = bind_addr.parse()?;
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(std::time::Duration::from_millis(10)))?;
        let mut mixer = Mixer::new();

        if let Some(seeder) = seeder {
            seeder(&mut mixer);
        }

        println!("X32 Emulator listening on {}", addr);

        let mut buf = [0; 8192];
        loop {
            if let Some(shutdown) = &shutdown {
                if shutdown.try_recv().is_ok() {
                    break;
                }
            }

            match socket.recv_from(&mut buf) {
                Ok((len, remote_addr)) => match mixer.dispatch(&buf[..len], remote_addr) {
                    Ok(responses) => {
                        for (addr, response) in responses {
                            socket.send_to(&response, addr)?;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error handling message: {}", e);
                    }
                },
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    // No data received, continue
                }
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}
pub use x32_core::Mixer;
