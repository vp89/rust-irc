use std::collections::HashMap;
use uuid::Uuid;

use crate::{ChannelContext, ConnectionContext, replies::Reply};

pub fn handle_quit(
    message: &Option<String>,
    channels: &mut HashMap<String, ChannelContext>,
    connections: &HashMap<Uuid, ConnectionContext>,
    conn_context: &ConnectionContext,
) -> HashMap<Uuid, Vec<Reply>> {
    let mut map = HashMap::new();
    let message = match message {
        Some(m) => m.to_string(),
        None => "DEFAULT QUIT MESSAGE TODO".to_string()
    };

    for channel in channels {
        if !channel.1.members.remove(&conn_context.connection_id) {
            continue;
        }

        println!("{} channel members remaining in {}", channel.1.members.len(), channel.0);

        for member in &channel.1.members {
            match connections.get(member) {
                Some(c) => {
                    map.insert(
                        *member,
                        vec![
                            Reply::Quit {
                                client_host: c.client_host.clone(),
                                nick: c.nick.clone(),
                                user: c.user.clone(),
                                message: message.to_string()
                            }
                        ]
                    );
                },
                None => { /* TODO */ }
            }
        }
    }
    
    map
}