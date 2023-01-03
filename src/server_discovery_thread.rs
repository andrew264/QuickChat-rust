use std::net::UdpSocket;

use log::{debug, trace};

const DISCOVERY_REQUEST: &str = "DISCOVER_CHAT_SERVER_REQUEST";
const DISCOVERY_RESPONSE: &str = "DISCOVER_CHAT_SERVER_RESPONSE";
const PORT: u16 = 8888;

pub struct DiscoveryThread {
    socket: UdpSocket,
}

impl DiscoveryThread {
    pub fn new() -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", PORT))?;
        debug!("Opening Socket: {:?}", socket.local_addr().unwrap());
        socket.set_broadcast(true).expect("Failed to set broadcast");
        debug!("Enabled broadcast for socket: {:?}", socket.local_addr().unwrap());
        Ok(Self {
            socket,
        })
    }

    pub fn run(self) {
        loop {
            let mut buf = [0u8; 15000];
            let (amt, src) = self.socket.recv_from(&mut buf)
                .expect("Failed to receive packet");
            trace!("Received packet from: {:?}", src);
            let message = String::from_utf8_lossy(&buf[..amt]);
            if message == DISCOVERY_REQUEST {
                trace!("Received discovery request from: {:?}", src);
                let response = DISCOVERY_RESPONSE.as_bytes();
                self.socket.send_to(response, src)
                    .expect("Failed to send response to client");
                trace!("Sent discovery response to: {:?}", src);
            }
        }
    }
}
