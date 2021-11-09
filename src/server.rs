use chrono::Utc;
use std::{collections::HashMap, sync::mpsc::Sender};
use uuid::Uuid;

use crate::{ChannelContext, ConnectionContext, ServerContext, channels::ReceiverWrapper, handlers::{join::handle_join, mode::handle_mode, nick::handle_nick, privmsg::handle_privmsg}, message_parsing::{ClientToServerCommand, ClientToServerMessage}, replies::Reply};

use crate::handlers::who::*;
use crate::result::Result;

pub fn run_server(
    server_context: &ServerContext,
    receiver_channel: &dyn ReceiverWrapper<ClientToServerMessage>,
) -> Result<()> {
    let mut connections = HashMap::new();
    let server_host = server_context.server_host.clone();
    let empty_str = &String::from("");
    let mut channels: HashMap<String, ChannelContext> = HashMap::new();

    loop {
        let received = receiver_channel.receive()?;

        if let ClientToServerCommand::Connected { sender, client_ip } = &received.command {
            let ctx = ConnectionContext {
                connection_id: received.connection_id,
                client_sender_channel: sender.clone(),
                nick: None,
                client: None,
                user: None,
                real_name: None,
                client_host: *client_ip,
            };
            connections.insert(received.connection_id, ctx);
            continue;
        }

        if let ClientToServerCommand::Nick { nick, .. } = &received.command {
            let mut conn_context = match connections.get_mut(&received.connection_id) {
                Some(c) => c,
                None => {
                    continue;
                }
            };

            let replies = handle_nick(
                &server_host,
                nick,
                &server_context.version,
                &server_context.start_time,
                &mut conn_context,
            );

            send_replies(replies, &connections);
            continue;
        }

        if let ClientToServerCommand::User {
            user,
            mode: _,
            realname,
        } = &received.command
        {
            let mut conn_context = match connections.get_mut(&received.connection_id) {
                Some(c) => c,
                None => {
                    continue;
                }
            };
            conn_context.user = Some(user.to_string());
            conn_context.real_name = Some(realname.trim_start_matches(':').to_string());
            continue;
        }

        let conn_context = match connections.get(&received.connection_id) {
            Some(c) => c,
            None => {
                continue;
            }
        };

        let ctx_client = conn_context.client.as_ref().unwrap_or(empty_str);
        let ctx_nick = conn_context.nick.as_ref().unwrap_or(empty_str);

        match &received.command {
            // These require modifying the connection context so we run them above to make below code easier
            ClientToServerCommand::Connected { .. } => {}
            ClientToServerCommand::User { .. } => {}
            ClientToServerCommand::Nick { .. } => {}
            ClientToServerCommand::Join { channels_to_join } => {
                let replies = handle_join(&server_host, ctx_nick, ctx_client, &conn_context, &mut channels, &connections, channels_to_join);
                send_replies(replies, &connections);
            }
            ClientToServerCommand::Mode { channel } => {
                let replies = handle_mode(&server_host, ctx_nick, &channel, &conn_context);
                send_replies(replies, &connections);
            }
            ClientToServerCommand::Who { mask, .. } => {
                let replies = handle_who(mask, &server_host, ctx_nick, &channels, &connections, &conn_context);
                send_replies(replies, &connections);
            }
            ClientToServerCommand::PrivMsg { channel, message } => {
                let replies = handle_privmsg(&server_host, ctx_nick, ctx_client, channel, message, &conn_context, &channels, &connections);
                send_replies(replies, &connections);
            }
            // these won't make it here
            ClientToServerCommand::Unhandled { .. } => {}
            ClientToServerCommand::Ping { .. } => {}
            ClientToServerCommand::Pong {} => {}
            ClientToServerCommand::Quit {} => {}
        };
    }
}

fn send_replies(replies_per_user: HashMap<Uuid, Vec<Reply>>, connections: &HashMap<Uuid, ConnectionContext>) {
    for (connection_id, replies) in replies_per_user {
        let sender = match connections.get(&connection_id) {
            Some(ctx) => &ctx.client_sender_channel.0,
            None => {
                // TODO
                continue;
            }
        };

        for reply in replies {
            if let Err(e) = sender.send(reply) {
                println!("Error sending replies {:?}", e);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::channels::FakeChannelReceiver;
    use crate::message_parsing::ReplySender;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use std::sync::mpsc::{self};

    #[test]
    pub fn server_nickcommandsent_replystormissent() {
        // Arrange
        let (sender, test_receiver) = mpsc::channel();
        let connection_id = Uuid::new_v4();

        let mut messages = VecDeque::new();
        messages.push_back(ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Connected {
                sender: ReplySender(sender),
                client_ip: Some(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    1234,
                ))),
            },
            connection_id,
        });

        messages.push_back(ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Nick {
                nick: Some("JOE".to_string()),
            },
            connection_id,
        });

        let receiver = FakeChannelReceiver {
            faked_messages: RefCell::new(Box::new(messages)),
            receive_count: RefCell::new(0),
        };

        let context = ServerContext {
            start_time: Utc::now(),
            server_host: "localhost".to_string(),
            version: "0.0.1".to_string(),
            ping_frequency: std::time::Duration::from_secs(60),
        };

        // Act
        let result = run_server(&context, &receiver);

        // Assert
        if let Ok(()) = result {
            assert!(false)
        }
        assert_eq!(3, receiver.receive_count.take());
        // try_iter is required because the sender channel is kept alive due
        // to cloning the Arc to the connections map
        // try_iter will yield whatever is in the receiver even
        // though the sender hasn't hung up whereas iter would block
        // because the sender hasn't hung up
        assert_eq!(16, test_receiver.try_iter().count());
    }
}
