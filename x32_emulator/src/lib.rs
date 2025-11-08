pub mod server {
    use anyhow::Result;
    use std::net::{SocketAddr, UdpSocket};
    use x32_core::Mixer;
    use std::sync::mpsc::Receiver;

    pub fn run(
        bind_addr: &str,
        seeder: Option<Box<dyn FnOnce(&mut Mixer) + Send>>,
        shutdown: Option<Receiver<()>>,
    ) -> Result<()> {
        let addr: SocketAddr = bind_addr.parse()?;
        let socket = UdpSocket::bind(&addr)?;
        socket.set_nonblocking(true)?;
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
                Ok((len, remote_addr)) => {
                    match mixer.dispatch(&buf[..len]) {
                        Ok(Some(response)) => {
                            socket.send_to(&response, remote_addr)?;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            eprintln!("Error handling message: {}", e);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data received, continue
                }
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }
}
pub use x32_core::Mixer;
