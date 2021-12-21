use std::collections::HashMap;

use uuid::Uuid;

use crate::{replies::Reply, ChannelContext, ConnectionContext};

pub fn handle_privmsg(
    server_host: &str,
    nick: &str,
    channel: &Option<String>,
    message: &Option<String>,
    conn_context: &ConnectionContext,
    channels: &HashMap<String, ChannelContext>,
    connections: &HashMap<Uuid, ConnectionContext>,
) -> Option<HashMap<Uuid, Vec<Reply>>> {
    let channel = match channel {
        Some(c) => c,
        None => {
            let mut map = HashMap::new();
            map.insert(
                conn_context.connection_id,
                vec![Reply::ErrNeedMoreParams {
                    server_host: server_host.to_string(),
                    nick: nick.to_string(),
                    command: "PRIVMSG".to_string(),
                }],
            );
            return Some(map);
        }
    };

    let message = match message {
        Some(m) => m,
        None => {
            let mut map = HashMap::new();
            map.insert(
                conn_context.connection_id,
                vec![Reply::ErrNeedMoreParams {
                    server_host: server_host.to_string(),
                    nick: nick.to_string(),
                    command: "PRIVMSG".to_string(),
                }],
            );
            return Some(map);
        }
    };

    let mut map = HashMap::new();
    let channel_ctx = match channels.get(channel) {
        Some(c) => c,
        None => {
            println!("Unable to send message to channel {}, not found", channel);
            return None;
        }
    };

    for member in &channel_ctx.members {
        if member == &conn_context.connection_id {
            continue;
        }

        let connected_member = match connections.get(member) {
            Some(conn) => conn,
            None => {
                println!("Unable to find member {} in connections map", member);
                continue;
            }
        };

        map.insert(
            connected_member.connection_id,
            vec![Reply::PrivMsg {
                nick: Some(nick.to_string()),
                user: conn_context.user.clone(),
                client_host: conn_context.client_host,
                channel: channel.to_string(),
                message: message.to_string(),
            }],
        );
    }

    Some(map)
}
