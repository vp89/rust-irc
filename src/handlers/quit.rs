use std::collections::HashMap;
use uuid::Uuid;

use crate::{ChannelContext, ConnectionContext, replies::Reply};

pub fn handle_quit(
    message: &Option<String>,
    channels: &mut HashMap<String, ChannelContext>,
    connections: &HashMap<Uuid, ConnectionContext>,
    connection_id: Uuid,
) -> HashMap<Uuid, Vec<Reply>> {
    let conn_context = match connections.get(&connection_id) {
        Some(c) => c,
        None => {
            return HashMap::new();
        }
    };

    let mut map = HashMap::new();
    let message = match message {
        Some(m) => m.to_string(),
        None => "DEFAULT QUIT MESSAGE TODO".to_string()
    };

    for channel in channels {
        if !channel.1.members.remove(&conn_context.connection_id) {
            println!("UNABLE TO REMOVE {} FROM CHANNEL {}", &conn_context.nick.as_ref().unwrap(), channel.0);
            continue;
        }

        println!("Removed {} from {} channel", conn_context.nick.as_ref().unwrap(), channel.0);
        println!("{} channel members remaining in {}", channel.1.members.len(), channel.0);

        for member in &channel.1.members {
            match connections.get(member) {
                Some(c) => {
                    map.insert(
                        *member,
                        vec![
                            Reply::Quit {
                                connection_id: connection_id.clone(),
                                client_host: conn_context.client_host.clone(),
                                nick: conn_context.nick.clone(),
                                user: conn_context.user.clone(),
                                message: message.to_string()
                            }
                        ]
                    );
                },
                None => { }
            }
        }
    }

    map.insert(
        connection_id,
        vec![
            Reply::Quit {
                connection_id: connection_id.clone(),
                client_host: conn_context.client_host.clone(),
                nick: conn_context.nick.clone(),
                user: conn_context.user.clone(),
                message: message.to_string()
            }
        ]
    );
    
    map
}