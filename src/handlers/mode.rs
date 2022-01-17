use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::{context::ConnectionContext, replies::Reply};

pub fn handle_mode(
    server_host: &str,
    nick: &str,
    channel: &Option<String>,
    conn_context: &ConnectionContext,
) -> Option<HashMap<Uuid, Vec<Reply>>> {
    let mut map = HashMap::new();

    let replies = match channel {
        Some(channel) => {
            vec![
                Reply::ChannelModeIs {
                    server_host: server_host.to_string(),
                    nick: nick.to_string(),
                    channel: channel.to_string(),
                    mode_string: "+mtn1".to_string(),
                    mode_arguments: "100".to_string(),
                },
                Reply::CreationTime {
                    server_host: server_host.to_string(),
                    nick: nick.to_string(),
                    channel: channel.to_string(),
                    created_at: Utc::now(),
                },
            ]
        }
        None => {
            vec![Reply::ErrNeedMoreParams {
                server_host: server_host.to_string(),
                nick: nick.to_string(),
                command: "MODE".to_string(),
            }]
        }
    };

    map.insert(conn_context.connection_id, replies);

    Some(map)
}
