use std::time::Duration;

use chrono::{Utc};
use tokio::{net::TcpListener, sync::mpsc::{self, Receiver}};
use uuid::Uuid;

use crate::{ServerContext, message_handler, message_parsing::{ClientToServerMessage, ClientToServerCommand, ReplySender}, client_sender, client_listener, result::{Result}, settings::Settings, error::Error::ErrorAcceptingConnection};

pub async fn start_server(settings: &Settings, mut shutdown_receiver: Receiver<()>) -> Result<()> {
    let context = ServerContext {
        start_time: Utc::now(),
        server_host: settings.host.clone(),
        version: "0.0.1".to_string(),
        ping_frequency: Duration::from_secs(settings.ping_frequency_secs),
        motd_lines: settings.motd_lines.clone(),
    };

    println!("Starting server on {}:{}", settings.host, settings.port);

    let listener = TcpListener::bind(format!("{}:{}", settings.host, settings.port)).await.map_err(|e| ErrorAcceptingConnection)?;
    let mut listener_handles = vec![];
    let mut sender_handles = vec![];

    let (sender_channel, mut receiver_channel) = mpsc::channel(1000);
    let server_context = context.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = message_handler::run_message_handler::<Receiver<ClientToServerMessage>>(
            &server_context,
            &mut receiver_channel,
        )
        .await
        {
            println!("Error returned from server worker {:?}", e);
        }
    });

    loop {
        let (stream, _) = tokio::select! {
            res = listener.accept() => match res {
                Ok(res) => res,
                Err(e) => {
                    println!("TODO");
                    break;
                }
            },
            _ = shutdown_receiver.recv() => { break; }
        };

        let server_context = context.clone();
        let cloned_server_sender_channel_listener = sender_channel.clone();
        let cloned_server_sender_channel_sender = sender_channel.clone();

        // pass this around in messages to grab details about this connection/user
        let connection_id = Uuid::new_v4();
        let (client_sender_channel, client_receiver_channel) = mpsc::channel(1000);
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
