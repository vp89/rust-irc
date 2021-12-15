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
use std::collections::HashSet;
use std::io;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::{
    net::{Shutdown, TcpListener},
    thread,
    time::Duration,
};
use uuid::Uuid;

use crate::message_parsing::{ClientToServerCommand, ClientToServerMessage, ReplySender};

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

    let server_context = context.clone();
    let server_handle = thread::spawn(move || {
        if let Err(e) = server::run_server(&server_context, &receiver_channel) {
            println!("Error returned from server worker {:?}", e);
        }
    });

    for connection_attempt in listener.incoming() {
        let server_context = context.clone();
        let cloned_server_sender_channel_listener = sender_channel.clone();
        let cloned_server_sender_channel_sender = sender_channel.clone();

        match connection_attempt {
            Ok(stream) => {
                // pass this around in messages to grab details about this connection/user
                let connection_id = Uuid::new_v4();
                let (client_sender_channel, client_receiver_channel) = mpsc::channel();
                let cloned_client_sender_channel = client_sender_channel.clone();

                let mut write_handle = stream.try_clone()?;

                sender_handles.push(thread::spawn(move || {
                    if let Err(e) = client_sender::run_sender(
                        client_receiver_channel,
                        &mut write_handle,
                        &connection_id,
                    ) {
                        println!("Error returned from client sender {:?}", e)
                    }

                    if let Err(e) = write_handle.shutdown(Shutdown::Write) {
                        println!("Error shutting down socket {:?}", e)
                    }
                }));

                listener_handles.push(thread::spawn(move || {
                    if let Err(e) = stream.set_read_timeout(Some(server_context.ping_frequency)) {
                        println!("Error setting read timeout {:?}", e);
                        return;
                    }

                    if let Err(e) =
                        cloned_server_sender_channel_listener.send(ClientToServerMessage {
                            source: None,
                            command: ClientToServerCommand::Connected {
                                sender: ReplySender(cloned_client_sender_channel.clone()),
                                client_ip: stream.peer_addr().ok(),
                            },
                            connection_id,
                        })
                    {
                        println!("Error sending connection initialization message {:?}", e)
                    };

                    if let Err(e) = client_listener::run_listener(
                        &connection_id,
                        &stream,
                        cloned_server_sender_channel_listener,
                        server_context,
                    ) {
                        println!("Error returned from client listener {:?}", e)
                    }

                    if let Err(e) =
                        cloned_server_sender_channel_sender.send(ClientToServerMessage {
                            source: None,
                            command: ClientToServerCommand::Disconnected,
                            connection_id,
                        })
                    {
                        println!(
                            "Error sending disconnected message for connection_id {} {:?}",
                            connection_id, e
                        );
                    }

                    if let Err(e) = stream.shutdown(Shutdown::Read) {
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

#[derive(Default)]
pub struct ConnectionContext {
    pub connection_id: Uuid,
    pub client: Option<String>,
    pub nick: Option<String>,
    pub user: Option<String>,
    pub real_name: Option<String>,
    pub client_host: Option<SocketAddr>,
}

pub struct ChannelContext {
    members: HashSet<Uuid>,
}
