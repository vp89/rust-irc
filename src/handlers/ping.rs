use std::collections::HashMap;

use uuid::Uuid;

use crate::{replies::Reply, ConnectionContext};

pub fn handle_ping(
    server_host: &str,
    nick: &str,
    token: &Option<String>,
    conn_context: &ConnectionContext,
) -> Option<HashMap<Uuid, Vec<Reply>>> {
    let mut map = HashMap::new();

    let reply = match token {
        None => Reply::ErrNeedMoreParams {
            server_host: server_host.to_string(),
            nick: nick.to_string(),
            command: "PING".to_string(),
        },
        Some(token) => Reply::Pong {
            server_host: server_host.to_string(),
            token: token.to_string(),
        },
    };

    let replies = vec![reply];

    map.insert(conn_context.connection_id, replies);

    Some(map)
}
