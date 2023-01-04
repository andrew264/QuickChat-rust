use std::net::TcpStream;
use std::process::exit;
use std::thread;

use env_logger::{Builder, Target};
use log::{error, info};

mod find_server;
mod server_discovery_thread;
mod server;
mod client_handler;
mod client;
mod message;
mod message_types;

fn main() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let mut ip = find_server::get_ip();
    if ip.is_none() {
        info!("Starting Server");
        let s = server::Server::new().unwrap();
        thread::spawn(move || {
            s.run().unwrap();
        });

        // sleep for 2 seconds
        thread::sleep(std::time::Duration::from_secs(2));
        ip = find_server::get_ip();
        // check if ip is none
        if ip.is_none() {
            error!("Could not find server");
            exit(1);
        }
    }
    info!(
        "Connecting to server: {}", ip.unwrap()
    );
    let server_socket = TcpStream::connect((ip.unwrap(), 42069)).unwrap();
    let mut client = client::Client::new(server_socket);
    client.run();
}
