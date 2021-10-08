use std::{collections::HashMap, io::{Error, ErrorKind}, sync::{Arc, RwLock, mpsc::{Receiver}}};
use chrono::Utc;
use uuid::Uuid;

use crate::{ConnectionContext, ServerContext, message_parsing::{ClientToServerCommand, ClientToServerMessage}, replies::Reply};

pub fn run_server(
    context: &ServerContext,
    receiver_channel: Receiver<ClientToServerMessage>,
    connections: Arc<RwLock<HashMap<Uuid, ConnectionContext>>>) -> std::io::Result<()> {
    
    let host = context.host.clone();

    loop {
        let received = match receiver_channel.recv() {
            Ok(reply) => reply,
            Err(_e) => return Err(Error::new(ErrorKind::BrokenPipe, "Sender has disconnected"))
        };
        
        let conn_read = connections.read().unwrap();
        let conn_ctx = conn_read.get(&received.connection_uuid).unwrap();

        let ctx_version = &context.version;
        let ctx_created_at = &context.start_time;
        let ctx_client = &conn_ctx.client;
        let ctx_nick = &conn_ctx.nick;
        let ctx_sender = conn_ctx.client_sender_channel.lock().unwrap().clone();

        let result = match &received.command {
            ClientToServerCommand::Join { channels} => {
                let now = Utc::now();

                for channel in channels {
                    ctx_sender.send(Reply::Join { client: ctx_client.clone(), channel: channel.clone() });
                    // TODO have Nick available here
                    // TODO persist the channel metadata
                    ctx_sender.send(Reply::Topic { host: host.clone(), nick: ctx_nick.clone(), channel: channel.clone(), topic: "foobar topic".to_string() });
                    ctx_sender.send(Reply::TopicWhoTime { host: host.clone(), channel: channel.clone(), nick: ctx_nick.clone(), set_at: now.clone() });
                    ctx_sender.send(Reply::NamReply { host: host.clone(), channel: channel.clone(), nick: ctx_nick.clone() });
                }
            },
            ClientToServerCommand::Mode { channel } => {
                let now = Utc::now();

                ctx_sender.send(Reply::Mode { host: host.clone(), channel: channel.clone(), mode_string: "+tn".to_string() });
                ctx_sender.send(Reply::ChannelModeIs { host: host.clone(), nick: ctx_nick.clone(), channel: channel.clone(), mode_string: "+mtn1".to_string(), mode_arguments: "100".to_string() });
                ctx_sender.send(Reply::CreationTime { host: host.clone(), nick: ctx_nick.clone(), channel: channel.clone(), created_at: now.clone() });
            },
            ClientToServerCommand::Who { channel } => {
                ctx_sender.send(Reply::WhoReply { host: host.clone(), channel: channel.clone(), nick: ctx_nick.clone(), other_nick: "~vince".to_string(), client: "localhost".to_string() });
                ctx_sender.send(Reply::EndOfWho { host: host.clone(), nick: ctx_nick.clone(), channel: channel.clone() });
            },
            ClientToServerCommand::Nick { nick } => {
                let mut conn_write = connections.write().unwrap();
                let writable_ctx = conn_write.get_mut(&received.connection_uuid).unwrap();
                writable_ctx.nick = nick.to_string();
                writable_ctx.client = format!("{}!~{}@localhost", nick, nick); 

                ctx_sender.send(Reply::Welcome { host: host.clone(), nick: nick.clone() });
                ctx_sender.send(Reply::YourHost { host: host.clone(), nick: nick.clone(), version: ctx_version.clone() });
                ctx_sender.send(Reply::Created { host: host.clone(), nick: nick.clone(), created_at: ctx_created_at.clone() });
                ctx_sender.send(Reply::MyInfo { host: host.clone(), nick: nick.clone(), version: ctx_version.clone(), user_modes: "r".to_string(), channel_modes: "i".to_string() });
                ctx_sender.send(Reply::Support { host: host.clone(), nick: nick.clone(), channel_len: 32 });
                ctx_sender.send(Reply::LuserClient { host: host.clone(), nick: nick.clone(), visible_users: 100, invisible_users: 20, servers: 1 });
                ctx_sender.send(Reply::LuserOp { host: host.clone(), nick: nick.clone(), operators: 1337 });
                ctx_sender.send(Reply::LuserUnknown { host: host.clone(), nick: nick.clone(), unknown: 7 });
                ctx_sender.send(Reply::LuserChannels { host: host.clone(), nick: nick.clone(), channels: 9999 });
                ctx_sender.send(Reply::LuserMe { host: host.clone(), nick: nick.clone(), clients: 900, servers: 1 });
                ctx_sender.send(Reply::LocalUsers { host: host.clone(), nick: nick.clone(), current: 845, max: 1000 });
                ctx_sender.send(Reply::GlobalUsers { host: host.clone(), nick: nick.clone(), current: 9832, max: 23455 });
                ctx_sender.send(Reply::StatsDLine { host: host.clone(), nick: nick.clone(), connections: 9998, clients: 9000, received: 99999 });

                ctx_sender.send(Reply::MotdStart { host: host.clone(), nick: nick.clone() });
                // TODO proper configurable MOTD
                ctx_sender.send(Reply::Motd { host: host.clone(), nick: nick.clone(), line: "Foobar".to_string() });
                ctx_sender.send(Reply::EndOfMotd { host: host.clone(), nick: nick.clone() });
            },
            // these won't be passed down
            ClientToServerCommand::Unhandled { .. } => { },
            ClientToServerCommand::Ping { .. } => { },
            ClientToServerCommand::Pong {} => { },
            ClientToServerCommand::Quit {} => { }
        };
    }
}
