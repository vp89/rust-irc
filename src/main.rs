mod channels;
mod client_listener;
mod client_sender;
mod context;
mod error;
mod handlers;
mod message_handler;
mod message_parsing;
mod replies;
mod result;
mod server;
mod settings;
mod util;

use settings::Settings;
use std::io;
use tokio::{
    signal,
    sync::mpsc::{self},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let settings = Settings::new().unwrap();
    let (server_shutdown_sender, shutdown_receiver) = mpsc::channel::<()>(1);

    let server_task = tokio::spawn(async move {
        if let Err(e) = server::run(&settings, shutdown_receiver).await {
            println!("Error received from server {:?}", e);
        };
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(e) => {
            println!("Unable to listen to shutdown signal {:?}", e);
        }
    }

    println!("Sending shutdown signal");

    if server_shutdown_sender.send(()).await.is_err() {
        println!("Unable to propagate shutdown signal to the rest of the program");
    }

    server_task.await?;

    Ok(())
}
