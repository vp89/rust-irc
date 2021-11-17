use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use crate::error::Error::*;
use crate::replies::Reply;
use crate::result::Result;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct ClientToServerMessage {
    pub source: Option<String>,
    pub command: ClientToServerCommand,
    pub connection_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ReplySender(pub Sender<Reply>);

// this is just implemented to keep the compiler happy
// we will never need to do equality comparison on this
impl PartialEq for ReplySender {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClientToServerCommand {
    Unhandled,
    Disconnected,
    Connected {
        sender: ReplySender,
        client_ip: Option<SocketAddr>,
    },
    Nick {
        nick: Option<String>,
    },
    Ping {
        token: String,
    },
    Join {
        channels_to_join: Vec<String>,
    },
    Mode {
        channel: String,
    },
    Who {
        mask: Option<String>,
        only_operators: bool,
    },
    PrivMsg {
        channel: String,
        message: String,
    },
    User {
        user: String,
        mode: String,
        realname: String,
    },
    Pong,
    Quit {
        message: Option<String>
    },
}

// TODO this doesnt handle NICK params
impl ClientToServerMessage {
    pub fn from_str(s: &str, connection_id: Uuid) -> Result<Self> {
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

                let satisfies_guard = &' ';
                match (&channel.chars().next(), satisfies_guard) {
                    (None, c) | (Some(c), _) if c != &'#' => {
                        Err(MessageParsingErrorInvalidChannelFormat {
                            provided_channel: channel.to_string(),
                        })
                    }
                    _ => Ok(()),
                }?;

                let message = words
                    .map(|w| format!("{} ", w))
                    .collect::<String>()
                    .trim_start_matches(':')
                    .trim_end()
                    .to_string();

                if message.is_empty() {
                    return Err(MessageParsingErrorMissingParameter {
                        param_name: "message".to_string(),
                    });
                };

                ClientToServerCommand::PrivMsg { channel, message }
            }
            "NICK" => {
                let nick = words.next().map(|s| s.to_owned());

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

                let channels_to_join = raw_channels.split(',').map(|s| s.to_string()).collect();
                ClientToServerCommand::Join { channels_to_join }
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
                let mask = words.next().map(|s| s.to_owned());

                let only_operators = match words.next() {
                    Some("o") => true,
                    Some(_) | None => false,
                };

                ClientToServerCommand::Who {
                    mask,
                    only_operators,
                }
            }
            "USER" => {
                let user = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "user".to_string(),
                    }),
                }?;

                let mode = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "mode".to_string(),
                    }),
                }?;

                // skip the "unused" argument
                words.next();

                let realname = match words.next() {
                    Some(s) => Ok(s.to_owned()),
                    None => Err(MessageParsingErrorMissingParameter {
                        param_name: "realname".to_string(),
                    }),
                }?;

                ClientToServerCommand::User {
                    user,
                    mode,
                    realname,
                }
            }
            "PONG" => ClientToServerCommand::Pong,
            "QUIT" => {
                let message = words.next().map(|s| s.to_string() );

                ClientToServerCommand::Quit { message }
            },
            _ => ClientToServerCommand::Unhandled,
        };

        let message = ClientToServerMessage {
            source,
            command,
            connection_id,
        };

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn messageparsing_missingcommand_errors() {
        let raw_str = ":";
        ClientToServerMessage::from_str(raw_str, Uuid::new_v4()).expect_err("Expected error!");
        let raw_str = ":abc";
        ClientToServerMessage::from_str(raw_str, Uuid::new_v4()).expect_err("Expected error!");
    }

    #[test]
    fn message_parsing_nick_command_has_prefix_success() {
        let expected_nick = format!("Joe");
        let connection_id = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: Some(format!("FOO")),
            command: ClientToServerCommand::Nick {
                nick: Some(expected_nick.clone()),
            },
            connection_id,
        };
        let raw_str = &format!(
            ":{} NICK {}",
            expected_message.source.as_ref().unwrap(),
            expected_nick
        );

        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid prefix");
        let actual_source = message.source;
        let actual_command = message.command;
        assert_eq!(expected_message.source, actual_source);
        assert_eq!(expected_message.command, actual_command);
    }

    #[test]
    fn message_parsing_nick_command_no_prefix_success() {
        let expected_nick = format!("Joe");
        let connection_id = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Nick {
                nick: Some(expected_nick.clone()),
            },
            connection_id,
        };
        let raw_str = &format!("NICK {}", expected_nick);
        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid prefix");
        let actual_command = message.command;
        assert_eq!(expected_message.command, actual_command);
    }

    #[test]
    fn message_parsing_handles_lowercase_commands() {
        let expected_nick = format!("Joe");
        let connection_id = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Nick {
                nick: Some(expected_nick.clone()),
            },
            connection_id,
        };
        let raw_str = &format!("nick {}", expected_nick);
        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid prefix");
        let actual_command = message.command;
        assert_eq!(expected_message.command, actual_command);
    }

    #[test]
    fn message_parsing_valid_join_command_success() {
        let expected_channel = "foobar".to_string();
        let connection_id = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Join {
                channels_to_join: vec![expected_channel.clone()],
            },
            connection_id,
        };
        let raw_str = &format!("JOIN {}", expected_channel);
        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_join_multiple_channels_success() {
        let expected_channel_1 = "foobar".to_string();
        let expected_channel_2 = "barbaz".to_string();
        let connection_id = Uuid::new_v4();

        let expected_channels = vec![expected_channel_1.clone(), expected_channel_2.clone()];
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Join {
                channels_to_join: expected_channels.clone(),
            },
            connection_id,
        };

        let raw_str = &format!("JOIN {}", expected_channels.join(","));
        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_who_with_no_mask_success() {
        let connection_id = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Who {
                mask: None,
                only_operators: false,
            },
            connection_id,
        };
        let raw_str = &format!("WHO");
        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_who_only_operators_defaults_to_false() {
        let connection_id = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Who {
                mask: Some("#heythere".to_string()),
                only_operators: false,
            },
            connection_id,
        };
        let raw_str = &format!("WHO #heythere");
        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_who_only_operators_requested_success() {
        let connection_id = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Who {
                mask: Some("#heythere".to_string()),
                only_operators: true,
            },
            connection_id,
        };
        let raw_str = &format!("WHO #heythere o");
        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_privmsg_multi_word_message_is_parsed() {
        let connection_id = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::PrivMsg {
                channel: "#blah".to_string(),
                message: "HI. THERE? HELLO!".to_string(),
            },
            connection_id,
        };
        let raw_str = &format!("PRIVMSG #blah :HI. THERE? HELLO!");
        let message = ClientToServerMessage::from_str(raw_str, connection_id)
            .expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_privmsg_channel_missing_errors() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PRIVMSG");
        let message = ClientToServerMessage::from_str(raw_str, connection_id);
        let expected = Err(MessageParsingErrorMissingParameter {
            param_name: "channel".to_string(),
        });
        assert_eq!(expected, message);
    }

    #[test]
    fn message_parsing_privmsg_channel_missing_hash_errors() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PRIVMSG :foo");
        let message = ClientToServerMessage::from_str(raw_str, connection_id);
        let expected = Err(MessageParsingErrorInvalidChannelFormat {
            provided_channel: ":foo".to_string(),
        });
        assert_eq!(expected, message);
    }

    #[test_case("PRIVMSG #hey" ; "_errors")]
    #[test_case("PRIVMSG #hey " ; "_trailing_space_errors")]
    fn message_parsing_privmsg_message_missing(raw_str: &str) {
        let connection_id = Uuid::new_v4();
        let message = ClientToServerMessage::from_str(raw_str, connection_id);
        let expected = Err(MessageParsingErrorMissingParameter {
            param_name: "message".to_string(),
        });
        assert_eq!(expected, message);
    }
}
