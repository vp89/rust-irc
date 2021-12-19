use std::collections::HashMap;

use uuid::Uuid;

use crate::{replies::Reply, ConnectionContext};

pub fn handle_user(
    server_host: &str,
    user: &Option<String>,
    realname: &Option<String>,
    conn_context: &mut ConnectionContext,
) -> Option<HashMap<Uuid, Vec<Reply>>> {
    let empty_str = &String::from("");
    let nick = conn_context.nick.as_ref().unwrap_or(empty_str);

    let user = match user {
        None => {
            let mut map = HashMap::new();
            map.insert(
                conn_context.connection_id,
                vec![
                    Reply::ErrNeedMoreParams {
                        server_host: server_host.to_string(),
                        nick: nick.to_string(),
                        command: "USER".to_string(),
                    }
                ]
            );

            return Some(map);
        },
        Some(user) => user
    };

    // TODO validate on mode?

    let realname = match realname {
        None => {
            let mut map = HashMap::new();
            map.insert(
                conn_context.connection_id,
                vec![
                    Reply::ErrNeedMoreParams {
                        server_host: server_host.to_string(),
                        nick: nick.to_string(),
                        command: "USER".to_string(),
                    }
                ]
            );

            return Some(map);
        },
        Some(realname) => realname.trim_start_matches(':')
    };

    conn_context.user = Some(user.to_string());
    // TODO add mode?
    conn_context.real_name = Some(realname.to_string());

    None
}
