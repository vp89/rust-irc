use std::fmt::Display;
use std::{str::FromStr};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ClientToServerMessage {
    pub source: Option<String>,
    pub command: ClientToServerCommand
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

pub trait ReplyMessage : ToString {
    const NUMBER: &'static str;
}

impl Display for ReplyWelcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{} {} {} :{} {}", self.host, ReplyWelcome::NUMBER, self.nick, self.welcome_message, self.nick)
    }
}

impl Display for ReplyYourHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{} {} {} :Your host is {}, running version {}", self.host, ReplyYourHost::NUMBER, self.nick, self.host, self.version)
    }
}

impl Display for ReplyCreated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{} {} {} :{} {}", self.host, ReplyCreated::NUMBER, self.nick, self.created_message, self.created_at)
    }
}

impl Display for ReplyMyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{} {} {} {} {} {} {}", self.host, ReplyMyInfo::NUMBER, self.nick, self.host, self.version, self.available_user_modes, self.available_channel_modes)
    }
}

impl Display for ReplySupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{} {} {} CHANNELLEN={} :are supported by this server", self.host, ReplySupport::NUMBER, self.nick, self.channel_len)
    }
}

impl Display for ReplyLUserClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{} {} {} :There are {} users and {} invisible on {} servers", self.host, ReplyLUserClient::NUMBER, self.nick, self.users, self.invisible_users, self.servers)
    }
}

impl Display for ReplyLUserOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{} {} {} {} :IRC Operators online", self.host, ReplyLUserOp::NUMBER, self.nick, self.operators)
    }
}

impl Display for ReplyLUserUnknown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{} {} {} {} :unknown connection(s)", self.host, ReplyLUserUnknown::NUMBER, self.nick, self.unknown)
    }
}

impl ReplyMessage for ReplyWelcome { const NUMBER: &'static str = "001"; }
impl ReplyMessage for ReplyYourHost { const NUMBER: &'static str = "002"; }
impl ReplyMessage for ReplyCreated { const NUMBER: &'static str = "003"; }
impl ReplyMessage for ReplyMyInfo { const NUMBER: &'static str = "004"; }
impl ReplyMessage for ReplySupport { const NUMBER: &'static str = "005"; }
impl ReplyMessage for ReplyLUserClient { const NUMBER: &'static str = "251"; }
impl ReplyMessage for ReplyLUserOp { const NUMBER: &'static str = "252"; }
impl ReplyMessage for ReplyLUserUnknown { const NUMBER: &'static str = "253"; }

pub struct ReplyWelcome {
    pub host: String,
    pub welcome_message: String,
    pub nick: String
}

pub struct ReplyYourHost {
    pub host: String,
    pub nick: String,
    pub version: String,
}

pub struct ReplyCreated {
    pub host: String,
    pub nick: String,
    pub created_message: String,
    pub created_at: DateTime<Utc>
}

pub struct ReplyMyInfo {
    pub host: String,
    pub nick: String,
    pub version: String,
    pub available_user_modes: String, // TODO set this up properly
    pub available_channel_modes: String, // TODO set this up properly
}

pub struct ReplySupport {
    pub host: String,
    pub nick: String,
    pub channel_len: u32 // TODO this is wrong this message needs to be much more flexible
}

pub struct ReplyLUserClient {
    pub host: String,
    pub nick: String,
    pub users: u32,
    pub invisible_users: u32,
    pub servers: u32
}

pub struct ReplyLUserOp {
    pub host: String,
    pub nick: String,
    pub operators: u32
}

pub struct ReplyLUserUnknown {
    pub host: String,
    pub nick: String,
    pub unknown: u32
}

impl ReplyWelcome {
    pub fn new(host: String, welcome_message: String, nick: String) -> Box<dyn Display> {
        Box::new(ReplyWelcome { host, welcome_message, nick })
    }
}

impl ReplyYourHost {
    pub fn new(host: String, version: String, nick: String) -> Box<dyn Display> {
        Box::new(ReplyYourHost { host, version, nick })
    }
}

impl ReplyCreated {
    pub fn new(host: String, nick: String, created_message: String, created_at: DateTime<Utc>) -> Box<dyn Display> {
        Box::new(ReplyCreated { host, nick, created_message, created_at })
    }
}

impl ReplyMyInfo {
    pub fn new(host: String, nick: String, version: String, available_user_modes: String, available_channel_modes: String) -> Box<dyn Display> {
        Box::new(ReplyMyInfo { host, nick, version, available_user_modes, available_channel_modes })
    }
}

impl ReplySupport {
    pub fn new(host: String, nick: String, channel_len: u32) -> Box<dyn Display> {
        Box::new(ReplySupport { host, nick, channel_len })
    }
}

impl ReplyLUserClient {
    pub fn new(host: String, nick: String, users: u32, invisible_users: u32, servers: u32) -> Box<dyn Display> {
        Box::new(ReplyLUserClient { host, nick, users, invisible_users, servers })
    }
}

impl ReplyLUserOp {
    pub fn new(host: String, nick: String, operators: u32) -> Box<dyn Display> {
        Box::new(ReplyLUserOp { host, nick, operators })
    }
}

impl ReplyLUserUnknown {
    pub fn new(host: String, nick: String, unknown: u32) -> Box<dyn Display> {
        Box::new(ReplyLUserUnknown { host, nick, unknown })
    }
}

#[derive(Debug)]
pub struct RplISupport {
    pub channel_len: u32 // TODO this is wrong this message needs to be much more flexible
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
impl FromStr for ClientToServerMessage {
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
            command
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

#[test]
fn client_to_server_has_prefix_is_parsed() {
    let expected_nick = format!("Joe");
    let expected_message = ClientToServerMessage {
        source: Some(format!("FOO")),
        command: ClientToServerCommand::Nick(NickCommand {
            nick: expected_nick.clone()
        })
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
        })
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
    let reply = ReplyWelcome::new("localhost".to_string(), "HELLO WORLD".to_string(), "JIM".to_string());
    let actual = reply.to_string();
    let expected = ":localhost 001 JIM :HELLO WORLD JIM";
    assert_eq!(expected, actual);
}

#[test]
fn rpl_yourhost_prints_correctly() {
    let reply = ReplyYourHost::new("localhost".to_string(), "0.0.1".to_string(), "JIM".to_string());
    let actual = reply.to_string();
    let expected = ":localhost 002 JIM :Your host is localhost, running version 0.0.1";
    assert_eq!(expected, actual);
}

#[test]
fn rpl_created_prints_correctly() {
    let now = Utc::now();
    let reply = ReplyCreated::new("localhost".to_string(), "JIM".to_string(), "This server was created".to_string(), now.clone());
    let actual = reply.to_string();
    let expected = format!(":localhost 003 JIM :This server was created {}", now);
    assert_eq!(expected, actual);
}

#[test]
fn rpl_myinfo_prints_correctly() {
    let reply = ReplyMyInfo::new("localhost".to_string(), "JIM".to_string(), "0.0.1".to_string(), "r".to_string(), "i".to_string());
    let actual = reply.to_string();
    let expected = format!(":localhost 004 JIM localhost 0.0.1 r i");
    assert_eq!(expected, actual);
}

#[test]
fn rpl_isupport_prints_correctly() {
    let reply = ReplySupport::new("localhost".to_string(), "JIM".to_string(), 100);
    let actual = reply.to_string();
    let expected = format!(":localhost 005 JIM CHANNELLEN=100 :are supported by this server");
    assert_eq!(expected, actual);
}

#[test]
fn rpl_luserclient_prints_correctly() {
    let reply = ReplyLUserClient::new("localhost".to_string(), "JIM".to_string(), 100, 20, 1);
    let actual = reply.to_string();
    let expected = format!(":localhost 251 JIM :There are 100 users and 20 invisible on 1 servers");
    assert_eq!(expected, actual);
}

#[test]
fn rpl_luserop_prints_correctly() {
    let reply = ReplyLUserOp::new("localhost".to_string(), "JIM".to_string(), 1337);
    let actual = reply.to_string();
    let expected = format!(":localhost 252 JIM 1337 :IRC Operators online");
    assert_eq!(expected, actual);
}

#[test]
fn rpl_luserunknown_prints_correctly() {
    let reply = ReplyLUserUnknown::new("localhost".to_string(), "JIM".to_string(), 7);
    let actual = reply.to_string();
    let expected = format!(":localhost 253 JIM 7 :unknown connection(s)");
    assert_eq!(expected, actual);
}
