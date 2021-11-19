use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    channels::ReceiverWrapper,
    handlers::{
        join::handle_join, mode::handle_mode, nick::handle_nick, privmsg::handle_privmsg,
        quit::handle_quit,
    },
    message_parsing::{ClientToServerCommand, ClientToServerMessage},
    replies::Reply,
    ChannelContext, ConnectionContext, ServerContext,
};

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

        // This should always be the first command received for any given connection
        // and thus the only one where there is no connection context available.
        // We can handle it here instead of in the match below so that the rest of the
        // commands can just deal with a ConnectionContext instead of an Option<ConnectionContext>
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

        let conn_context = match connections.get(&received.connection_id) {
            Some(c) => c,
            None => {
                continue;
            }
        };

        let ctx_client = conn_context.client.as_ref().unwrap_or(empty_str);
        let ctx_nick = conn_context.nick.as_ref().unwrap_or(empty_str);

        let replies = match &received.command {
            ClientToServerCommand::Disconnected => {
                match connections.remove(&received.connection_id) {
                    Some(_) => {
                        println!(
                            "REMOVED CONNECTION {}. {} users now connected",
                            &received.connection_id,
                            connections.len()
                        );
                    }
                    None => {
                        println!(
                            "UNABLE TO REMOVE CONNECTION SHUTDOWN {}",
                            &received.connection_id
                        );
                    }
                }
                None
            }
            ClientToServerCommand::User {
                user,
                mode: _,
                realname,
            } => {
                let mut conn_context = match connections.get_mut(&received.connection_id) {
                    Some(c) => c,
                    None => {
                        continue;
                    }
                };
                conn_context.user = Some(user.to_string());
                conn_context.real_name = Some(realname.trim_start_matches(':').to_string());
                None
            }
            ClientToServerCommand::Nick { nick, .. } => {
                let mut conn_context = match connections.get_mut(&received.connection_id) {
                    Some(c) => c,
                    None => {
                        continue;
                    }
                };

                handle_nick(
                    &server_host,
                    nick,
                    &server_context.version,
                    &server_context.start_time,
                    &mut conn_context,
                )
            }
            ClientToServerCommand::Join { channels_to_join } => handle_join(
                &server_host,
                ctx_nick,
                ctx_client,
                conn_context,
                &mut channels,
                &connections,
                channels_to_join,
            ),
            ClientToServerCommand::Mode { channel } => {
                handle_mode(&server_host, ctx_nick, channel, conn_context)
            }
            ClientToServerCommand::Who { mask, .. } => handle_who(
                mask,
                &server_host,
                ctx_nick,
                &channels,
                &connections,
                conn_context,
            ),
            ClientToServerCommand::PrivMsg { channel, message } => handle_privmsg(
                ctx_nick,
                channel,
                message,
                conn_context,
                &channels,
                &connections,
            ),
            ClientToServerCommand::Quit { message } => {
                handle_quit(message, &mut channels, &connections, received.connection_id)
            }
            ClientToServerCommand::Connected { .. } => None,
            ClientToServerCommand::Unhandled { .. } => None,
            ClientToServerCommand::Ping { .. } => None,
            ClientToServerCommand::Pong {} => None,
        };

        if let Some(replies) = replies {
            send_replies(replies, &connections)
        }
    }
}

fn send_replies(
    replies_per_user: HashMap<Uuid, Vec<Reply>>,
    connections: &HashMap<Uuid, ConnectionContext>,
) {
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
    use chrono::Utc;
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
