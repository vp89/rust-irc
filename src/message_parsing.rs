use crate::error::Error::*;
use crate::result::Result;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct ClientToServerMessage {
    pub source: Option<String>,
    pub command: ClientToServerCommand,
    pub connection_uuid: Uuid,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClientToServerCommand {
    Unhandled,
    Nick {
        nick: String,
    },
    Ping {
        token: String,
    },
    Join {
        channels: Vec<String>,
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
                let mask = match words.next() {
                    Some(s) => Some(s.to_owned()),
                    None => None,
                };

                let only_operators = match words.next() {
                    Some("o") => true,
                    Some(_) | None => false,
                };

                ClientToServerCommand::Who { mask, only_operators }
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
    fn message_parsing_nick_command_no_prefix_success() {
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
    fn message_parsing_handles_lowercase_commands() {
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
    fn message_parsing_valid_join_command_success() {
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
    fn message_parsing_join_multiple_channels_success() {
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
    fn message_parsing_who_with_no_mask_success() {
        let uuid = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Who { mask: None, only_operators: false },
            connection_uuid: uuid,
        };
        let raw_str = &format!("WHO");
        let message =
            ClientToServerMessage::from_str(raw_str, uuid).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_who_only_operators_defaults_to_false() {
        let uuid = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Who {
                mask: Some("#heythere".to_string()),
                only_operators: false,
            },
            connection_uuid: uuid,
        };
        let raw_str = &format!("WHO #heythere");
        let message =
            ClientToServerMessage::from_str(raw_str, uuid).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_who_only_operators_requested_success() {
        let uuid = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::Who {
                mask: Some("#heythere".to_string()),
                only_operators: true,
            },
            connection_uuid: uuid,
        };
        let raw_str = &format!("WHO #heythere o");
        let message =
            ClientToServerMessage::from_str(raw_str, uuid).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_privmsg_multi_word_message_is_parsed() {
        let uuid = Uuid::new_v4();
        let expected_message = ClientToServerMessage {
            source: None,
            command: ClientToServerCommand::PrivMsg {
                channel: "#blah".to_string(),
                message: "HI. THERE? HELLO!".to_string(),
            },
            connection_uuid: uuid,
        };
        let raw_str = &format!("PRIVMSG #blah :HI. THERE? HELLO!");
        let message =
            ClientToServerMessage::from_str(raw_str, uuid).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_privmsg_channel_missing_errors() {
        let uuid = Uuid::new_v4();
        let raw_str = &format!("PRIVMSG");
        let message = ClientToServerMessage::from_str(raw_str, uuid);
        let expected = Err(MessageParsingErrorMissingParameter {
            param_name: "channel".to_string(),
        });
        assert_eq!(expected, message);
    }

    #[test]
    fn message_parsing_privmsg_channel_missing_hash_errors() {
        let uuid = Uuid::new_v4();
        let raw_str = &format!("PRIVMSG :foo");
        let message = ClientToServerMessage::from_str(raw_str, uuid);
        let expected = Err(MessageParsingErrorInvalidChannelFormat {
            provided_channel: ":foo".to_string(),
        });
        assert_eq!(expected, message);
    }

    #[test_case("PRIVMSG #hey" ; "_errors")]
    #[test_case("PRIVMSG #hey " ; "_trailing_space_errors")]
    fn message_parsing_privmsg_message_missing(raw_str: &str) {
        let uuid = Uuid::new_v4();
        let message = ClientToServerMessage::from_str(raw_str, uuid);
        let expected = Err(MessageParsingErrorMissingParameter {
            param_name: "message".to_string(),
        });
        assert_eq!(expected, message);
    }
}
