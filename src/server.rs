use std::time::Duration;

use chrono::Utc;
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc, mpsc::Receiver},
};
use uuid::Uuid;

use crate::{
    client_listener, client_sender,
    error::Error::UnableToBindToPort,
    message_handler,
    message_parsing::{ClientToServerCommand, ClientToServerMessage, ReplySender},
    result::Result,
    settings::Settings,
    ServerContext,
};

pub async fn run(settings: &Settings, mut shutdown_receiver: Receiver<()>) -> Result<()> {
    let context = ServerContext {
        start_time: Utc::now(),
        server_host: settings.host.clone(),
        version: "0.0.1".to_string(),
        ping_frequency: Duration::from_secs(settings.ping_frequency_secs),
        motd_lines: settings.motd_lines.clone(),
    };

    println!("Starting server on {}:{}", settings.host, settings.port);

    let listener = TcpListener::bind(format!("{}:{}", settings.host, settings.port))
        .await
        .map_err(|_| UnableToBindToPort(settings.port))?;

    let mut client_listener_tasks = vec![];
    let mut client_sender_tasks = vec![];

    let (message_sender, mut message_receiver) = mpsc::channel(1000);

    let (message_handler_shutdown_sender, message_handler_shutdown_receiver) = mpsc::channel(1);
    let (listener_shutdown_sender, _listener_shutdown_receiver) = broadcast::channel(1000);
    let (sender_shutdown_sender, _sender_shutdown_receiver) = broadcast::channel(1000);

    let server_context = context.clone();

    let message_handler_task = tokio::spawn(async move {
        if let Err(e) = message_handler::run::<Receiver<ClientToServerMessage>>(
            &server_context,
            &mut message_receiver,
            message_handler_shutdown_receiver,
        )
        .await
        {
            println!("Error returned from message handler {:?}", e);
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
                println!("Server received shutdown signal");
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
            break;
        };

        let listener_shutdown_receiver = listener_shutdown_sender.subscribe();
        let sender_shutdown_receiver = sender_shutdown_sender.subscribe();

        client_sender_tasks.push(tokio::spawn(async move {
            if let Err(e) = client_sender::run_sender(
                &connection_id,
                &mut write_handle,
                reply_receiver,
                sender_shutdown_receiver,
            )
            .await
            {
                println!("Error returned from client sender {:?}", e)
            }

            write_handle.forget();
        }));

        client_listener_tasks.push(tokio::spawn(async move {
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
                listener_shutdown_receiver,
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

    // signal to listeners we are shutting down, then await all their tasks
    match listener_shutdown_sender.send(()) {
        Ok(_) => {
            for task in client_listener_tasks {
                println!("Waiting for client listener task to finish");

                if let Err(e) = task.await {
                    println!("Error awaiting client listener task {:?}", e);
                }
            }
        },
        Err(e) => {
            println!("Error sending shutdown message to client listener tasks {:?}", e);
        }
    }

    // signal to senders we are shutting down, then await all their tasks
    match sender_shutdown_sender.send(()) {
        Ok(_) => {
            for task in client_sender_tasks {
                println!("Waiting for client sender task to finish");

                if let Err(e) = task.await {
                    println!("Error awaiting client sender task {:?}", e);
                }
            }
        },
        Err(e) => {
            println!("Error sending shutdown message to client sender tasks {:?}", e);
        }
    }

    // signal to message_handler we are shutting down, then wait
    // everyone should be disconnected by this point

    println!("Waiting for message handler task to finish");

    match message_handler_shutdown_sender.send(()).await {
        Ok(()) => {
            if let Err(e) = message_handler_task.await {
                println!("Error awaiting message handler task {:?}", e);
            }                    
        },
        Err(e) => {
            println!("Error sending shutdown message to message handler task {:?}", e);
        }
    }

    Ok(())
}
