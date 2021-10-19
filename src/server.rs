use chrono::Utc;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, RwLock,
    },
};
use uuid::Uuid;

use crate::{
    message_parsing::{ClientToServerCommand, ClientToServerMessage},
    replies::Reply,
    ConnectionContext, ServerContext,
};

use crate::result::Result;
use crate::error::Error::ClientToServerChannelFailedToReceive;

pub fn run_server(
    context: &ServerContext,
    receiver_channel: Receiver<ClientToServerMessage>,
    connections: Arc<RwLock<HashMap<Uuid, ConnectionContext>>>,
) -> Result<()> {
    let host = context.host.clone();
    let empty_str = &String::from("");
    let mut srv_channels: HashMap<String, ChannelContext> = HashMap::new();

    loop {
        let received = receiver_channel.recv().map_err(ClientToServerChannelFailedToReceive)?;

        match &received.command {
            // These require modifying the connection context
            ClientToServerCommand::Nick { .. } => {
                let mut conn_write = match connections.write() {
                    Ok(c) => c,
                    Err(e) => {
                        println!("RwLock on connections map is poisoned {:?}", e);
                        continue;
                    }
                };

                let writable_ctx = match conn_write.get_mut(&received.connection_uuid) {
                    Some(ctx) => ctx,
                    None => {
                        println!("Received a command for an unexpected connection UUID, client worker should be properly initialized before reading from the TcpStream");
                        continue;
                    }
                };

                let ctx_version = &context.version;
                let ctx_created_at = &context.start_time;
                let ctx_sender = writable_ctx.client_sender_channel.lock().unwrap().clone();

                if let ClientToServerCommand::Nick { nick } = &received.command {
                    writable_ctx.nick = Some(nick.to_string());
                    writable_ctx.client = Some(format!("{}!~{}@localhost", nick, nick));

                    send_replies(
                        &ctx_sender,
                        vec![
                            Reply::Welcome {
                                host: host.clone(),
                                nick: nick.clone(),
                            },
                            Reply::YourHost {
                                host: host.clone(),
                                nick: nick.clone(),
                                version: ctx_version.clone(),
                            },
                            Reply::Created {
                                host: host.clone(),
                                nick: nick.clone(),
                                created_at: *ctx_created_at,
                            },
                            Reply::MyInfo {
                                host: host.clone(),
                                nick: nick.clone(),
                                version: ctx_version.clone(),
                                user_modes: "r".to_string(),
                                channel_modes: "i".to_string(),
                            },
                            Reply::Support {
                                host: host.clone(),
                                nick: nick.clone(),
                                channel_len: 32,
                            },
                            Reply::LuserClient {
                                host: host.clone(),
                                nick: nick.clone(),
                                visible_users: 100,
                                invisible_users: 20,
                                servers: 1,
                            },
                            Reply::LuserOp {
                                host: host.clone(),
                                nick: nick.clone(),
                                operators: 1337,
                            },
                            Reply::LuserUnknown {
                                host: host.clone(),
                                nick: nick.clone(),
                                unknown: 7,
                            },
                            Reply::LuserChannels {
                                host: host.clone(),
                                nick: nick.clone(),
                                channels: 9999,
                            },
                            Reply::LuserMe {
                                host: host.clone(),
                                nick: nick.clone(),
                                clients: 900,
                                servers: 1,
                            },
                            Reply::LocalUsers {
                                host: host.clone(),
                                nick: nick.clone(),
                                current: 845,
                                max: 1000,
                            },
                            Reply::GlobalUsers {
                                host: host.clone(),
                                nick: nick.clone(),
                                current: 9832,
                                max: 23455,
                            },
                            Reply::StatsDLine {
                                host: host.clone(),
                                nick: nick.clone(),
                                connections: 9998,
                                clients: 9000,
                                received: 99999,
                            },
                            Reply::MotdStart {
                                host: host.clone(),
                                nick: nick.clone(),
                            },
                            // TODO proper configurable MOTD
                            Reply::Motd {
                                host: host.clone(),
                                nick: nick.clone(),
                                line: "Foobar".to_string(),
                            },
                            Reply::EndOfMotd {
                                host: host.clone(),
                                nick: nick.clone(),
                            },
                        ],
                    )
                }
            }
            // Everything else is read-only
            _ => {
                let conn_read = connections.read().unwrap();
                let conn_ctx = conn_read.get(&received.connection_uuid).unwrap();
                let ctx_client = conn_ctx.client.as_ref().unwrap_or(empty_str);
                let ctx_nick = conn_ctx.nick.as_ref().unwrap_or(empty_str);
                let ctx_sender = conn_ctx.client_sender_channel.lock().unwrap().clone();

                match &received.command {
                    ClientToServerCommand::Join { channels } => {
                        let now = Utc::now();

                        for channel in channels {
                            if !srv_channels.contains_key(channel) {
                                srv_channels.insert(
                                    channel.clone(),
                                    // TODO this probably won't be right eventually
                                    // if there needs to be persisted channel ownership?
                                    ChannelContext {
                                        members: vec![received.connection_uuid],
                                    },
                                );
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
                                        host: host.clone(),
                                        nick: ctx_nick.clone(),
                                        channel: channel.clone(),
                                        topic: "foobar topic".to_string(),
                                    },
                                    Reply::TopicWhoTime {
                                        host: host.clone(),
                                        channel: channel.clone(),
                                        nick: ctx_nick.clone(),
                                        set_at: now,
                                    },
                                    Reply::Nam {
                                        host: host.clone(),
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
                                    host: host.clone(),
                                    channel: channel.clone(),
                                    mode_string: "+tn".to_string(),
                                },
                                Reply::ChannelModeIs {
                                    host: host.clone(),
                                    nick: ctx_nick.clone(),
                                    channel: channel.clone(),
                                    mode_string: "+mtn1".to_string(),
                                    mode_arguments: "100".to_string(),
                                },
                                Reply::CreationTime {
                                    host: host.clone(),
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
                                host: host.clone(),
                                channel: channel.clone(),
                                nick: ctx_nick.clone(),
                                other_nick: "~vince".to_string(),
                                client: "localhost".to_string(),
                            },
                            Reply::EndOfWho {
                                host: host.clone(),
                                nick: ctx_nick.clone(),
                                channel: channel.clone(),
                            },
                        ],
                    ),
                    ClientToServerCommand::PrivMsg { channel, message } => {
                        let channel_ctx = match srv_channels.get(channel) {
                            Some(c) => c,
                            None => {
                                println!(
                                    "Unable to send message to channel {}, not found",
                                    channel
                                );
                                continue;
                            }
                        };

                        for member in &channel_ctx.members {
                            let connected_member = match conn_read.get(member) {
                                Some(conn) => conn,
                                None => {
                                    println!("Unable to find member {} in connections map", member);
                                    continue;
                                }
                            };
                            send_replies(
                                // TODO get rid of unwrap
                                &connected_member
                                    .client_sender_channel
                                    .lock()
                                    .unwrap()
                                    .clone(),
                                vec![Reply::PrivMsg {
                                    client: host.clone(),
                                    channel: channel.clone(),
                                    message: message.clone(),
                                }],
                            );
                        }
                    }
                    // these won't make it here
                    ClientToServerCommand::Nick { .. } => {}
                    ClientToServerCommand::Unhandled { .. } => {}
                    ClientToServerCommand::Ping { .. } => {}
                    ClientToServerCommand::Pong {} => {}
                    ClientToServerCommand::Quit {} => {}
                };
            }
        }
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
