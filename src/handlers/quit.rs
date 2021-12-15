use std::collections::HashMap;
use uuid::Uuid;

use crate::{replies::Reply, ChannelContext, ConnectionContext};

pub fn handle_quit(
    message: &Option<String>,
    channels: &mut HashMap<String, ChannelContext>,
    connections: &HashMap<Uuid, ConnectionContext>,
    connection_id: Uuid,
) -> Option<HashMap<Uuid, Vec<Reply>>> {
    let conn_context = match connections.get(&connection_id) {
        Some(c) => c,
        None => {
            return None;
        }
    };

    let mut map = HashMap::new();
    let message = match message {
        Some(m) => m.to_string(),
        None => "DEFAULT QUIT MESSAGE TODO".to_string(),
    };

    for channel in channels {
        if !channel.1.members.contains(&conn_context.connection_id) {
            continue;
        }

        // if the quitting user was part of this channel, remove them
        // from the list and then send a QUIT to everyone other user
        // in the channel
        if !channel.1.members.remove(&conn_context.connection_id) {
            println!(
                "UNABLE TO REMOVE {} FROM CHANNEL {}",
                &conn_context.nick.as_ref().unwrap(),
                channel.0
            );
            continue;
        }

        for member in &channel.1.members {
            match connections.get(member) {
                Some(_c) => {
                    map.insert(
                        *member,
                        vec![Reply::Quit {
                            connection_id,
                            client_host: conn_context.client_host,
                            nick: conn_context.nick.clone(),
                            user: conn_context.user.clone(),
                            message: message.to_string(),
                        }],
                    );
                }
                None => {
                    println!(
                        "Connection context not found for user in a channel list {}",
                        member
                    );
                }
            }
        }
    }

    map.insert(
        connection_id,
        vec![Reply::Quit {
            connection_id,
            client_host: conn_context.client_host,
            nick: conn_context.nick.clone(),
            user: conn_context.user.clone(),
            message,
        }],
    );

    Some(map)
}
