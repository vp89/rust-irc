mod channels;
mod client_listener;
mod client_sender;
mod error;
mod handlers;
mod message_parsing;
mod replies;
mod result;
mod message_handler;
mod settings;
mod util;
mod server;

use chrono::{DateTime, Utc};
use settings::Settings;
use std::collections::HashSet;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, Receiver},
    signal
};
use uuid::Uuid;

use crate::message_parsing::{ClientToServerCommand, ClientToServerMessage, ReplySender};

#[tokio::main]
async fn main() -> io::Result<()> {
    let settings = Settings::new().unwrap();
    let (shutdown_sender, mut shutdown_receiver) = mpsc::channel::<()>(1);

    tokio::spawn(async move {
        if let Err(e) = server::start_server(&settings, shutdown_receiver).await {
            println!("TODO TODO TODO");
        };
    });

    match signal::ctrl_c().await {
        Ok(()) => {},
        Err(e) => {
            println!("Unable to listen to shutdown signal {:?}", e);
        }
    }

    // TODO error handling
    shutdown_sender.send(()).await;

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
