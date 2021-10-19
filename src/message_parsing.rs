use crate::error::Error::*;
use crate::result::Result;
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
    PrivMsg { channel: String, message: String },
    Pong,
    Quit,
}

// TODO this doesnt handle NICK params
impl ClientToServerMessage {
    pub fn from_str(s: &str, conn_uuid: Uuid) -> Result<Self> {
        let has_source = s.starts_with(':');
        let mut words = s.split_whitespace();

        let source = if has_source {
            words.next().map(|s| s.trim_start_matches(':').to_owned())
        } else {
            None
        };

        let mut raw_command = match words.next() {
            Some(s) => Ok(s),
            None => Err(MessageParsingErrorMissingCommand),
        }?;

        let uppercased = raw_command.to_uppercase();
        raw_command = uppercased.as_ref();

        let command = match raw_command {
            "PRIVMSG" => {
                let channel = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "channel".to_string(),
                    }),
                }?;

                let message = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "message".to_string(),
                    }),
                }?;

                ClientToServerCommand::PrivMsg { channel, message }
            }
            "NICK" => {
                let nick = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "nick".to_string(),
                    }),
                }?;

                ClientToServerCommand::Nick { nick }
            }
            "PING" => {
                let token = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "token".to_string(),
                    }),
                }?;

                ClientToServerCommand::Ping { token }
            }
            "JOIN" => {
                let raw_channels: String = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "channels".to_string(),
                    }),
                }?;

                let channels = raw_channels.split(',').map(|s| s.to_string()).collect();
                ClientToServerCommand::Join { channels }
            }
            "MODE" => {
                let channel = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "channel".to_string(),
                    }),
                }?;

                ClientToServerCommand::Mode { channel }
            }
            "WHO" => {
                let channel = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "channel".to_string(),
                    }),
                }?;

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

#[test]
fn client_to_server_justprefix_returnserror() {
    let raw_str = ":";
    ClientToServerMessage::from_str(raw_str, Uuid::new_v4()).expect_err("Expected error!");
    let raw_str = ":abc";
    ClientToServerMessage::from_str(raw_str, Uuid::new_v4()).expect_err("Expected error!");
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
fn client_to_server_lowercase_is_parsed() {
    let expected_nick = format!("Joe");
    let uuid = Uuid::new_v4();
    let expected_message = ClientToServerMessage {
        source: None,
        command: ClientToServerCommand::Nick {
            nick: expected_nick.clone(),
        },
        connection_uuid: uuid,
    };
    let raw_str = &format!("nick {}", expected_nick);
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
