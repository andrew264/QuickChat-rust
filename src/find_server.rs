use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::str::FromStr;
use std::time::Duration;

use log::{debug, info, trace};

use crate::adapter;

pub(crate) fn get_ip() -> Option<IpAddr> {
    const REQUEST_MESSAGE: &[u8] = "DISCOVER_CHAT_SERVER_REQUEST".as_bytes();

    // Open a random port to send the package
    let socket = UdpSocket::bind("0.0.0.0:1234").expect("Failed to bind socket");
    debug!("Opening Socket: {:?}", socket.local_addr().unwrap());
    socket.set_broadcast(true).expect("Failed to set broadcast");
    debug!("Enabled broadcast");
    socket.set_read_timeout(Some(Duration::new(5, 0))).expect("Failed to set timeout");

    // Try the 255.255.255.255 first
    let broadcast_addr = SocketAddr::new(IpAddr::from_str("255.255.255.255").unwrap(), 8888);
    debug!("Broadcasting to: {:?}\n", broadcast_addr);
    socket.send_to(REQUEST_MESSAGE, broadcast_addr).expect("Failed to send broadcast");

    for mut adapt in adapter::get_adapters() {
        let broadcast_addr = adapt.broadcast_address();
        if broadcast_addr.is_some() {
            trace!("Adaptor Name: {:?}", adapt.get_adapter_name());
            debug!("Broadcasting to: {:?}\n", broadcast_addr);
            socket
                .send_to(REQUEST_MESSAGE, SocketAddr::new(broadcast_addr.unwrap(), 1234))
                .expect("Failed to send broadcast");
        }
    }

    debug!("Waiting for a reply from Server!");
    // Wait for a response
    let mut receive_buf = [0; 15000];
    let (received_bytes, server_addr) = socket.recv_from(&mut receive_buf).expect("Failed to receive packet");
    let message = String::from_utf8(receive_buf[..received_bytes].to_vec()).expect("Failed to convert packet data to string");
    // Check if the message is correct
    if message.trim() == "DISCOVER_CHAT_SERVER_RESPONSE" {
        debug!("Broadcast response from server: {}", server_addr.ip());
        return Some(server_addr.ip());
    }

    info!("Timeout: No response from Server!");
    None
}