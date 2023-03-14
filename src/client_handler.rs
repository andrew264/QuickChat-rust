use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use log::{debug, error, trace};
use serde_json::json;

use crate::message::Message;
use crate::message_types::MessageType;
use crate::server;

pub struct ClientHandler {
    buffer_reader: BufReader<TcpStream>,
    buffer_writer: BufWriter<TcpStream>,
    username: String,
    pub(crate) client_name: String,
}

pub(crate) type Messages = Arc<Mutex<Vec<Message>>>;

lazy_static! {
    static ref MESSAGES: Messages = Arc::new(Mutex::new(Vec::new()));
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
            debug!("Received {} bytes from {}", amt, self.client_name);
            let messages = Message::from_bytes(&buf[..amt]);
            debug!("Received {} messages", messages.len());
            for message in messages {
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
                    MessageType::FetchMessages => {
                        self.sync_messages();
                    }
                    _ => {
                        error!("Unknown message type {}", message.get_type());
                    }
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
        let msg = message.clone();
        let msg_arr = json!([msg]).to_string().into_bytes();

        let _ = match self.buffer_writer
            .write(&msg_arr)
            .and_then(|_| self.buffer_writer.flush()) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to flush {}'s buffer: {}", self.client_name, e);
            }
        };
    }

    fn send_to_other_clients(&mut self, message: &Message) {
        MESSAGES.lock().unwrap().push(message.clone());
        for client in server::CLIENT_HANDLERS.lock().unwrap().iter_mut() {
            if self == client {
                trace!("Skipped sending message to {}", client.client_name);
                continue;
            }
            trace!("Sending message to client: {}", client.client_name);
            client.send_to_client(&message);
        }
    }

    fn sync_messages(&mut self) {
        debug!("Syncing messages with {}", self.client_name);
        let messages = MESSAGES.lock().unwrap().clone();
        // json array of messages
        let msg_arr = json!(messages.to_vec());
        debug!("Sending {} messages to {}", messages.len(), self.client_name);
        trace!("Sending {}", msg_arr.to_string());
        let _ = match self.buffer_writer
            .write(msg_arr.to_string().as_bytes())
            .and_then(|_| self.buffer_writer.flush()) {
            Ok(_) => {
                debug!("Sent {} messages to {}", messages.len(), self.client_name);
            }
            Err(e) => {
                error!("Failed to flush {}'s buffer: {}", self.client_name, e);
            }
        };

        trace!("Synced messages {} with {}", messages.len(), self.client_name);
        self.send_to_client(&Message::builder()
            .message_type(MessageType::ClearToSend)
            .build());
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
