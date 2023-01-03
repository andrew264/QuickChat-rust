use std::collections::VecDeque;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, spawn};
use std::time::Duration;

use lazy_static::lazy_static;
use log::{error, info, trace};

use crate::client_handler::ClientHandler;
use crate::server_discovery_thread::DiscoveryThread;

const SERVER_PORT: u16 = 42069;

type ClientHandlers = Arc<Mutex<VecDeque<ClientHandler>>>;

lazy_static! {
    pub(crate) static ref CLIENT_HANDLERS: ClientHandlers = Arc::new(Mutex::new(VecDeque::new()));
}

pub struct Server {
    server_socket: TcpListener,
}

impl Server {
    pub fn new() -> Result<Self, std::io::Error> {
        let server_socket = TcpListener::bind(("0.0.0.0", SERVER_PORT))?;
        // Bind to all interfaces
        info!("Server listening on: {:?}", server_socket.local_addr().unwrap());
        Ok(Self {
            server_socket,
        })
    }

    pub fn run(self) -> Result<(), std::io::Error> {
        // start discovery thread
        let discovery_thread = DiscoveryThread::new().unwrap();
        spawn(move || {
            discovery_thread.run();
        });
        trace!("Discovery thread started");

        for client_socket in self.server_socket.incoming() {
            match client_socket {
                Ok(client_socket) => unsafe {
                    info!("New connection: {}", client_socket.peer_addr().unwrap());
                    let client_handler = ClientHandler::new(client_socket);
                    trace!("New client handler {} created", client_handler);
                    CLIENT_HANDLERS.lock().unwrap().push_back(client_handler.clone());
                    trace!("Client handler added to CLIENT_HANDLERS");
                    thread::Builder::new()
                        .name("Starting ClientHandler Thread ".to_string()
                            + &client_handler.client_name)
                        .spawn(move || {
                            trace!("ClientHandler thread started");
                            client_handler.run().unwrap();
                        })
                        .expect("Failed to spawn client handler thread");
                }
                Err(e) => {
                    error!("Error: {}", e);
                    sleep(Duration::from_secs(1));
                }
            }
        }

        Ok(())
    }
}