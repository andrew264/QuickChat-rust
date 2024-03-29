use std::{io, thread};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread::JoinHandle;

use lazy_static::lazy_static;
use log::{debug, error, trace};
use serde_json::json;

use crate::client_handler::Messages;
use crate::message::Message;
use crate::message_types::MessageType;

lazy_static! {
    static ref MESSAGES: Messages = Arc::new(Mutex::new(Vec::new()));
}

pub(crate) struct Client {
    username: String,
    server_socket: TcpStream,
    buffer_writer: BufWriter<TcpStream>,
    receiver: Option<Receiver<Message>>,
}

impl Client {
    pub(crate) fn new(server_socket: TcpStream) -> Client {
        let buffer_writer = BufWriter::new(
            server_socket
                .try_clone()
                .expect("Failed to create client BufWriter"));


        Client {
            username: "".to_string(),
            server_socket,
            buffer_writer,
            receiver: None,
        }
    }

    pub(crate) fn run(&mut self) {
        trace!("Client is running");
        self.receive_from_server();
        self.set_username();
        let username = self.username.clone();

        // fetch messages from server
        self.fetch_messages();

        // send join message
        self.send_message({
            &Message::builder()
                .username(&username)
                .message_type(MessageType::Join)
                .build()
        });

        let mut msg = String::new();
        loop {
            msg.clear();
            io::stdin()
                .read_line(&mut msg)
                .expect("Failed to read line")
                .to_string();
            msg = msg.trim().to_string();
            if msg == "exit" {
                self.server_socket.shutdown(std::net::Shutdown::Both).unwrap();
                break;
            }

            self.send_message(&Message::builder()
                .username(&username)
                .message(&msg)
                .build());
        }
    }

    fn set_username(&mut self) {
        print!("Enter username: ");
        io::stdout().flush().expect("Failed to flush stdout");
        let mut username = String::new();
        io::stdin()
            .read_line(&mut username)
            .unwrap()
            .to_string();
        username = username.trim().to_string();
        if username.is_empty() {
            error!("Username cannot be empty");
            self.set_username();
            return;
        }
        if username.contains(" ") {
            error!("Username cannot contain spaces");
            self.set_username();
            return;
        }
        if username.len() < 3 || username.len() > 20 {
            error!("Username cannot be less than 3 or more than 20 characters");
            self.set_username();
            return;
        }
        if username.to_uppercase() == "SERVER" {
            error!("Username cannot be SERVER");
            self.set_username();
            return;
        }
        if self.check_username_availability(&username) {
            println!("Username set to {}", self.username);
            return;
        } else {
            error!("Username is already taken");
            self.set_username();
            return;
        }
    }

    fn check_username_availability(&mut self, username: &String) -> bool {
        self.send_message(
            &Message::builder()
                .username(username)
                .message_type(MessageType::SetUsername)
                .build()
        );

        while self.receiver.is_none() {
            trace!("Waiting for response from server");
            thread::sleep(std::time::Duration::from_millis(100));
        }
        let received_msg = self.receiver.as_ref().unwrap().recv().unwrap();

        match received_msg.get_type() {
            MessageType::UsernameAvailable => {
                self.username = received_msg.get_username();
                true
            }
            MessageType::UsernameTaken => {
                false
            }
            _ => {
                error!("Unexpected message type");
                false
            }
        }
    }

    fn fetch_messages(&mut self) {
        // fetch messages from server
        self.send_message(
            &Message::builder()
                .username(&self.username)
                .message_type(MessageType::FetchMessages)
                .build()
        );
        // wait for messages to be fetched
        while self.receiver.is_none() {
            trace!("Waiting for response from server");
            thread::sleep(std::time::Duration::from_millis(100));
        }
        let received_message = self.receiver.as_ref().unwrap().recv().unwrap();

        if received_message.get_type() == MessageType::ClearToSend {
            return;
        } else {
            error!("Unexpected message type {}", received_message.to_string());
            exit(1);
        }
    }


    fn receive_from_server(&mut self) -> JoinHandle<()> {
        let mut buf = [0u8; 15000];
        let mut buffer_reader = BufReader::new(
            self.server_socket.try_clone()
                .expect("Failed to create client BufReader"));

        trace!("Starting receive_from_server thread");
        let (sender, receiver) = sync_channel(0);
        self.receiver = Some(receiver);
        thread::Builder::new()
            .name("Message Receving Thread".to_string())
            .spawn(move || {
                trace!("Message Receving Thread started");
                loop {
                    let result_amt = buffer_reader.read(&mut buf);
                    let amt = match result_amt {
                        Ok(amt) => amt,
                        Err(_) => {
                            debug!("Socket is closed, exiting now...");
                            exit(0);
                        }
                    };

                    let messages = Message::from_bytes(&buf[..amt]);
                    MESSAGES.lock().unwrap().extend(messages.clone().into_iter());
                    buf = [0u8; 15000];
                    for message in messages {
                        trace!("Received {}", message.to_string());
                        match message.get_type() {
                            MessageType::Message | MessageType::Join | MessageType::Leave => {
                                println!("{}", message.to_string());
                            }
                            MessageType::UsernameAvailable | MessageType::UsernameTaken | MessageType::ClearToSend => {
                                sender.send(message).unwrap();
                                trace!("Sent message to main thread");
                            }
                            _ => {
                                error!("Unknown message type {}", message.get_type());
                            }
                        }
                    }
                }
            }).unwrap()
    }

    fn send_message(&mut self, msg: &Message) {
        trace!("Sending {}", msg.to_string());
        MESSAGES.lock().unwrap().push(msg.clone());
        let msg = msg.clone();
        let msg_arr = json!([msg]).to_string().into_bytes();

        let _ = match self.buffer_writer
            .write(&msg_arr)
            .and_then(|_| self.buffer_writer.flush()) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to flush buffer: {}", e);
            }
        };
    }
}