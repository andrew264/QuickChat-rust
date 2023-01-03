use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::thread::JoinHandle;

use log::{trace};

pub(crate) struct Client {
    username: String,
    server_socket: TcpStream,
}

impl Client {
    pub(crate) fn new(server_socket: TcpStream) -> Client {
        Client {
            username: Client::get_username(),
            server_socket,
        }
    }

    pub(crate) fn run(&mut self) {
        trace!("Client is running");
        let mut msg = String::new();
        self.receive_from_server();

        let mut buffer_writer = BufWriter::new(
            self.server_socket
                .try_clone()
                .expect("Failed to create client BufWriter"));

        loop {
            msg.clear();
            std::io::stdin().read_line(&mut msg).expect("Failed to read line");
            let msg = format!("{}: {}", self.username, msg);
            buffer_writer.write(msg.as_bytes()).expect("Failed to send message");
            buffer_writer.flush().expect("Failed to flush message");
        }
    }

    fn get_username() -> String {
        println!("Enter username: ");
        let mut username = String::new();
        std::io::stdin()
            .read_line(&mut username)
            .unwrap()
            .to_string();
        username.to_string()
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
                let message = String::from_utf8_lossy(&buf[..amt]);
                if message == "exit" {
                    break;
                }
                println!("{}", message);
                buf = [0u8; 15000];
            }
        }).unwrap()
    }
}