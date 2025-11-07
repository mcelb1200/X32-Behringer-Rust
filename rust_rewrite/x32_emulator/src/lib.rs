
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use x32_core::Mixer;

pub struct X32Emulator {
    mixer: Arc<Mutex<Mixer>>,
    ip: String,
    port: u16,
    handle: Option<JoinHandle<()>>,
    local_addr: Option<SocketAddr>,
    running: Arc<AtomicBool>,
}

impl Default for X32Emulator {
    fn default() -> Self {
        Self {
            mixer: Arc::new(Mutex::new(Mixer::new())),
            ip: "127.0.0.1".to_string(),
            port: 0,
            handle: None,
            local_addr: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl X32Emulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mixer_mut(&mut self) -> std::sync::MutexGuard<'_, Mixer> {
        self.mixer.lock().unwrap()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr.unwrap()
    }

    pub fn start(&mut self) {
        let mixer = self.mixer.clone();
        let addr: SocketAddr = format!("{}:{}", self.ip, self.port).parse().unwrap();
        let socket = UdpSocket::bind(addr).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        self.local_addr = Some(socket.local_addr().unwrap());

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let handle = thread::spawn(move || {
            let mut buf = [0; 8192];
            while running.load(Ordering::SeqCst) {
                if let Ok((len, remote_addr)) = socket.recv_from(&mut buf) {
                    let mut mixer = mixer.lock().unwrap();
                    match mixer.dispatch(&buf[..len]) {
                        Ok(Some(response)) => {
                            socket.send_to(&response, remote_addr).unwrap();
                        }
                        Ok(None) => {}
                        Err(e) => {
                            eprintln!("Error handling message: {}", e);
                        }
                    }
                }
            }
        });
        self.handle = Some(handle);
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.running.store(false, Ordering::SeqCst);
            handle.join().unwrap();
        }
    }
}

impl Drop for X32Emulator {
    fn drop(&mut self) {
        self.stop();
    }
}
