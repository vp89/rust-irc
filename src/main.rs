mod channels;
mod client_listener;
mod client_sender;
mod error;
mod handlers;
mod message_parsing;
mod replies;
mod result;
mod server;
mod util;

use chrono::{DateTime, Utc};
use replies::Reply;
use std::io;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::{
    collections::HashMap,
    net::{Shutdown, TcpListener},
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Duration,
};
use uuid::Uuid;

fn main() -> io::Result<()> {
    let server_host = "localhost".to_string();
    let context = ServerContext {
        start_time: Utc::now(),
        server_host: server_host.clone(),
        version: "0.0.1".to_string(),
        ping_frequency: Duration::from_secs(60),
    };

    println!("Starting server on {}:6667", server_host);

    let listener = TcpListener::bind(format!("{}:6667", server_host))?;
    let mut listener_handles = vec![];
    let mut sender_handles = vec![];

    let (sender_channel, receiver_channel) = mpsc::channel();

    let connections = Arc::new(RwLock::new(HashMap::new()));

    // need to clone the Arc before spawning the thread so that
    // it doesnt take ownership of the original
    let server_connections = connections.clone();
    let server_context = context.clone();
    let server_handle = thread::spawn(move || {
        if let Err(e) = server::run_server(&server_context, &receiver_channel, server_connections) {
            println!("Error returned from server worker {:?}", e);
        }
    });

    for connection_attempt in listener.incoming() {
        let server_context = context.clone();
        let cloned_sender_channel = sender_channel.clone();

        match connection_attempt {
            Ok(stream) => {
                // pass this around in messages to grab details about this connection/user
                let connection_uuid = Uuid::new_v4();
                let (client_sender_channel, client_receiver_channel) = mpsc::channel();

                let context = ConnectionContext {
                    uuid: connection_uuid,
                    client_sender_channel: Mutex::new(client_sender_channel),
                    nick: None,
                    client: None,
                    user: None,
                    real_name: None,
                    client_host: stream.peer_addr().ok(),
                };

                let mut writeable_map = match connections.write() {
                    Ok(g) => g,
                    Err(e) => {
                        println!("Unable to acquire write lock on connections map {:?}", e);
                        continue;
                    }
                };
                writeable_map.insert(connection_uuid, context);
                println!("connections map now has {} entries", writeable_map.len());
                let mut write_handle = stream.try_clone()?;
                sender_handles.push(thread::spawn(move || {
                    if let Err(e) =
                        client_sender::run_sender(client_receiver_channel, &mut write_handle)
                    {
                        println!("Error returned from client sender {:?}", e)
                    }
                }));

                listener_handles.push(thread::spawn(move || {
                    if let Err(e) = stream.set_read_timeout(Some(server_context.ping_frequency)) {
                        println!("Error setting read timeout {:?}", e);
                        return;
                    }

                    if let Err(e) = client_listener::run_listener(
                        &connection_uuid,
                        &stream,
                        cloned_sender_channel,
                        server_context,
                    ) {
                        println!("Error returned from client listener {:?}", e)
                    }

                    // TODO sender should shut itself down by receiving a message from server manager?
                    if let Err(e) = stream.shutdown(Shutdown::Both) {
                        println!("Error shutting down socket {:?}", e)
                    }
                }));
            }
            Err(e) => println!("Error connecting {:?}", e),
        };
    }

    for handle in listener_handles {
        if let Err(e) = handle.join() {
            println!("Error joining client listener thread handle {:?}", e);
        }
    }

    for handle in sender_handles {
        if let Err(e) = handle.join() {
            println!("Error joining client sender thread handle {:?}", e);
        }
    }

    if let Err(e) = server_handle.join() {
        println!("Error joining server thread handle {:?}", e);
    }

    Ok(())
}

#[derive(Clone)]
pub struct ServerContext {
    pub start_time: DateTime<Utc>,
    pub server_host: String,
    pub version: String,
    pub ping_frequency: Duration,
}

pub struct ConnectionContext {
    pub uuid: Uuid,
    pub client_sender_channel: Mutex<Sender<Reply>>,
    // TODO remove this?
    pub client: Option<String>,
    pub nick: Option<String>,
    pub user: Option<String>,
    pub real_name: Option<String>,
    pub client_host: Option<SocketAddr>,
}

pub struct ChannelContext {
    members: Vec<Uuid>,
}
