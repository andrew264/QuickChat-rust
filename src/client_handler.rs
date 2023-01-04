use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

use log::{error, trace};

use crate::message::Message;
use crate::message_types::MessageType;
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

            username: String::new(),

            client_name: client_socket.peer_addr().unwrap().to_string(),
        }
    }

    pub unsafe fn run(mut self) -> Result<(), std::io::Error> {
        let mut buf = [0u8; 15000];
        loop {
            let amt = self.buffer_reader.read(&mut buf)?;
            let message = Message::new_from_bytes(&buf[..amt]);
            trace!("Received {}", message.to_string());
            match message.get_type() {
                MessageType::Message | MessageType::Join | MessageType::Leave => {
                    self.send_to_other_clients(message);
                }
                MessageType::SetUsername => {
                    let username = message.get_username().to_string();
                    if self.is_username_available(username.to_string()) {
                        self.username = username;
                        self.set_username(self.username.clone());
                        trace!("Username set to {}", self.username);
                        self.send_to_client({
                            let mut x = Message::new(self.username.clone(),
                                                     "Username set".to_string(),
                                                     None);
                            x.set_type(MessageType::UsernameAvailable);
                            x
                        }
                        );
                    } else {
                        trace!("Username {} is not available", username);
                        self.send_to_client(Message::new_from_type(MessageType::UsernameTaken));
                    }
                }
                _ => {
                    error!("Unknown message type {}", message.get_type());
                }
            }
            buf = [0u8; 15000];
        }
    }

    fn send_to_client(&mut self, message: Message) {
        trace!("Sending {}", message.to_string());
        self.buffer_writer.write(&message.to_bytes())
            .expect("Failed to write to buffer");
        self.buffer_writer.flush()
            .expect("Failed to flush buffer");
    }

    fn send_to_other_clients(&mut self, message: Message) {
        for client in server::CLIENT_HANDLERS.lock().unwrap().iter_mut() {
            if client.client_name == self.client_name {
                trace!("Skipping sending message to {}", client.client_name);
                continue;
            }
            trace!("Sending message to client: {}", client.client_name);
            client.buffer_writer.write(&*message.to_bytes()).expect("Failed to send message");
            client.buffer_writer.flush().expect("Failed to flush message");
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

    fn set_username(&mut self, username: String) {
        for client in server::CLIENT_HANDLERS.lock().unwrap().iter_mut() {
            if client.client_name == self.client_name {
                client.username = username.clone();
            }
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
        write!(f, "Client: {}\t Username: {}", self.client_name, self.username)
    }
}