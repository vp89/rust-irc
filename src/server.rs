use chrono::Utc;
use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{
    channels::ReceiverWrapper,
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

        if let ClientToServerCommand::Nick {
            nick,
            sender,
            client_ip,
        } = &received.command
        {
            let ctx_version = &server_context.version;
            let ctx_created_at = &server_context.start_time;
            let ctx = ConnectionContext {
                connection_id: received.connection_id,
                client_sender_channel: sender.clone(),
                nick: Some(nick.to_string()),
                client: Some(format!("{}!~{}@localhost", nick, nick)),
                user: None,
                real_name: None,
                client_host: *client_ip,
            };
            connections.insert(received.connection_id, ctx);

            send_replies(
                sender,
                vec![
                    Reply::Welcome {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                    },
                    Reply::YourHost {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        version: ctx_version.clone(),
                    },
                    Reply::Created {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        created_at: *ctx_created_at,
                    },
                    Reply::MyInfo {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        version: ctx_version.clone(),
                        user_modes: "r".to_string(),
                        channel_modes: "i".to_string(),
                    },
                    Reply::Support {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        channel_len: 32,
                    },
                    Reply::LuserClient {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        visible_users: 100,
                        invisible_users: 20,
                        servers: 1,
                    },
                    Reply::LuserOp {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        operators: 1337,
                    },
                    Reply::LuserUnknown {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        unknown: 7,
                    },
                    Reply::LuserChannels {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        channels: 9999,
                    },
                    Reply::LuserMe {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        clients: 900,
                        servers: 1,
                    },
                    Reply::LocalUsers {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        current: 845,
                        max: 1000,
                    },
                    Reply::GlobalUsers {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        current: 9832,
                        max: 23455,
                    },
                    Reply::StatsDLine {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        connections: 9998,
                        clients: 9000,
                        received: 99999,
                    },
                    Reply::MotdStart {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                    },
                    // TODO proper configurable MOTD
                    Reply::Motd {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                        line: "Foobar".to_string(),
                    },
                    Reply::EndOfMotd {
                        server_host: server_host.clone(),
                        nick: nick.clone(),
                    },
                ],
            );
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
            // These require modifying the connection context
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

                    send_replies(&conn_context.client_sender_channel, replies);

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
                            &other_user.client_sender_channel,
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
                    &conn_context.client_sender_channel,
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
                &conn_context.client_sender_channel,
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
                        &connected_member.client_sender_channel,
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
    use super::*;
    use crate::channels::FakeChannelReceiver;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::sync::{
        mpsc::{self},
        Mutex,
    };

    /*
    #[test]
    pub fn server_nickcommandsent_replystormissent() {
        // Arrange
        let mut connections = HashMap::new();

        let (sender, test_receiver) = mpsc::channel();
        let connection_uuid = Uuid::new_v4();
        connections.insert(
            connection_uuid,
            ConnectionContext {
                uuid: connection_uuid,
                client_sender_channel: sender,
                client: None,
                nick: None,
                user: None,
                real_name: None,
                client_host: None,
            },
        );

        let connections = Arc::new(RwLock::new(connections));

        let mut messages = VecDeque::new();
        messages.push_front(ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Nick {
                nick: "JOE".to_string(),
            },
            connection_uuid,
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

        // TODO fix this test after refactor
        /*
        // Act
        let assert_connections = connections.clone();
        let result = run_server(&context, &receiver);

        // Assert
        if let Ok(()) = result {
            assert!(false)
        }
        assert_eq!(2, receiver.receive_count.take());
        // try_iter is required because the sender channel is kept alive due
        // to cloning the Arc to the connections map
        // try_iter will yield whatever is in the receiver even
        // though the sender hasn't hung up whereas iter would block
        // because the sender hasn't hung up
        assert_eq!(16, test_receiver.try_iter().count());

        let dict = assert_connections.read().unwrap();
        let conn_ctx = dict.get(&connection_uuid).unwrap();
        assert_eq!(&Some("JOE".to_string()), &conn_ctx.nick);
        */
    }
    */
}
