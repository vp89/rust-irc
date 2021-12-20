use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use chrono::Utc;
use uuid::Uuid;

use crate::{replies::Reply, ChannelContext, ConnectionContext};

pub fn handle_join(
    server_host: &str,
    nick: &str,
    client: &str,
    conn_context: &ConnectionContext,
    channels: &mut HashMap<String, ChannelContext>,
    connections: &HashMap<Uuid, ConnectionContext>,
    channels_to_join: &Option<Vec<String>>,
) -> Option<HashMap<Uuid, Vec<Reply>>> {
    let channels_to_join = match channels_to_join {
        Some(c) => c,
        None => {
            return Some(HashMap::<_, _>::from_iter([(
                conn_context.connection_id,
                vec![Reply::ErrNeedMoreParams {
                    server_host: server_host.to_owned(),
                    nick: nick.to_owned(),
                    command: "JOIN".to_string(),
                }],
            )]));
        }
    };

    let now = Utc::now();
    let mut map = HashMap::new();

    for channel in channels_to_join {
        match channels.get_mut(channel) {
            Some(c) => {
                c.members.insert(conn_context.connection_id);
            }
            None => {
                let mut members = HashSet::new();
                members.insert(conn_context.connection_id);

                channels.insert(
                    channel.clone(),
                    // TODO this probably won't be right eventually
                    // if there needs to be persisted channel ownership?
                    ChannelContext { members },
                );
            }
        }

        let mut replies = vec![
            Reply::Join {
                client: client.to_string(),
                channel: channel.clone(),
            },
            // TODO persist the channel metadata
            Reply::Topic {
                server_host: server_host.to_string(),
                nick: nick.to_string(),
                channel: channel.clone(),
                topic: "foobar topic".to_string(),
            },
            Reply::TopicWhoTime {
                server_host: server_host.to_string(),
                channel: channel.clone(),
                nick: nick.to_string(),
                set_at: now,
            },
        ];

        let chan_ctx = match channels.get(channel) {
            Some(c) => c,
            None => {
                println!(
                    "Unable for {} to join topic {} as user list not found for it",
                    nick, channel
                );
                continue;
            }
        };

        let mut channel_users = vec![];

        for member in &chan_ctx.members {
            let other_user = match connections.get(member) {
                Some(c) => c,
                None => {
                    println!(
                        "Connection context not found for matched channel user {}",
                        member
                    );
                    continue;
                }
            };

            if let Some(e) = &other_user.nick {
                channel_users.push(e.clone())
            }
        }

        replies.push(Reply::Nam {
            server_host: server_host.to_string(),
            nick: nick.to_string(),
            channel: channel.clone(),
            channel_users,
        });

        replies.push(Reply::EndOfNames {
            server_host: server_host.to_string(),
            nick: nick.to_string(),
            channel: channel.clone(),
        });

        map.insert(conn_context.connection_id, replies);

        for member in &chan_ctx.members {
            if member == &conn_context.connection_id {
                continue;
            }

            let other_user = match connections.get(member) {
                Some(c) => c,
                None => {
                    println!(
                        "Connection context not found for matched channel user {}",
                        member
                    );
                    continue;
                }
            };

            map.insert(
                other_user.connection_id,
                vec![Reply::Join {
                    client: client.to_string(),
                    channel: channel.clone(),
                }],
            );
        }
    }

    Some(map)
}
