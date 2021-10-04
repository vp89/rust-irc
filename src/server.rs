use std::{collections::HashMap, sync::{Arc, Mutex, RwLock, mpsc::{Receiver}}};
use chrono::Utc;
use uuid::Uuid;

use crate::{ConnectionContext, ServerContext, message_parsing::{ClientToServerCommand, ClientToServerMessage}, replies::Reply};

pub fn run_server(
    context: &ServerContext,
    receiver_channel: Receiver<ClientToServerMessage>,
    connections: Arc<RwLock<HashMap<Uuid, ConnectionContext>>>) {
    
    let host = context.host.clone();
    // TODO pluck these out of the connection context
    let mut connection_nick = "".to_string();
    let mut connection_client = "".to_string();

    loop {
        for received in receiver_channel.recv() {
            // TODO is this right?? This seems so janky
            let sender = connections
                .read()
                .unwrap()
                .get(&received.connection_uuid)
                .unwrap()
                .client_sender_channel
                .lock()
                .unwrap()
                .clone();

            let version = &context.version;
            let created_at = &context.start_time;

            match &received.command {
                ClientToServerCommand::Join { channels} => {
                    let now = Utc::now();

                    for channel in channels {
                        sender.send(Reply::Join { client: connection_client.clone(), channel: channel.clone() });
                        // TODO have Nick available here
                        // TODO persist the channel metadata
                        sender.send(Reply::Topic { host: host.clone(), nick: connection_nick.clone(), channel: channel.clone(), topic: "foobar topic".to_string() });
                        sender.send(Reply::TopicWhoTime { host: host.clone(), channel: channel.clone(), nick: connection_nick.clone(), set_at: now.clone() });
                        sender.send(Reply::NamReply { host: host.clone(), channel: channel.clone(), nick: connection_nick.clone() });
                    }
                },
                ClientToServerCommand::Mode { channel } => {
                    let now = Utc::now();

                    sender.send(Reply::Mode { host: host.clone(), channel: channel.clone(), mode_string: "+tn".to_string() });
                    sender.send(Reply::ChannelModeIs { host: host.clone(), nick: connection_nick.clone(), channel: channel.clone(), mode_string: "+mtn1".to_string(), mode_arguments: "100".to_string() });
                    sender.send(Reply::CreationTime { host: host.clone(), nick: connection_nick.clone(), channel: channel.clone(), created_at: now.clone() });
                },
                ClientToServerCommand::Who { channel } => {
                    sender.send(Reply::WhoReply { host: host.clone(), channel: channel.clone(), nick: connection_nick.clone(), other_nick: "~vince".to_string(), client: "localhost".to_string() });
                    sender.send(Reply::EndOfWho { host: host.clone(), nick: connection_nick.clone(), channel: channel.clone() });
                },
                ClientToServerCommand::Nick { nick } => {   
                    connection_nick = nick.to_string();
                    connection_client = format!("{}!~{}@localhost", connection_nick, connection_nick);

                    sender.send(Reply::Welcome { host: host.clone(), nick: connection_nick.clone() });
                    sender.send(Reply::YourHost { host: host.clone(), nick: connection_nick.clone(), version: version.clone() });
                    sender.send(Reply::Created { host: host.clone(), nick: connection_nick.clone(), created_at: created_at.clone() });
                    sender.send(Reply::MyInfo { host: host.clone(), nick: connection_nick.clone(), version: version.clone(), user_modes: "r".to_string(), channel_modes: "i".to_string() });
                    sender.send(Reply::Support { host: host.clone(), nick: connection_nick.clone(), channel_len: 32 });
                    sender.send(Reply::LuserClient { host: host.clone(), nick: connection_nick.clone(), visible_users: 100, invisible_users: 20, servers: 1 });
                    sender.send(Reply::LuserOp { host: host.clone(), nick: connection_nick.clone(), operators: 1337 });
                    sender.send(Reply::LuserUnknown { host: host.clone(), nick: connection_nick.clone(), unknown: 7 });
                    sender.send(Reply::LuserChannels { host: host.clone(), nick: connection_nick.clone(), channels: 9999 });
                    sender.send(Reply::LuserMe { host: host.clone(), nick: connection_nick.clone(), clients: 900, servers: 1 });
                    sender.send(Reply::LocalUsers { host: host.clone(), nick: connection_nick.clone(), current: 845, max: 1000 });
                    sender.send(Reply::GlobalUsers { host: host.clone(), nick: connection_nick.clone(), current: 9832, max: 23455 });
                    sender.send(Reply::StatsDLine { host: host.clone(), nick: connection_nick.clone(), connections: 9998, clients: 9000, received: 99999 });

                    sender.send(Reply::MotdStart { host: host.clone(), nick: connection_nick.clone() });
                    // TODO proper configurable MOTD
                    sender.send(Reply::Motd { host: host.clone(), nick: connection_nick.clone(), line: "Foobar".to_string() });
                    sender.send(Reply::EndOfMotd { host: host.clone(), nick: connection_nick.clone() });
                },
                // these won't be passed down
                ClientToServerCommand::Unhandled { .. } => { },
                ClientToServerCommand::Ping { .. } => { },
                ClientToServerCommand::Pong {} => { },
                ClientToServerCommand::Quit {} => { }
            };
        }
    }
}
