use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

use log::{debug, trace};

use crate::server;

pub struct ClientHandler {
    buffer_reader: BufReader<TcpStream>,
    buffer_writer: BufWriter<TcpStream>,
    username: String,
    pub(crate) client_name: String,
}

impl ClientHandler {
    pub fn new(client_socket: TcpStream) -> ClientHandler {
        ClientHandler {
            buffer_reader: BufReader::new(
                client_socket
                    .try_clone()
                    .expect("Failed to create client BufReader")),

            buffer_writer: BufWriter::new(
                client_socket
                    .try_clone()
                    .expect("Failed to create client BufWriter")),

            username: "".to_string(),

            client_name: client_socket.peer_addr().unwrap().to_string(),
        }
    }

    pub unsafe fn run(mut self) -> Result<(), std::io::Error> {
        let mut buf = [0u8; 15000];
        loop {
            let amt = self.buffer_reader.read(&mut buf)?;
            let message = String::from_utf8_lossy(&buf[..amt]);
            debug!("Received {} from {}", message, self.client_name);
            self.send_to_other_clients(&buf[..amt]);
            buf = [0u8; 15000];
        }
    }

    fn send_to_other_clients(&mut self, message: &[u8]) {
        for client in server::CLIENT_HANDLERS.lock().unwrap().iter_mut() {
            if client.client_name == self.client_name {
                trace!("Skipping sending message to {}", client.client_name);
                continue;
            }
            trace!("Sending message to client: {}", client.client_name);
            client.buffer_writer.write(message).expect("Failed to send message");
            client.buffer_writer.flush().expect("Failed to flush message");
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            buffer_reader: BufReader::new(
                self.buffer_reader.get_ref().try_clone().expect("Failed to create client BufReader")),
            buffer_writer: BufWriter::new(
                self.buffer_writer.get_ref().try_clone().expect("Failed to create client BufWriter")),
            username: self.username.clone(),
            client_name: self.client_name.clone(),
        }
    }
}

impl std::fmt::Display for ClientHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.client_name)
    }
}