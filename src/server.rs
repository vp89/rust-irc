use chrono::Utc;
use std::{cell::RefCell, collections::{HashMap, VecDeque}, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, sync::{Arc, Mutex, RwLock, mpsc::{self, Sender}}};
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
            // TODO add mask
            /*
                352    RPL_WHOREPLY
                        "<channel> <user> <host> <server> <nick>
                        ( "H" / "G" > ["*"] [ ( "@" / "+" ) ]
                        :<hopcount> <real name>"

                315    RPL_ENDOFWHO
                        "<name> :End of WHO list"

                    - The RPL_WHOREPLY and RPL_ENDOFWHO pair are used
                    to answer a WHO message.  The RPL_WHOREPLY is only
                    sent if there is an appropriate match to the WHO
                    query.  If there is a list of parameters supplied
                    with a WHO message, a RPL_ENDOFWHO MUST be sent
                    after processing each list item with <name> being
                    the item.
           */
            ClientToServerCommand::Who { mask } => {
                /*
                The <mask> passed to WHO is matched against users' host, server, real
                name and nickname if the channel <mask> cannot be found.
                */

                match mask {
                    Some(m) => {
                        let raw_mask = &m.value;

                        // if there is a mask, first check that it matches a channel
                        // The <mask> passed to WHO is matched against users' host, server, real
                        // name and nickname if the channel <mask> cannot be found.

                        let chan_ctx = srv_channels.get(raw_mask);

                        let empty_members = vec![];

                        let users = match chan_ctx {
                            Some(c) => &c.members,
                            None => {
                                 // todo return all conns that match the mask
                                 &empty_members
                            }
                        };

                        println!("!! {} users found for WHO", users.len());

                        let mut replies = vec![];

                        for user in users {
                            let other_user = match conn_read.get(&user) {
                                Some(c) => c,
                                None => {
                                    println!(
                                        "Connection context not found for matched WHO user {}",
                                        user
                                    );
                                    continue;
                                }
                            };

                            let empty_str = "".to_string();
                            println!("WHO {} {}", other_user.uuid, other_user.nick.as_ref().unwrap_or(&empty_str));

                            // TODO
                            let empty_ip = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 1234));

                            // TODOTODOTODO
                            replies.push(Reply::Who {
                                server_host: server_host.clone(),
                                nick: ctx_nick.clone(),
                                mask: raw_mask.clone(),
                                other_user: other_user.user.as_ref().unwrap_or(&empty_str).clone(),
                                other_host: other_user.client_host.as_ref().unwrap_or(&empty_ip).to_string(),
                                other_server: server_host.clone(), // multi-server not supported
                                other_nick: other_user.nick.as_ref().unwrap_or(&empty_str).clone(),
                                other_realname: other_user.real_name.as_ref().unwrap_or(&empty_str).clone()
                            })
                        }

                        replies.push(Reply::EndOfWho {
                            server_host: server_host.clone(),
                            nick: ctx_nick.clone(),
                            mask: raw_mask.clone(),
                        });

                        send_replies(&ctx_sender, replies);
                    },
                    None => {
                        // TODO send error reply
                        /*
                        send_replies(

                        )
                        */
                    }
                }
            }
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

#[test]
pub fn server_nickcommandsent_replystormissent() {
    // Arrange
    let mut connections = HashMap::new();

    let (sender, test_receiver) = mpsc::channel();
    let connection_uuid = Uuid::new_v4();
    connections.insert(connection_uuid, ConnectionContext {
        uuid: connection_uuid,
        client_sender_channel: Mutex::new(sender),
        client: None,
        nick: None,
        user: None,
        real_name: None,
        client_host: None,
    });

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
        receive_count: RefCell::new(0)
    };

    let context = ServerContext {
        start_time: Utc::now(),
        server_host: "localhost".to_string(),
        version: "0.0.1".to_string(),
        ping_frequency: std::time::Duration::from_secs(60),
    };

    // Act
    let assert_connections = connections.clone();
    let result = run_server(&context, &receiver, connections);

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
}
