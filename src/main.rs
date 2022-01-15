mod channels;
mod client_listener;
mod client_sender;
mod error;
mod handlers;
mod message_handler;
mod message_parsing;
mod replies;
mod result;
mod server;
mod settings;
mod util;

use chrono::{DateTime, Utc};
use settings::Settings;
use std::collections::HashSet;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::{
    signal,
    sync::mpsc::{self},
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> io::Result<()> {
    let settings = Settings::new().unwrap();
    let (server_shutdown_sender, shutdown_receiver) = mpsc::channel::<()>(1);

    let server_task = tokio::spawn(async move {
        if let Err(e) = server::run(&settings, shutdown_receiver).await
        {
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

#[derive(Clone)]
pub struct ServerContext {
    pub start_time: DateTime<Utc>,
    pub server_host: String,
    pub version: String,
    pub ping_frequency: Duration,
    pub motd_lines: Vec<String>,
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
