use std::{collections::HashMap, net::SocketAddr};

use uuid::Uuid;

use crate::{ChannelContext, ConnectionContext, replies::Reply};

pub fn handle_privmsg(
    server_host: &str,
    nick: &str,
    client: &str,
    channel: &str,
    message: &str,
    conn_context: &ConnectionContext,
    channels: &HashMap<String, ChannelContext>,
    connections: &HashMap<Uuid, ConnectionContext>,
) -> HashMap<Uuid, Vec<Reply>> {
    let mut map = HashMap::new();
    let channel_ctx = match channels.get(channel) {
        Some(c) => c,
        None => {
            println!("Unable to send message to channel {}, not found", channel);
            return map;
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
            }]
        );
    }

    map
}