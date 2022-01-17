use std::{collections::HashSet, net::SocketAddr, time::Duration};

use chrono::{DateTime, Utc};
use uuid::Uuid;

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
    pub members: HashSet<Uuid>,
}
