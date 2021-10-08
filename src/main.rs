mod message_parsing;
mod replies;
mod server;
mod client_listener;
mod client_sender;

use std::{collections::{HashMap}, net::{Shutdown, TcpListener}, sync::{Arc, Mutex, RwLock}, thread, time::{Duration}};
use chrono::{DateTime, Utc};
use message_parsing::ClientToServerMessage;
use replies::Reply;
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::{Sender};
use uuid::Uuid;

fn main() -> io::Result<()> {
    let host = "localhost".to_string();
    let context = ServerContext {
        start_time: Utc::now(),
        host: host.clone(),
        version: "0.0.1".to_string(),
        ping_frequency: Duration::from_secs(10)
    };

    println!("STARTING SERVER ON {}:6667", host);

    let listener = TcpListener::bind(format!("{}:6667", host))?;
    let mut listener_handles = vec![];
    let mut sender_handles = vec![];

    let (sender_channel, receiver_channel) = mpsc::channel();

    let connections = Arc::new(RwLock::new(HashMap::new()));

    // need to clone the Arc before spawning the thread so that
    // it doesnt take ownership of the original
    let server_connections = connections.clone();
    let server_context = context.clone();
    let server_handle = thread::spawn(move || {
        server::run_server(&server_context, receiver_channel, server_connections);
    });

    for connection_attempt in listener.incoming() {
        let server_context = context.clone();
        let cloned_sender_channel = sender_channel.clone();

        match connection_attempt {
            Ok(stream) => {
                // use this to identity the connection until we finalize the connection handshake?
                let connection_uuid = Uuid::new_v4();
                let (client_sender_channel, client_receiver_channel) = mpsc::channel();

                let context = ConnectionContext {
                    uuid: connection_uuid,
                    client_sender_channel: Mutex::new(client_sender_channel),
                    nick: "".to_owned(),
                    client: "".to_owned()
                };

                connections
                    .write()
                    .unwrap() // TODO remove unwrap
                    .insert(connection_uuid.clone(), context);

                let mut write_handle = stream.try_clone()?;
                sender_handles.push(
                    thread::spawn(move || {
                        if let Err(e) = client_sender::run_sender(client_receiver_channel, &mut write_handle) {
                            println!("ERROR FROM CLIENT SENDER {:?}", e)
                        }
                    })
                );

                listener_handles.push(
                    thread::spawn(move || {
                        if let Err(e) = stream.set_read_timeout(Some(server_context.ping_frequency)) {
                            println!("ERROR SETTING READ TIMEOUT {:?}", e);
                            return;
                        }

                        if let Err(e) = client_listener::run_listener(&connection_uuid, &stream, cloned_sender_channel, server_context) {
                            println!("ERROR FROM CLIENT LISTENER {:?}", e)
                        }
                        
                        if let Err(e) = stream.shutdown(Shutdown::Both) {
                            println!("ERROR SHUTTING DOWN SOCKET {:?}", e)
                        }

                        println!("FINISHED HANDLING THIS CONNECTION")
                    }));
            },
            Err(e) => println!("ERROR CONNECTING {:?}", e),
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
    pub host: String,
    pub version: String,
    pub ping_frequency: Duration
}

pub struct ConnectionContext {
    pub uuid: Uuid,
    pub client: String,
    pub nick: String,
    pub client_sender_channel: Mutex<Sender<Reply>>
}
