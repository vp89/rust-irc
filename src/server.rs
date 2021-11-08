use chrono::Utc;
use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{
    channels::ReceiverWrapper,
    handlers::nick::handle_nick,
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
    let mut srv_channels: HashMap<String, ChannelContext> = HashMap::new();

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

            send_replies(&conn_context.client_sender_channel.0, replies);

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
            ClientToServerCommand::Join { channels } => {
                let now = Utc::now();

                for channel in channels {
                    match srv_channels.get_mut(channel) {
                        Some(c) => c.members.push(received.connection_id),
                        None => {
                            srv_channels.insert(
                                channel.clone(),
                                // TODO this probably won't be right eventually
                                // if there needs to be persisted channel ownership?
                                ChannelContext {
                                    members: vec![received.connection_id],
                                },
                            );
                        }
                    }

                    let mut replies = vec![
                        Reply::Join {
                            client: ctx_client.clone(),
                            channel: channel.clone(),
                        },
                        // TODO persist the channel metadata
                        Reply::Topic {
                            server_host: server_host.clone(),
                            nick: ctx_nick.clone(),
                            channel: channel.clone(),
                            topic: "foobar topic".to_string(),
                        },
                        Reply::TopicWhoTime {
                            server_host: server_host.clone(),
                            channel: channel.clone(),
                            nick: ctx_nick.clone(),
                            set_at: now,
                        },
                    ];

                    let chan_ctx = match srv_channels.get(channel) {
                        Some(c) => c,
                        None => {
                            println!(
                                "Unable for {} to join topic {} as user list not found for it",
                                ctx_nick, channel
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
                        server_host: server_host.clone(),
                        nick: ctx_nick.clone(),
                        channel: channel.clone(),
                        channel_users,
                    });

                    replies.push(Reply::EndOfNames {
                        server_host: server_host.clone(),
                        nick: ctx_nick.clone(),
                        channel: channel.clone(),
                    });

                    send_replies(&conn_context.client_sender_channel.0, replies);

                    for member in &chan_ctx.members {
                        if member == &received.connection_id {
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

                        send_replies(
                            &other_user.client_sender_channel.0,
                            vec![Reply::Join {
                                client: ctx_client.clone(),
                                channel: channel.clone(),
                            }],
                        );
                    }
                }
            }
            ClientToServerCommand::Mode { channel } => {
                let now = Utc::now();

                send_replies(
                    &conn_context.client_sender_channel.0,
                    vec![
                        Reply::ChannelModeIs {
                            server_host: server_host.clone(),
                            nick: ctx_nick.clone(),
                            channel: channel.clone(),
                            mode_string: "+mtn1".to_string(),
                            mode_arguments: "100".to_string(),
                        },
                        Reply::CreationTime {
                            server_host: server_host.clone(),
                            nick: ctx_nick.clone(),
                            channel: channel.clone(),
                            created_at: now,
                        },
                    ],
                )
            }
            ClientToServerCommand::Who { mask, .. } => send_replies(
                &conn_context.client_sender_channel.0,
                handle_who(mask, &server_host, ctx_nick, &srv_channels, &connections),
            ),
            ClientToServerCommand::PrivMsg { channel, message } => {
                let sender_member = match connections.get(&received.connection_id) {
                    Some(conn) => conn,
                    None => {
                        println!(
                            "Unable to find sender member {} in connections map",
                            &received.connection_id
                        );
                        continue;
                    }
                };

                let channel_ctx = match srv_channels.get(channel) {
                    Some(c) => c,
                    None => {
                        println!("Unable to send message to channel {}, not found", channel);
                        continue;
                    }
                };

                for member in &channel_ctx.members {
                    if member == &received.connection_id {
                        continue;
                    }

                    let connected_member = match connections.get(member) {
                        Some(conn) => conn,
                        None => {
                            println!("Unable to find member {} in connections map", member);
                            continue;
                        }
                    };

                    send_replies(
                        &connected_member.client_sender_channel.0,
                        vec![Reply::PrivMsg {
                            nick: sender_member.nick.clone(),
                            user: sender_member.user.clone(),
                            client_host: sender_member.client_host,
                            channel: channel.clone(),
                            message: message.clone(),
                        }],
                    );
                }
            }
            // these won't make it here
            ClientToServerCommand::Unhandled { .. } => {}
            ClientToServerCommand::Ping { .. } => {}
            ClientToServerCommand::Pong {} => {}
            ClientToServerCommand::Quit {} => {}
        };
    }
}

fn send_replies(sender: &Sender<Reply>, replies: Vec<Reply>) {
    for reply in replies {
        if let Err(e) = sender.send(reply) {
            println!("Error sending replies {:?}", e);
            return;
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
