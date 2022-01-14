use std::time::Duration;

use chrono::{Utc};
use tokio::{net::TcpListener, sync::{broadcast, mpsc, mpsc::{Receiver}}};
use uuid::Uuid;

use crate::{ServerContext, message_handler, message_parsing::{ClientToServerMessage, ClientToServerCommand, ReplySender}, client_sender, client_listener, result::{Result}, settings::Settings, error::Error::ErrorAcceptingConnection};

pub async fn start_server(
    settings: &Settings,
    mut shutdown_receiver: Receiver<()>
) -> Result<()> {
    let context = ServerContext {
        start_time: Utc::now(),
        server_host: settings.host.clone(),
        version: "0.0.1".to_string(),
        ping_frequency: Duration::from_secs(settings.ping_frequency_secs),
        motd_lines: settings.motd_lines.clone(),
    };

    println!("Starting server on {}:{}", settings.host, settings.port);

    let listener = TcpListener::bind(format!("{}:{}", settings.host, settings.port)).await.map_err(|_| ErrorAcceptingConnection)?;

    let mut listener_handles = vec![];
    let mut sender_handles = vec![];

    let (message_sender, mut message_receiver) = mpsc::channel(1000);

    let (message_handler_shutdown_sender, message_handler_shutdown_receiver) = mpsc::channel(1);
    let (listener_shutdown_sender, _) = broadcast::channel(1000);
    let (sender_shutdown_sender, _) = broadcast::channel(1000);

    let server_context = context.clone();

    let server_handle = tokio::spawn(async move {
        if let Err(e) = message_handler::run_message_handler::<Receiver<ClientToServerMessage>>(
            &server_context,
            &mut message_receiver,
            message_handler_shutdown_receiver
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
                Err(_) => {
                    println!("TODO");
                    break;
                }
            },
            _ = shutdown_receiver.recv() => {
                break;
            }
        };

        let server_context = context.clone();

        let message_sender = message_sender.clone();

        // pass this around in messages to grab details about this connection/user
        let connection_id = Uuid::new_v4();
        let (reply_sender, reply_receiver) = mpsc::channel(1000);
        
        // given to message handler so it can send replies to this client when needed
        let message_handler_reply_sender = reply_sender.clone();
        
        // used to send reply directly from client listener task for pinging loop
        let client_reply_sender = reply_sender.clone();

        let addr = stream.peer_addr().ok();
        let (mut read_handle, mut write_handle) = stream.into_split();

        if let Err(e) = message_sender
            .send(ClientToServerMessage {
                source: None,
                command: ClientToServerCommand::Connected {
                    sender: ReplySender(message_handler_reply_sender),
                    client_ip: addr,
                },
                connection_id,
            })
            .await
        {
            println!("Error sending connection initialization message {:?}", e);
            break; // TODO is this right?
        };

        let listener_shutdown_receiver = listener_shutdown_sender.subscribe();
        let sender_shutdown_receiver = sender_shutdown_sender.subscribe();

        sender_handles.push(tokio::spawn(async move {
            if let Err(e) = client_sender::run_sender(
                &connection_id,
                &mut write_handle,
                reply_receiver,
                sender_shutdown_receiver
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

            if let Err(e) = client_listener::run_listener(
                server_context,
                &connection_id,
                &mut read_handle,
                &message_sender,
                client_reply_sender,
                listener_shutdown_receiver
            )
            .await
            {
                println!("Error returned from client listener {:?}", e)
            }

            if let Err(e) = message_sender
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

    // signal to listeners we are shutting down, then wait all their tasks
    listener_shutdown_sender.send(()).unwrap(); // TODO

    for handle in listener_handles {
        println!("waiting for listener task to finish");

        if let Err(e) = handle.await {
            println!("Error joining client listener thread handle {:?}", e);
        }
    }

    // signal to senders we are shutting down, then wait all their tasks
    sender_shutdown_sender.send(()).unwrap(); // TODO

    for handle in sender_handles {
        println!("waiting for sender task to finish");

        if let Err(e) = handle.await {
            println!("Error joining client sender thread handle {:?}", e);
        }
    }

    // signal to message_handler we are shutting down, then wait
    // everyone should be disconnected by this point
    message_handler_shutdown_sender.send(()).await.unwrap(); // TODO

    println!("waiting for message handler task to finish");

    if let Err(e) = server_handle.await {
        println!("Error joining server thread handle {:?}", e);
    }

    Ok(())
}
