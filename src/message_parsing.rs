use std::fmt::Display;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ClientToServerMessage {
    pub source: Option<String>,
    pub command: ClientToServerCommand,
    pub connection_uuid: Uuid,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClientToServerCommand {
    Unhandled,
    Nick { nick: String },
    Ping { token: String },
    Join { channels: Vec<String> },
    Mode { channel: String },
    Who { channel: String },
    Pong,
    Quit,
}

#[derive(Debug)]
pub struct ServerToClientMessage {
    pub source: Source,
}

#[derive(Debug)]
pub enum Source {
    Server(String),
    Client {
        nick: String,
        user: String,
        host: String,
    },
}

#[derive(Debug)]
pub struct ServerToServerMessage {
    pub source: String,
}

// TODO this doesnt handle NICK params
impl ClientToServerMessage {
    pub fn from_str(s: &str, conn_uuid: Uuid) -> Result<Self, ()> {
        let has_source = s.starts_with(':');
        let mut words = s.split_whitespace();

        let source = if has_source {
            let raw_source = words.next().unwrap().trim_start_matches(':').to_owned(); // TODO fix this??
            Some(raw_source)
        } else {
            None
        };

        // TODO case sensitivity?
        let raw_command = words.next().unwrap(); // TODO remove unwrap

        let command = match raw_command {
            "NICK" => {
                let nick = words.next().unwrap().to_owned(); // TODO handle error
                ClientToServerCommand::Nick { nick }
            }
            "PING" => {
                let token = words.next().unwrap().to_owned(); // TODO handle error
                ClientToServerCommand::Ping { token }
            }
            "JOIN" => {
                let raw_channels: String = words.next().unwrap().to_owned();
                let channels = raw_channels.split(',').map(|s| s.to_string()).collect();
                ClientToServerCommand::Join { channels }
            }
            "MODE" => {
                let channel = words.next().unwrap().to_owned(); // TODO handle error
                ClientToServerCommand::Mode { channel }
            }
            "WHO" => {
                let channel = words.next().unwrap().to_owned(); // TODO handle error
                ClientToServerCommand::Who { channel }
            }
            "PONG" => ClientToServerCommand::Pong,
            "QUIT" => ClientToServerCommand::Quit,
            _ => ClientToServerCommand::Unhandled,
        };

        let message = ClientToServerMessage {
            source,
            command,
            connection_uuid: conn_uuid,
        };

        Ok(message)
    }
}

impl Display for ServerToClientMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw_source = match &self.source {
            Source::Server(s) => s.to_owned(),
            Source::Client { nick, user, host } => format!("{}!{}@{}", nick, user, host),
        };

        write!(f, ":{}", raw_source)
    }
}

#[test]
fn client_to_server_has_prefix_is_parsed() {
    let expected_nick = format!("Joe");
    let uuid = Uuid::new_v4();
    let expected_message = ClientToServerMessage {
        source: Some(format!("FOO")),
        command: ClientToServerCommand::Nick {
            nick: expected_nick.clone(),
        },
        connection_uuid: uuid,
    };
    let raw_str = &format!(
        ":{} NICK {}",
        expected_message.source.as_ref().unwrap(),
        expected_nick
    );

    let message =
        ClientToServerMessage::from_str(raw_str, uuid).expect("Failed to parse valid prefix");
    let actual_source = message.source;
    let actual_command = message.command;
    assert_eq!(expected_message.source, actual_source);
    assert_eq!(expected_message.command, actual_command);
}

#[test]
fn client_to_server_no_prefix_is_parsed() {
    let expected_nick = format!("Joe");
    let uuid = Uuid::new_v4();
    let expected_message = ClientToServerMessage {
        source: None,
        command: ClientToServerCommand::Nick {
            nick: expected_nick.clone(),
        },
        connection_uuid: uuid,
    };
    let raw_str = &format!("NICK {}", expected_nick);
    let message =
        ClientToServerMessage::from_str(raw_str, uuid).expect("Failed to parse valid prefix");
    let actual_command = message.command;
    assert_eq!(expected_message.command, actual_command);
}

#[test]
fn from_client_valid_join_is_parsed() {
    let expected_channel = "foobar".to_string();
    let uuid = Uuid::new_v4();
    let expected_message = ClientToServerMessage {
        source: None,
        command: ClientToServerCommand::Join {
            channels: vec![expected_channel.clone()],
        },
        connection_uuid: uuid,
    };
    let raw_str = &format!("JOIN {}", expected_channel);
    let message =
        ClientToServerMessage::from_str(raw_str, uuid).expect("Failed to parse valid message");
    assert_eq!(expected_message.command, message.command);
}

#[test]
fn from_client_join_multiplechannels_is_parsed() {
    let expected_channel_1 = "foobar".to_string();
    let expected_channel_2 = "barbaz".to_string();
    let uuid = Uuid::new_v4();

    let expected_channels = vec![expected_channel_1.clone(), expected_channel_2.clone()];
    let expected_message = ClientToServerMessage {
        source: None,
        command: ClientToServerCommand::Join {
            channels: expected_channels.clone(),
        },
        connection_uuid: uuid,
    };

    let raw_str = &format!("JOIN {}", expected_channels.join(","));
    let message =
        ClientToServerMessage::from_str(raw_str, uuid).expect("Failed to parse valid message");
    assert_eq!(expected_message.command, message.command);
}

#[test]
fn server_to_client_from_server_is_valid() {
    let source = "foobar".to_owned();
    let message = ServerToClientMessage {
        source: Source::Server(source.to_owned()),
    };
    let actual = message.to_string();
    let expected = format!(":{}", source);
    assert_eq!(expected, actual);
}

#[test]
fn server_to_client_from_client_is_valid() {
    let nick = "foo";
    let user = "bar";
    let host = "baz";
    let message = ServerToClientMessage {
        source: Source::Client {
            nick: nick.to_owned(),
            user: user.to_owned(),
            host: host.to_owned(),
        },
    };
    let actual = message.to_string();
    let expected = format!(":{}!{}@{}", nick, user, host);
    assert_eq!(expected, actual);
}
