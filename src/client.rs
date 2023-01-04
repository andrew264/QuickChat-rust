use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;
use std::thread::JoinHandle;

use log::{error, trace};

use crate::message::Message;
use crate::message_types::MessageType;

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
        let mut msg = String::new();
        let username = self.username.clone();

        // send join message
        let mut join_message = Message::new(username.clone(),
                                            "joined the chat".to_string(),
                                            None);
        join_message.set_type(MessageType::Join);
        self.send_message(join_message);

        loop {
            msg.clear();
            std::io::stdin()
                .read_line(&mut msg)
                .expect("Failed to read line")
                .to_string();
            msg = msg.trim().to_string();

            let msg: Message = Message::new(username.clone(),
                                            msg.clone(),
                                            None);
            self.send_message(msg);
        }
    }

    fn set_username(&mut self) {
        println!("Enter username: ");
        let mut username = String::new();
        std::io::stdin()
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
        if self.check_username_availability(username.clone()) {
            println!("Username set to {}", self.username);
            return;
        } else {
            error!("Username is already taken");
            self.set_username();
            return;
        }
    }

    fn check_username_availability(&mut self, username: String) -> bool {
        let mut msg = Message::new(username.to_string(),
                                   String::new(),
                                   None);

        msg.set_type(MessageType::SetUsername);
        self.send_message(msg);

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


    fn receive_from_server(&mut self) -> JoinHandle<()> {
        let mut buf = [0u8; 15000];
        let mut buffer_reader = BufReader::new(
            self.server_socket.try_clone().expect("Failed to create client BufReader"));

        trace!("Starting receive_from_server thread");
        let (sender, receiver) = sync_channel(0);
        self.receiver = Some(receiver);
        thread::Builder::new().name("receive_from_server".to_string()).spawn(move || {
            trace!("receive_from_server thread started");
            loop {
                let amt = buffer_reader.read(&mut buf)
                    .expect("Failed to read from server");
                let message = Message::new_from_bytes(&buf[..amt]);
                buf = [0u8; 15000];
                trace!("Received {}", message.to_string());
                match message.get_type() {
                    MessageType::Message | MessageType::Join | MessageType::Leave => {
                        println!("{}", message.to_string());
                    }
                    MessageType::UsernameAvailable | MessageType::UsernameTaken => {
                        sender.send(message).unwrap();
                        trace!("Sent message to main thread");
                    }
                    _ => {
                        error!("Unknown message type {}", message.get_type());
                    }
                }
            }
        }).unwrap()
    }

    fn send_message(&mut self, msg: Message) {
        trace!("Sending {}", msg.to_string());

        self.buffer_writer.write(&*msg.to_bytes())
            .expect("Failed to send message");
        self.buffer_writer.flush()
            .expect("Failed to flush message");
    }
}