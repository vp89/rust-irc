use std::{collections::HashMap, sync::{Arc, Mutex, RwLock, mpsc::{Receiver}}};
use chrono::Utc;
use uuid::Uuid;

use crate::{ConnectionContext, ServerContext, message_parsing::{ClientToServerCommand, ClientToServerMessage}, replies::Reply};

pub fn run_server(
    context: ServerContext,
    receiver_channel: Receiver<ClientToServerMessage>,
    connections: Arc<RwLock<HashMap<Uuid, ConnectionContext>>>) {

    /*
    connections
        .read()
        .unwrap()
        .get(&Uuid::new_v4())
        .unwrap()
        .client_sender_channel
        .lock()
        .unwrap()
        .send("BLARGH!".to_string());
    */
    
    let host = &context.host;
    // TODO pluck these out of the connection context
    let mut connection_nick = "".to_string();
    let mut connection_client = "".to_string();

    for received in receiver_channel.recv() {
        let version = &context.version;
        let created_at = &context.start_time;

        match &received.command {
            ClientToServerCommand::Join { channels} => {
                let now = Utc::now();

                for channel in channels {
                    // TODO grab connection sender and send here
                    /*
                    Reply::Join { client: &connection_client, channel },
                    // TODO have Nick available here
                    // TODO persist the channel metadata
                    Reply::Topic { host, nick: &connection_nick, channel, topic: "foobar topic" },
                    Reply::TopicWhoTime { host, channel, nick: &connection_nick, set_at: &now },
                    Reply::NamReply { host, channel, nick: &connection_nick },
                    */
                }
            },
            ClientToServerCommand::Mode { channel } => {
                let now = Utc::now();

                // TODO grab connection sender and send here
                /*
                Reply::Mode { host, channel, mode_string: "+tn" }
                Reply::ChannelModeIs { host, nick: &connection_nick, channel, mode_string: "+mtn1", mode_arguments: "100" }
                Reply::CreationTime { host, nick: &connection_nick, channel, created_at: &now }
                */
            },
            ClientToServerCommand::Who { channel } => {
                // TODO grab connection sender and send here
                /*
                Reply::WhoReply { host, channel, nick: &connection_nick, other_nick: "~vince", client: "localhost" },
                Reply::EndOfWho { host, nick: &connection_nick, channel },
                */
            },
            ClientToServerCommand::Nick { nick } => {   
                connection_nick = nick.to_string();
                connection_client = format!("{}!~{}@localhost", connection_nick, connection_nick);

                // TODO grab connection sender and send here
                /*
                    Reply::Welcome { host, nick },
                    Reply::YourHost { host, nick, version },
                    Reply::Created { host, nick, created_at },
                    Reply::MyInfo { host, nick, version, user_modes: "r", channel_modes: "i" },
                    Reply::Support { host, nick, channel_len: 32 },
                    Reply::LuserClient { host, nick, visible_users: 100, invisible_users: 20, servers: 1 },
                    Reply::LuserOp { host, nick, operators: 1337 },
                    Reply::LuserUnknown { host, nick, unknown: 7 },
                    Reply::LuserChannels { host, nick, channels: 9999 },
                    Reply::LuserMe { host, nick, clients: 900, servers: 1 },
                    Reply::LocalUsers { host, nick, current: 845, max: 1000 },
                    Reply::GlobalUsers { host, nick, current: 9832, max: 23455 },
                    Reply::StatsDLine { host, nick, connections: 9998, clients: 9000, received: 99999 }
                */

                /*
                Reply::MotdStart { host, nick }
                // TODO proper configurable MOTD
                Reply::Motd { host, nick, line: "Foobar" };
                Reply::EndOfMotd { host, nick }
                */
            },
            // these won't be passed down
            ClientToServerCommand::Unhandled { .. } => { },
            ClientToServerCommand::Ping { .. } => { },
            ClientToServerCommand::Pong {} => { },
            ClientToServerCommand::Quit {} => { }
        };
    }
}
