use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{replies::Reply, ConnectionContext};

pub fn handle_nick(
    server_host: &str,
    nick: &Option<String>,
    ctx_version: &str,
    &ctx_created_at: &DateTime<Utc>,
    conn_context: &mut ConnectionContext,
) -> HashMap<Uuid, Vec<Reply>> {
    let nick = match nick {
        Some(n) => n,
        None => {
            let mut map = HashMap::new();
            map.insert(
                conn_context.connection_id,
                vec![Reply::ErrNoNickGiven {
                    server_host: server_host.to_owned(),
                }],
            );
            return map;
        }
    };

    let mut map = HashMap::new();
    let mut replies: Vec<Reply> = vec![];

    conn_context.nick = Some(nick.to_string());
    conn_context.client = Some(format!("{}!~{}@localhost", nick, nick));

    replies.push(Reply::Welcome {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
    });
    replies.push(Reply::YourHost {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        version: ctx_version.to_owned(),
    });
    replies.push(Reply::Created {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        created_at: ctx_created_at,
    });
    replies.push(Reply::MyInfo {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        version: ctx_version.to_owned(),
        user_modes: "r".to_string(),
        channel_modes: "i".to_string(),
    });
    replies.push(Reply::Support {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        channel_len: 32,
    });
    replies.push(Reply::LuserClient {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        visible_users: 100,
        invisible_users: 20,
        servers: 1,
    });
    replies.push(Reply::LuserOp {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        operators: 1337,
    });
    replies.push(Reply::LuserUnknown {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        unknown: 7,
    });
    replies.push(Reply::LuserChannels {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        channels: 9999,
    });
    replies.push(Reply::LuserMe {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        clients: 900,
        servers: 1,
    });
    replies.push(Reply::LocalUsers {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        current: 845,
        max: 1000,
    });
    replies.push(Reply::GlobalUsers {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        current: 9832,
        max: 23455,
    });
    replies.push(Reply::StatsDLine {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        connections: 9998,
        clients: 9000,
        received: 99999,
    });
    replies.push(Reply::MotdStart {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
    });
    // TODO proper configurable MOTD
    replies.push(Reply::Motd {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
        line: "Foobar".to_string(),
    });
    replies.push(Reply::EndOfMotd {
        server_host: server_host.to_owned(),
        nick: nick.clone(),
    });

    map.insert(conn_context.connection_id, replies);

    map
}
