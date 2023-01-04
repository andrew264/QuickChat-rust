use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::thread::JoinHandle;
use crate::message::Message;

use log::{trace};

pub(crate) struct Client {
    username: String,
    server_socket: TcpStream,
    buffer_writer: BufWriter<TcpStream>,
}

impl Client {
    pub(crate) fn new(server_socket: TcpStream) -> Client {

        let buffer_writer = BufWriter::new(
            server_socket
                .try_clone()
                .expect("Failed to create client BufWriter"));

        Client {
            username: Client::get_username(),
            server_socket,
            buffer_writer,
        }
    }

    pub(crate) fn run(&mut self) {
        trace!("Client is running");
        let mut msg = String::new();
        self.receive_from_server();

        loop {
            msg.clear();
            std::io::stdin()
                .read_line(&mut msg)
                .expect("Failed to read line")
                .to_string();
            msg = msg.trim().to_string();

            let msg: Message = Message::new(self.username.clone(),
                                            msg.clone(),
                                            None);
            self.send_message(msg);
        }
    }

    fn get_username() -> String {
        println!("Enter username: ");
        let mut username = String::new();
        std::io::stdin()
            .read_line(&mut username)
            .unwrap()
            .to_string();
        username = username.trim().to_string();
        username
    }

    fn receive_from_server(&mut self) -> JoinHandle<()> {
        let mut buf = [0u8; 15000];
        let mut buffer_reader = BufReader::new(
            self.server_socket.try_clone().expect("Failed to create client BufReader"));

        trace!("Starting receive_from_server thread");
        thread::Builder::new().name("receive_from_server".to_string()).spawn(move || {
            trace!("receive_from_server thread started");
            loop {
                let amt = buffer_reader.read(&mut buf)
                    .expect("Failed to read from server");
                let message = Message::new_from_bytes(&buf[..amt]);
                if message.get_message() == "exit" {
                    break;
                }
                println!("{}", message.to_string());
                buf = [0u8; 15000];
            }
        }).unwrap()
    }

    fn send_message(&mut self, msg: Message){
        trace!("Sending {}", msg.to_string());

        self.buffer_writer.write(&*msg.to_bytes())
            .expect("Failed to send message");
        self.buffer_writer.flush()
            .expect("Failed to flush message");
    }
}