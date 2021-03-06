use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

use crate::error::Error::*;
use crate::replies::Reply;
use crate::result::Result;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub source: Option<String>,
    pub command: Command,
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
pub enum Command {
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
        token: Option<String>,
    },
    Join {
        channels_to_join: Option<Vec<String>>,
    },
    Mode {
        channel: Option<String>,
    },
    Who {
        mask: Option<String>,
        only_operators: bool,
    },
    PrivMsg {
        channel: Option<String>,
        message: Option<String>,
    },
    User {
        user: Option<String>,
        mode: Option<String>,
        realname: Option<String>,
    },
    Pong,
    Quit {
        message: Option<String>,
    },
    Part {
        channels_to_leave: Option<Vec<String>>,
    },
}

// TODO this doesnt handle NICK params
impl Message {
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
                let channel = words
                    .next()
                    .map(|s| s.to_owned())
                    .filter(|s| s.starts_with('#'));

                let message = words
                    .map(|w| format!("{} ", w))
                    .collect::<String>()
                    .trim_start_matches(':')
                    .trim_end()
                    .to_string();

                let message = Some(message).filter(|m| !String::is_empty(m));

                Command::PrivMsg { channel, message }
            }
            "NICK" => {
                let nick = words.next().map(|s| s.to_owned());

                Command::Nick { nick }
            }
            "PING" => {
                let token = words.next().map(|s| s.trim_start_matches(':').to_owned());
                Command::Ping { token }
            }
            "JOIN" => {
                let channels_to_join: Option<Vec<String>> = words
                    .next()
                    .map(|s| s.to_owned().split(',').map(|s| s.to_string()).collect());
                Command::Join { channels_to_join }
            }
            "PART" => {
                let channels_to_leave: Option<Vec<String>> = words
                    .next()
                    .map(|s| s.to_owned().split(',').map(|s| s.to_string()).collect());
                Command::Part { channels_to_leave }
            }
            "MODE" => {
                let channel = words.next().map(|s| s.to_owned());
                Command::Mode { channel }
            }
            "WHO" => {
                let mask = words.next().map(|s| s.to_owned());

                let only_operators = match words.next() {
                    Some("o") => true,
                    Some(_) | None => false,
                };

                Command::Who {
                    mask,
                    only_operators,
                }
            }
            "USER" => {
                let user = words.next().map(|s| s.to_owned());
                let mode = words.next().map(|s| s.to_owned());

                // skip the "unused" argument
                words.next();

                let realname = words.next().map(|s| s.to_owned());

                Command::User {
                    user,
                    mode,
                    realname,
                }
            }
            "PONG" => Command::Pong,
            "QUIT" => {
                let message = words.next().map(|s| s.to_string());

                Command::Quit { message }
            }
            _ => Command::Unhandled,
        };

        let message = Message {
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
        Message::from_str(raw_str, Uuid::new_v4()).expect_err("Expected error!");
        let raw_str = ":abc";
        Message::from_str(raw_str, Uuid::new_v4()).expect_err("Expected error!");
    }

    #[test]
    fn message_parsing_nick_command_has_prefix_success() {
        let expected_nick = format!("Joe");
        let connection_id = Uuid::new_v4();
        let expected_message = Message {
            source: Some(format!("FOO")),
            command: Command::Nick {
                nick: Some(expected_nick.clone()),
            },
            connection_id,
        };
        let raw_str = &format!(
            ":{} NICK {}",
            expected_message.source.as_ref().unwrap(),
            expected_nick
        );

        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid prefix");
        let actual_source = message.source;
        let actual_command = message.command;
        assert_eq!(expected_message.source, actual_source);
        assert_eq!(expected_message.command, actual_command);
    }

    #[test]
    fn message_parsing_nick_command_no_prefix_success() {
        let expected_nick = format!("Joe");
        let connection_id = Uuid::new_v4();
        let expected_message = Message {
            source: None,
            command: Command::Nick {
                nick: Some(expected_nick.clone()),
            },
            connection_id,
        };
        let raw_str = &format!("NICK {}", expected_nick);
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid prefix");
        let actual_command = message.command;
        assert_eq!(expected_message.command, actual_command);
    }

    #[test]
    fn message_parsing_handles_lowercase_commands() {
        let expected_nick = format!("Joe");
        let connection_id = Uuid::new_v4();
        let expected_message = Message {
            source: None,
            command: Command::Nick {
                nick: Some(expected_nick.clone()),
            },
            connection_id,
        };
        let raw_str = &format!("nick {}", expected_nick);
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid prefix");
        let actual_command = message.command;
        assert_eq!(expected_message.command, actual_command);
    }

    #[test]
    fn message_parsing_valid_join_command_success() {
        let expected_channel = "foobar".to_string();
        let connection_id = Uuid::new_v4();
        let expected_message = Message {
            source: None,
            command: Command::Join {
                channels_to_join: Some(vec![expected_channel.clone()]),
            },
            connection_id,
        };
        let raw_str = &format!("JOIN {}", expected_channel);
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_join_multiple_channels_success() {
        let expected_channel_1 = "foobar".to_string();
        let expected_channel_2 = "barbaz".to_string();
        let connection_id = Uuid::new_v4();

        let expected_channels = vec![expected_channel_1.clone(), expected_channel_2.clone()];
        let expected_message = Message {
            source: None,
            command: Command::Join {
                channels_to_join: Some(expected_channels.clone()),
            },
            connection_id,
        };

        let raw_str = &format!("JOIN {}", expected_channels.join(","));
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_who_with_no_mask_success() {
        let connection_id = Uuid::new_v4();
        let expected_message = Message {
            source: None,
            command: Command::Who {
                mask: None,
                only_operators: false,
            },
            connection_id,
        };
        let raw_str = &format!("WHO");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_who_only_operators_defaults_to_false() {
        let connection_id = Uuid::new_v4();
        let expected_message = Message {
            source: None,
            command: Command::Who {
                mask: Some("#heythere".to_string()),
                only_operators: false,
            },
            connection_id,
        };
        let raw_str = &format!("WHO #heythere");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_who_only_operators_requested_success() {
        let connection_id = Uuid::new_v4();
        let expected_message = Message {
            source: None,
            command: Command::Who {
                mask: Some("#heythere".to_string()),
                only_operators: true,
            },
            connection_id,
        };
        let raw_str = &format!("WHO #heythere o");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_privmsg_multi_word_message_is_parsed() {
        let connection_id = Uuid::new_v4();
        let expected_message = Message {
            source: None,
            command: Command::PrivMsg {
                channel: Some("#blah".to_string()),
                message: Some("HI. THERE? HELLO!".to_string()),
            },
            connection_id,
        };
        let raw_str = &format!("PRIVMSG #blah :HI. THERE? HELLO!");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        assert_eq!(expected_message.command, message.command);
    }

    #[test]
    fn message_parsing_privmsg_channel_missing_returns_none() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PRIVMSG");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::PrivMsg {
                channel: None,
                message: None,
            },
            connection_id,
        };
        assert_eq!(expected, message);
    }

    #[test]
    fn message_parsing_privmsg_channel_missing_hash_errors() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PRIVMSG :foo");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::PrivMsg {
                channel: None,
                message: None,
            },
            connection_id,
        };
        assert_eq!(expected, message);
    }

    #[test_case("PRIVMSG #hey" ; "_errors")]
    #[test_case("PRIVMSG #hey " ; "_trailing_space_errors")]
    fn message_parsing_privmsg_message_missing_channel_is_returned(raw_str: &str) {
        let connection_id = Uuid::new_v4();
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::PrivMsg {
                channel: Some("#hey".to_string()),
                message: None,
            },
            connection_id,
        };

        assert_eq!(expected, message);
    }

    #[test]
    fn message_parsing_part_missing_channels() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PART");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::Part {
                channels_to_leave: None,
            },
            connection_id,
        };
        assert_eq!(expected, message);
    }

    #[test]
    fn message_parsing_part_single_channel_parses_correctly() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PART #foo");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::Part {
                channels_to_leave: Some(vec!["#foo".to_string()]),
            },
            connection_id,
        };

        assert_eq!(expected, message);
    }

    #[test]
    fn message_parsing_part_multiple_channels_parses_correctly() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PART #foo,#bar,#baz");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::Part {
                channels_to_leave: Some(vec![
                    "#foo".to_string(),
                    "#bar".to_string(),
                    "#baz".to_string(),
                ]),
            },
            connection_id,
        };

        assert_eq!(expected, message);
    }

    #[test]
    fn message_parsing_ping_token_provided_parses_correctly() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PING :foobar");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::Ping {
                token: Some("foobar".to_string()),
            },
            connection_id,
        };

        assert_eq!(expected, message);
    }

    #[test]
    fn message_parsing_ping_empty_token_colon_parses_correctly() {
        let connection_id = Uuid::new_v4();
        let raw_str = &format!("PING :");
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::Ping {
                token: Some("".to_string()),
            },
            connection_id,
        };

        assert_eq!(expected, message);
    }

    #[test_case("PING" ; "just_ping")]
    #[test_case("PING " ; "ping_with_trailing_space")]
    fn message_parsing_ping_missing_token_is_none(raw_str: &str) {
        let connection_id = Uuid::new_v4();
        let message =
            Message::from_str(raw_str, connection_id).expect("Failed to parse valid message");
        let expected = Message {
            source: None,
            command: Command::Ping { token: None },
            connection_id,
        };

        assert_eq!(expected, message);
    }
}
