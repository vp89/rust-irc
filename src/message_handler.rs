use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::{
    channels::ReceiverWrapper,
    context::{ChannelContext, ConnectionContext, ServerContext},
    handlers::{
        join::handle_join, mode::handle_mode, nick::handle_nick, part::handle_part,
        ping::handle_ping, privmsg::handle_privmsg, quit::handle_quit, user::handle_user,
    },
    message_parsing::{Command, Message, ReplySender},
    replies::Reply,
};

use crate::handlers::who::*;
use crate::result::Result;

pub async fn run<T>(
    server_context: &ServerContext,
    receiver_channel: &mut T,
    mut shutdown_receiver: Receiver<()>,
) -> Result<()>
where
    T: ReceiverWrapper<Message>,
{
    let mut connections = HashMap::new();
    let mut sender_channels = HashMap::new();
    let server_host = server_context.server_host.clone();
    let empty_str = &String::from("");
    let mut channels: HashMap<String, ChannelContext> = HashMap::new();

    loop {
        let received = tokio::select! {
            received = receiver_channel.receive() => match received {
                Some(r) => r,
                None => {
                    return Ok(());
                }
            },
            _ = shutdown_receiver.recv() => {
                return Ok(());
            }
        };

        // This should always be the first command received for any given connection
        // and thus the only one where there is no connection context available.
        // We can handle it here instead of in the match below so that the rest of the
        // commands can just deal with a ConnectionContext instead of an Option<ConnectionContext>
        if let Command::Connected { sender, client_ip } = &received.command {
            let ctx = ConnectionContext {
                connection_id: received.connection_id,
                nick: None,
                client: None,
                user: None,
                real_name: None,
                client_host: *client_ip,
            };
            connections.insert(received.connection_id, ctx);
            sender_channels.insert(received.connection_id, sender.clone());
            continue;
        }

        let conn_context = match connections.get(&received.connection_id) {
            Some(c) => c,
            None => {
                println!(
                    "Unexpected message sequence, received a {:?} message for {} before properly establishing a connection",
                    received.command,
                    received.connection_id
                );

                continue;
            }
        };

        let ctx_client = conn_context.client.as_ref().unwrap_or(empty_str);
        let ctx_nick = conn_context.nick.as_ref().unwrap_or(empty_str);

        let replies = match &received.command {
            Command::Disconnected => {
                if connections.remove(&received.connection_id).is_none() {
                    println!(
                        "Disconnected connection {} already removed",
                        &received.connection_id
                    );
                }

                if sender_channels.remove(&received.connection_id).is_none() {
                    println!(
                        "Disconnected connection {} already removed from sender channels",
                        &received.connection_id
                    );
                }

                None
            }
            Command::User {
                user,
                mode: _,
                realname,
            } => {
                let conn_context = match connections.get_mut(&received.connection_id) {
                    Some(c) => c,
                    None => {
                        continue;
                    }
                };

                handle_user(&server_host, user, realname, conn_context)
            }
            Command::Nick { nick, .. } => {
                let conn_context = match connections.get_mut(&received.connection_id) {
                    Some(c) => c,
                    None => {
                        continue;
                    }
                };

                handle_nick(
                    server_context,
                    &server_host,
                    nick,
                    &server_context.version,
                    &server_context.start_time,
                    conn_context,
                )
            }
            Command::Join { channels_to_join } => handle_join(
                &server_host,
                ctx_nick,
                ctx_client,
                conn_context,
                &mut channels,
                &connections,
                channels_to_join,
            ),
            Command::Part { channels_to_leave } => handle_part(
                &server_host,
                ctx_nick,
                ctx_client,
                conn_context,
                &mut channels,
                channels_to_leave,
            ),
            Command::Mode { channel } => handle_mode(&server_host, ctx_nick, channel, conn_context),
            Command::Who { mask, .. } => handle_who(
                mask,
                &server_host,
                ctx_nick,
                &channels,
                &connections,
                conn_context,
            ),
            Command::PrivMsg { channel, message } => handle_privmsg(
                &server_host,
                ctx_nick,
                channel,
                message,
                conn_context,
                &channels,
                &connections,
            ),
            Command::Quit { message } => {
                handle_quit(message, &mut channels, &connections, received.connection_id)
            }
            Command::Connected { .. } => None,
            Command::Unhandled { .. } => None,
            Command::Ping { token } => handle_ping(&server_host, ctx_nick, token, conn_context),
            Command::Pong {} => None,
        };

        if let Some(replies) = replies {
            send_replies(replies, &sender_channels).await
        }
    }
}

async fn send_replies(
    replies_per_user: HashMap<Uuid, Vec<Reply>>,
    sender_channels: &HashMap<Uuid, ReplySender>,
) {
    for (connection_id, replies) in replies_per_user {
        let sender = match sender_channels.get(&connection_id) {
            Some(sender) => &sender.0,
            None => {
                // TODO
                continue;
            }
        };

        for reply in replies {
            if let Err(e) = sender.send(reply).await {
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
    use std::collections::VecDeque;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use tokio::sync::mpsc::{self};

    #[tokio::test]
    pub async fn server_nickcommandsent_replystormissent() {
        // Arrange
        let (sender, mut test_receiver) = mpsc::channel(1000);
        let connection_id = Uuid::new_v4();

        let mut messages = VecDeque::new();
        messages.push_back(Message {
            source: None,
            command: Command::Connected {
                sender: ReplySender(sender),
                client_ip: Some(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    1234,
                ))),
            },
            connection_id,
        });

        messages.push_back(Message {
            source: None,
            command: Command::Nick {
                nick: Some("JOE".to_string()),
            },
            connection_id,
        });

        let mut receiver = FakeChannelReceiver {
            faked_messages: Box::new(messages),
            receive_count: 0,
        };

        let context = ServerContext {
            start_time: Utc::now(),
            server_host: "localhost".to_string(),
            version: "0.0.1".to_string(),
            ping_frequency: std::time::Duration::from_secs(60),
            motd_lines: vec![],
        };

        // Act
        let (_shutdown_sender, shutdown_receiver) = mpsc::channel(1);
        run(&context, &mut receiver, shutdown_receiver)
            .await
            .unwrap();

        // Assert
        assert_eq!(&3, &receiver.receive_count);

        let mut received = vec![];
        while let Ok(m) = test_receiver.try_recv() {
            received.push(m);
        }

        assert_eq!(15, received.len());
    }
}
