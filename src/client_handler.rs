use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

use log::{debug, error, trace};

use crate::server;
use crate::message::Message;
use crate::message_types::MessageType;

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

            username: String::new(),

            client_name: client_socket.peer_addr().unwrap().to_string(),
        }
    }

    pub unsafe fn run(mut self) {
        let mut buf = [0u8; 15000];
        loop {
            let result_amt = self.buffer_reader.read(&mut buf);
            let amt = match result_amt {
                Ok(amt) => amt,
                Err(_) => {
                    debug!("Failed to read from client. Probably disconnected.");
                    break;
                }
            };
            if buf[..amt].is_empty() {
                debug!("Client {} disconnected.", self.client_name);
                break;
            }
            let message = Message::from_bytes(&buf[..amt]);
            trace!("Received {}", message.to_string());
            match message.get_type() {
                MessageType::Message | MessageType::Join | MessageType::Leave => {
                    self.send_to_other_clients(&message);
                }
                MessageType::SetUsername => {
                    let username = message.get_username().to_string();
                    if self.is_username_available(username.to_string()) {
                        self.set_username(&username);
                        trace!("Username set to {}", self.username);
                        self.send_to_client({
                            &Message::builder()
                                .username(&username)
                                .message_type(MessageType::UsernameAvailable)
                                .build()
                        }
                        );
                    } else {
                        trace!("Username {} is not available", username);
                        self.send_to_client(&Message::builder()
                            .username(&self.username)
                            .message_type(MessageType::UsernameTaken)
                            .build());
                    }
                }
                _ => {
                    error!("Unknown message type {}", message.get_type());
                }
            }
            buf = [0u8; 15000];
        }
        self.send_to_other_clients(&Message::builder()
            .username(&self.username)
            .message_type(MessageType::Leave)
            .build());
        server::remove_client(&self.client_name);
        drop(self);
    }

    fn send_to_client(&mut self, message: &Message) {
        trace!("Sending {}", message.to_string());
        let _ = match self.buffer_writer.write(&message.to_bytes()) {
            Ok(_) => {
                let _ = match self.buffer_writer.flush() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to flush {}'s buffer: {}", self.client_name, e);
                    }
                };
            }
            _ => {}
        };
    }

    fn send_to_other_clients(&mut self, message: &Message) {
        for client in server::CLIENT_HANDLERS.lock().unwrap().iter_mut() {
            if self == client {
                trace!("Skipped sending message to {}", client.client_name);
                continue;
            }
            trace!("Sending message to client: {}", client.client_name);
            client.send_to_client(&message);
        }
    }

    fn is_username_available(&self, username: String) -> bool {
        for client in server::CLIENT_HANDLERS.lock().unwrap().iter() {
            trace!("Checking username {} against {}", username, client.username);
            trace!("{}", client);
            if client.username.eq(&username) {
                trace!("Username {} is not available", username);
                return false;
            }
        }
        trace!("Username {} is available", username);
        true
    }

    fn set_username(&mut self, username: &String) {
        self.username = username.clone();
        for client in server::CLIENT_HANDLERS.lock().unwrap().iter_mut() {
            if self == client {
                client.username = username.clone();
            }
        }
    }
}

impl std::fmt::Display for ClientHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Client: {}\t Username: {}", self.client_name, self.username)
    }
}

impl Clone for ClientHandler {
    fn clone(&self) -> Self {
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

impl PartialEq<Self> for ClientHandler {
    fn eq(&self, other: &Self) -> bool {
        self.client_name == other.client_name
    }
}

impl Eq for ClientHandler {}
