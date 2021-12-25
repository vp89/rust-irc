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
use std::time::Duration;
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, Receiver},
};
use uuid::Uuid;

use crate::message_parsing::{ClientToServerCommand, ClientToServerMessage, ReplySender};

#[tokio::main]
async fn main() -> io::Result<()> {
    let server_host = "localhost".to_string();
    let context = ServerContext {
        start_time: Utc::now(),
        server_host: server_host.clone(),
        version: "0.0.1".to_string(),
        ping_frequency: Duration::from_secs(60),
    };

    println!("Starting server on {}:6667", server_host);

    let listener = TcpListener::bind(format!("{}:6667", server_host)).await?;
    let mut listener_handles = vec![];
    let mut sender_handles = vec![];

    // TODO capacity is arbitrary, whats a good value?
    let (sender_channel, mut receiver_channel) = mpsc::channel(10);
    let server_context = context.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::run_server::<Receiver<ClientToServerMessage>>(
            &server_context,
            &mut receiver_channel,
        )
        .await
        {
            println!("Error returned from server worker {:?}", e);
        }
    });

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let server_context = context.clone();
                let cloned_server_sender_channel_listener = sender_channel.clone();
                let cloned_server_sender_channel_sender = sender_channel.clone();

                // pass this around in messages to grab details about this connection/user
                let connection_id = Uuid::new_v4();
                // TODO capacity is arbitrary, whats a good value?
                let (client_sender_channel, client_receiver_channel) = mpsc::channel(10);
                let cloned_client_sender_channel = client_sender_channel.clone();
                let cloned_client_sender_channel_2 = client_sender_channel.clone();

                let addr = stream.peer_addr().ok();
                let (mut read_handle, mut write_handle) = stream.into_split();

                sender_handles.push(tokio::spawn(async move {
                    if let Err(e) = client_sender::run_sender(
                        client_receiver_channel,
                        &mut write_handle,
                        &connection_id,
                    )
                    .await
                    {
                        println!("Error returned from client sender {:?}", e)
                    }

                    write_handle.forget();
                }));

                listener_handles.push(tokio::spawn(async move {
                    // TODO There is no timeout on the read handle but we need the listener to stop after some seconds
                    // figure out an alternate here
                    /*
                    if let Err(e) = stream.set_read_timeout(Some(server_context.ping_frequency)) {
                        println!("Error setting read timeout {:?}", e);
                        return;
                    }
                    */

                    if let Err(e) = cloned_server_sender_channel_listener
                        .send(ClientToServerMessage {
                            source: None,
                            command: ClientToServerCommand::Connected {
                                sender: ReplySender(cloned_client_sender_channel),
                                client_ip: addr,
                            },
                            connection_id,
                        })
                        .await
                    {
                        println!("Error sending connection initialization message {:?}", e)
                    };

                    if let Err(e) = client_listener::run_listener(
                        &connection_id,
                        &mut read_handle,
                        cloned_server_sender_channel_listener,
                        server_context,
                        cloned_client_sender_channel_2,
                    )
                    .await
                    {
                        println!("Error returned from client listener {:?}", e)
                    }

                    if let Err(e) = cloned_server_sender_channel_sender
                        .send(ClientToServerMessage {
                            source: None,
                            command: ClientToServerCommand::Disconnected,
                            connection_id,
                        })
                        .await
                    {
                        println!(
                            "Error sending disconnected message for connection_id {} {:?}",
                            connection_id, e
                        );
                    }

                    // TODO -> IS THIS RIGHT??
                    drop(read_handle);
                }));
            }
            // TODO graceful shutdown / error accepting conn shouldn't kill the program
            Err(e) => {
                println!("Error connecting {:?}", e);
                break;
            }
        };
    }

    for handle in listener_handles {
        if let Err(e) = handle.await {
            println!("Error joining client listener thread handle {:?}", e);
        }
    }

    for handle in sender_handles {
        if let Err(e) = handle.await {
            println!("Error joining client sender thread handle {:?}", e);
        }
    }

    if let Err(e) = server_handle.await {
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
