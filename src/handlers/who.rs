use crate::{replies::Reply, util, ChannelContext, ConnectionContext};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::RwLockReadGuard,
};

use uuid::Uuid;

pub fn handle_who(
    mask: &Option<String>,
    server_host: &str,
    nick: &str,
    srv_channels: &HashMap<String, ChannelContext>,
    conn_read: &RwLockReadGuard<HashMap<Uuid, ConnectionContext>>,
) -> Vec<Reply> {
    /*
    The <mask> passed to WHO is matched against users' host, server, real
    name and nickname if the channel <mask> cannot be found.
    */
    let mask = match mask {
        Some(m) => m,
        None => {
            return vec![Reply::ErrNeedMoreParams {
                server_host: server_host.to_string(),
                nick: nick.to_string(),
                command: "WHO".to_string(),
            }]
        }
    };

    let mut is_mask_channel = false;

    // if there is a mask, first check that it matches a channel
    // The <mask> passed to WHO is matched against users' host, server, real
    // name and nickname if the channel <mask> cannot be found.

    let chan_ctx = srv_channels.get(mask);

    let mut members = vec![];

    let users = match chan_ctx {
        Some(c) => {
            is_mask_channel = true;
            &c.members
        }
        None => {
            let empty_str = "".to_string();
            let empty_ip = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 1234));
            for (k, v) in conn_read.iter() {
                let hostmask = format!(
                    "{}!{}@{}",
                    v.nick.as_ref().unwrap_or(&empty_str).clone(),
                    v.user.as_ref().unwrap_or(&empty_str).clone(),
                    v.client_host.as_ref().unwrap_or(&empty_ip).clone() // TODO
                );

                if util::match_mask(&hostmask, mask) {
                    members.push(*k);
                }
            }

            &members
        }
    };

    let mut replies = vec![];

    for user in users {
        let other_user = match conn_read.get(user) {
            Some(c) => c,
            None => {
                println!("Connection context not found for matched WHO user {}", user);
                continue;
            }
        };

        let empty_str = "".to_string();

        // TODO
        let empty_ip = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 1234));

        // TODOTODOTODO
        replies.push(Reply::Who {
            server_host: server_host.to_string(),
            nick: nick.to_string(),
            channel: match is_mask_channel {
                true => mask.clone(),
                false => "*".to_string(),
            },
            other_user: other_user.user.as_ref().unwrap_or(&empty_str).clone(),
            other_host: other_user
                .client_host
                .as_ref()
                .unwrap_or(&empty_ip)
                .to_string(),
            other_server: server_host.to_string(), // multi-server not supported
            other_nick: other_user.nick.as_ref().unwrap_or(&empty_str).clone(),
            other_realname: other_user.real_name.as_ref().unwrap_or(&empty_str).clone(),
        })
    }

    replies.push(Reply::EndOfWho {
        server_host: server_host.to_string(),
        nick: nick.to_string(),
        mask: mask.to_string(),
    });

    replies
}