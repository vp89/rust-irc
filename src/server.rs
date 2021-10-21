use chrono::Utc;
use std::{cell::RefCell, collections::{HashMap, VecDeque}, sync::{Arc, RwLock, mpsc::{self, Receiver, Sender}}};
use uuid::Uuid;

use crate::{ConnectionContext, ServerContext, channels::{FakeChannelReceiver, ReceiverWrapper}, message_parsing::{ClientToServerCommand, ClientToServerMessage}, replies::Reply};

use crate::result::Result;

pub fn run_server(
    server_context: &ServerContext,
    receiver_channel: &dyn ReceiverWrapper<ClientToServerMessage>,
    connections: Arc<RwLock<HashMap<Uuid, ConnectionContext>>>,
) -> Result<()> {
    let server_host = server_context.server_host.clone();
    let empty_str = &String::from("");
    let mut srv_channels: HashMap<String, ChannelContext> = HashMap::new();

    loop {
        let received = receiver_channel
            .receive()?;

        let conn_read = match connections.read() {
            Ok(c) => c,
            Err(e) => {
                println!("Error acquiring read lock on shared connections map, must skip handling this message {:?}", e);
                continue;
            }
        };

        let conn_context = match conn_read.get(&received.connection_uuid) {
            Some(c) => c,
            None => {
                println!(
                    "Connection context not found for UUID {}",
                    &received.connection_uuid
                );
                continue;
            }
        };

        let ctx_client = conn_context.client.as_ref().unwrap_or(empty_str);
        let ctx_nick = conn_context.nick.as_ref().unwrap_or(empty_str);

        let ctx_sender = match conn_context.client_sender_channel.lock() {
            Ok(c) => c.clone(),
            Err(e) => {
                println!("Error when trying to lock on client sender channel, therefore must skip handling this message {:?}", e);
                continue;
            }
        };

        match &received.command {
            // These require modifying the connection context
            ClientToServerCommand::Nick { .. } => {
                // drop the read lock before taking a write-lock, this needs to be done for any command
                // that needs write access to the connection context
                drop(conn_read);

                let mut conn_write = match connections.write() {
                    Ok(c) => c,
                    Err(e) => {
                        println!("RwLock on connections map is poisoned {:?}", e);
                        continue;
                    }
                };

                let conn_context = match conn_write.get_mut(&received.connection_uuid) {
                    Some(ctx) => ctx,
                    None => {
                        println!("Received a command for an unexpected connection UUID, client worker should be properly initialized before reading from the TcpStream");
                        continue;
                    }
                };

                let ctx_version = &server_context.version;
                let ctx_created_at = &server_context.start_time;
                let ctx_sender = match conn_context.client_sender_channel.lock() {
                    Ok(c) => c.clone(),
                    Err(e) => {
                        println!("Error when trying to lock on client sender channel, therefore must skip handling this message {:?}", e);
                        continue;
                    }
                };

                if let ClientToServerCommand::Nick { nick } = &received.command {
                    conn_context.nick = Some(nick.to_string());
                    conn_context.client = Some(format!("{}!~{}@localhost", nick, nick));

                    send_replies(
                        &ctx_sender,
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
                    )
                }
            }
            ClientToServerCommand::Join { channels } => {
                let now = Utc::now();

                for channel in channels {
                    match srv_channels.get_mut(channel) {
                        Some(c) => c.members.push(received.connection_uuid),
                        None => {
                            srv_channels.insert(
                                channel.clone(),
                                // TODO this probably won't be right eventually
                                // if there needs to be persisted channel ownership?
                                ChannelContext {
                                    members: vec![received.connection_uuid],
                                },
                            );
                        }
                    }

                    send_replies(
                        &ctx_sender,
                        vec![
                            Reply::Join {
                                client: ctx_client.clone(),
                                channel: channel.clone(),
                            },
                            // TODO have Nick available here
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
                            Reply::Nam {
                                server_host: server_host.clone(),
                                channel: channel.clone(),
                                nick: ctx_nick.clone(),
                            },
                        ],
                    );
                }
            }
            ClientToServerCommand::Mode { channel } => {
                let now = Utc::now();

                send_replies(
                    &ctx_sender,
                    vec![
                        Reply::Mode {
                            server_host: server_host.clone(),
                            channel: channel.clone(),
                            mode_string: "+tn".to_string(),
                        },
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
            ClientToServerCommand::Who { channel } => send_replies(
                &ctx_sender,
                vec![
                    Reply::Who {
                        server_host: server_host.clone(),
                        channel: channel.clone(),
                        nick: ctx_nick.clone(),
                        other_nick: "~vince".to_string(),
                        client: "localhost".to_string(),
                    },
                    Reply::EndOfWho {
                        server_host: server_host.clone(),
                        nick: ctx_nick.clone(),
                        channel: channel.clone(),
                    },
                ],
            ),
            ClientToServerCommand::PrivMsg { channel, message } => {
                let sender_member = match conn_read.get(&received.connection_uuid) {
                    Some(conn) => conn,
                    None => {
                        println!(
                            "Unable to find sender member {} in connections map",
                            &received.connection_uuid
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

                println!(
                    "Found {} channel members connected to {}",
                    &channel_ctx.members.len(),
                    channel
                );
                for member in &channel_ctx.members {
                    let connected_member = match conn_read.get(member) {
                        Some(conn) => conn,
                        None => {
                            println!("Unable to find member {} in connections map", member);
                            continue;
                        }
                    };

                    println!("FOUND {}", &connected_member.nick.as_ref().unwrap());

                    let lock = &connected_member.client_sender_channel.lock();
                    let sender = match lock {
                        Ok(s) => s,
                        Err(e) => {
                            println!(
                                "Unable to lock sender channel for member {} {:?}",
                                member, e
                            );
                            continue;
                        }
                    };

                    send_replies(
                        sender,
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

struct ChannelContext {
    members: Vec<Uuid>,
}

// TODO fill this out with test logic but this skeleton should work for mocking
#[test]
pub fn server_test_skeleton() {
    let messages = VecDeque::new();
    let receiver = FakeChannelReceiver {
        faked_messages: RefCell::new(Box::new(messages))
    };
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let context = ServerContext {
        start_time: Utc::now(),
        server_host: "localhost".to_string(),
        version: "0.0.1".to_string(),
        ping_frequency: std::time::Duration::from_secs(60),
    };

    run_server(&context, &receiver, connections);
}