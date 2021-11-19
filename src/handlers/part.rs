use std::{collections::HashMap, iter::FromIterator};

use uuid::Uuid;

use crate::{replies::Reply, ChannelContext, ConnectionContext};

pub fn handle_part(
    server_host: &str,
    nick: &str,
    client: &str,
    conn_context: &ConnectionContext,
    channels: &mut HashMap<String, ChannelContext>,
    channels_to_leave: &[String],
) -> Option<HashMap<Uuid, Vec<Reply>>> {
    if channels_to_leave.is_empty() {
        return Some(HashMap::<_, _>::from_iter([(
            conn_context.connection_id,
            vec![Reply::ErrNeedMoreParams {
                server_host: server_host.to_owned(),
                nick: nick.to_owned(),
                command: "PART".to_string(),
            }],
        )]));
    }

    let mut map = HashMap::new();
    let mut replies_to_user = vec![];

    for channel in channels_to_leave {
        match channels.get_mut(channel) {
            Some(ctx) => {
                if !ctx.members.contains(&conn_context.connection_id) {
                    replies_to_user.push(Reply::ErrNotOnChannel {
                        server_host: server_host.to_owned(),
                        channel: channel.to_owned(),
                    });
                    continue;
                }

                for member in &ctx.members {
                    if member == &conn_context.connection_id {
                        continue;
                    }

                    map.insert(
                        *member,
                        vec![Reply::Part {
                            client: client.to_owned(),
                            channel: channel.to_owned(),
                        }],
                    );
                }

                ctx.members.remove(&conn_context.connection_id);
            }
            None => {
                replies_to_user.push(Reply::ErrNoSuchChannel {
                    server_host: server_host.to_owned(),
                    channel: channel.to_owned(),
                });
            }
        }
    }

    Some(map)
}
