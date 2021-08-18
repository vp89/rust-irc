use std::fmt::Display;
use std::{str::FromStr};

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ClientToServerMessage<'a> {
    pub source: Option<String>,
    pub command: ClientToServerCommand,
    pub params: &'a str // TODO remove these?
}

#[derive(Debug, PartialEq)]
pub enum ClientToServerCommand {
    Unhandled,
    Nick(NickCommand),
    Quit
}

#[derive(Debug, PartialEq)]
pub struct NickCommand {
    pub nick: String
}

#[derive(Debug)]
pub struct ServerToClientMessage {
    pub source: Source
}

#[derive(Debug)]
pub struct ServerReplyMessage<'a> {
    pub source: &'a str,
    pub target: &'a str,
    pub reply_number: &'a str, // TODO this sucks
    pub reply: NumericReply<'a>
}

#[derive(Debug)]
pub enum NumericReply<'a> {
    RplWelcome(RplWelcome<'a>),
    RplCreated(RplCreated<'a>)
}

#[derive(Debug)]
pub struct RplWelcome<'a> {
    pub welcome_message: &'a str,
    pub nick: &'a str
}

#[derive(Debug)]
pub struct RplCreated<'a> {
    pub created_message: &'a str,
    pub created_at: &'a DateTime<Utc>
}

#[derive(Debug)]
pub enum Source {
    Server(String),
    Client(ClientSource)
}

#[derive(Debug)]
pub struct ClientSource {
    pub nick: String,
    pub user: String,
    pub host: String
}

#[derive(Debug)]
pub struct ServerToServerMessage {
    pub source: String
}

// TODO this doesnt handle NICK params
impl FromStr for ClientToServerMessage<'_> {
    type Err = (); // TODO?

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
                ClientToServerCommand::Nick(NickCommand {
                    nick
                })
            },
            "QUIT" => ClientToServerCommand::Quit,
            _ => ClientToServerCommand::Unhandled
        };

        let message = ClientToServerMessage {
            source,
            command,
            params: ""
        };

        Ok(message)
    }
}

impl Display for ServerToClientMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw_source = match &self.source {
            Source::Server(s) => s.to_owned(), // TODO remove to_owned
            Source::Client(s) => format!("{}!{}@{}", s.nick, s.user, s.host) 
        };

        write!(f, ":{}", raw_source)
    }
}

impl Display for ServerReplyMessage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            ":{} {} {} {}",
            self.source,
            self.reply_number,
            self.target,
            self.reply.to_string())
    }
}

impl Display for NumericReply<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            NumericReply::RplWelcome(r) => {
                write!(
                    f,
                    ":{} {}",
                    r.welcome_message,
                    r.nick)
            },
            NumericReply::RplCreated(r) =>
            {
                write!(
                    f,
                    ":{} {}",
                    r.created_message,
                    r.created_at
                )
            }
        }
    }
}

#[test]
fn client_to_server_has_prefix_is_parsed() {
    let expected_nick = format!("Joe");
    let expected_message = ClientToServerMessage {
        source: Some(format!("FOO")),
        command: ClientToServerCommand::Nick(NickCommand {
            nick: expected_nick.clone()
        }),
        params: ""
    };
    let raw_str = &format!(
        ":{} NICK {}",
        expected_message.source.as_ref().unwrap(),
        expected_nick);

    let message = ClientToServerMessage::from_str(raw_str).expect("Failed to parse valid prefix");
    let actual_source = message.source;
    let actual_command = message.command;
    assert_eq!(expected_message.source, actual_source);
    assert_eq!(expected_message.command, actual_command);
}

#[test]
fn client_to_server_no_prefix_is_parsed() {
    let expected_nick = format!("Joe");
    let expected_message = ClientToServerMessage {
        source: None,
        command: ClientToServerCommand::Nick(NickCommand {
            nick: expected_nick.clone()
        }),
        params: ""
    };
    let raw_str = &format!("NICK {}", expected_nick);
    let message = ClientToServerMessage::from_str(raw_str).expect("Failed to parse valid prefix");
    let actual_command = message.command;
    assert_eq!(expected_message.command, actual_command);
}

#[test]
fn server_to_client_from_server_is_valid() {
    let source = "foobar".to_owned();
    let message = ServerToClientMessage {
        source: Source::Server(source.to_owned())
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
        source: Source::Client(ClientSource {
            nick: nick.to_owned(),
            user: user.to_owned(),
            host: host.to_owned()
        })
    };
    let actual = message.to_string();
    let expected = format!(":{}!{}@{}", nick, user, host);
    assert_eq!(expected, actual);
}

#[test]
fn rpl_welcome_prints_correctly() {
    let reply = ServerReplyMessage {
        source: "localhost",
        target: "JIM",
        reply_number: "001",
        reply: NumericReply::RplWelcome(RplWelcome {
            welcome_message: "HELLO WORLD",
            nick: "JIM"
        })
    };

    let actual = reply.to_string();
    let expected = ":localhost 001 JIM :HELLO WORLD JIM";
    assert_eq!(expected, actual);
}
