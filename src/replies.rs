use std::fmt::Display;
use chrono::{DateTime, Utc};

pub enum Reply<'a> {
    Welcome { host: &'a str, nick: &'a str },
    YourHost { host: &'a str, nick: &'a str, version: &'a str },
    Created { host: &'a str, nick: &'a str, created_at: &'a DateTime<Utc> },
    MyInfo { host: &'a str, nick: &'a str, version: &'a str, user_modes: &'a str, channel_modes: &'a str },
    Support { host: &'a str, nick: &'a str, channel_len: u32 },
    StatsDLine { host: &'a str, nick: &'a str, connections: u32, clients: u32, received: u32 },
    LuserClient { host: &'a str, nick: &'a str, visible_users: u32, invisible_users: u32, servers: u32 },
    LuserOp { host: &'a str, nick: &'a str, operators: u32 },
    LuserUnknown { host: &'a str, nick: &'a str, unknown: u32 },
    LuserChannels { host: &'a str, nick: &'a str, channels: u32 },
    LuserMe { host: &'a str, nick: &'a str, clients: u32, servers: u32 },
    LocalUsers { host: &'a str, nick: &'a str, current: u32, max: u32 },
    GlobalUsers { host: &'a str, nick: &'a str, current: u32, max: u32 },
    Motd { host: &'a str, nick: &'a str, line: &'a str },
    MotdStart { host: &'a str, nick: &'a str },
    EndOfMotd { host: &'a str, nick: &'a str },
    // TODO should these non-numerics be in a different file??
    Pong { host: &'a str, token: String }
}

impl Display for Reply<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reply::Welcome { host, nick } => write!(f, ":{} 001 {} :Welcome to the server {}", host, nick, nick),
            Reply::YourHost { host, nick, version } => write!(f, ":{} 002 {} :Your host is {}, running version {}", host, nick, host, version),
            Reply::Created { host, nick, created_at } => write!(f, ":{} 003 {} :This server was created {}", host, nick, created_at),
            Reply::MyInfo { host, nick, version, user_modes, channel_modes } => {
                write!(f, ":{} 004 {} {} {} {} {}", host, nick, host, version, user_modes, channel_modes)
            },
            Reply::Support { host, nick, channel_len } => write!(f, ":{} 005 {} CHANNELLEN={} :are supported by this server", host, nick, channel_len),
            Reply::StatsDLine { host, nick, connections, clients, received } => {
                write!(f, ":{} 250 {} :Highest connection count: {} ({} clients) ({} connections received)", host, nick, connections, clients, received)
            },
            Reply::LuserClient { host, nick, visible_users, invisible_users, servers } => {
                write!(f, ":{} 251 {} :There are {} users and {} invisible on {} servers", host, nick, visible_users, invisible_users, servers)
            },
            Reply::LuserOp { host, nick, operators } => write!(f, ":{} 252 {} {} :IRC Operators online", host, nick, operators),
            Reply::LuserUnknown { host, nick, unknown } => write!(f, ":{} 253 {} {} :unknown connection(s)", host, nick, unknown),
            Reply::LuserChannels { host, nick, channels } => write!(f, ":{} 254 {} {} :channels formed", host, nick, channels),
            Reply::LuserMe { host, nick, clients, servers } => write!(f, ":{} 255 {} :I have {} clients and {} servers", host, nick, clients, servers),
            Reply::LocalUsers { host, nick, current, max } => {
                write!(f, ":{} 265 {} {} {} :Current local users {}, max {}", host, nick, current, max, current, max)
            },
            Reply::GlobalUsers { host, nick, current, max } => {
                write!(f, ":{} 266 {} {} {} :Current global users {}, max {}", host, nick, current, max, current, max)
            },
            Reply::Motd { host, nick, line } => write!(f, ":{} 372 {} :- {}", host, nick, line),
            Reply::MotdStart { host, nick } => write!(f, ":{} 375 {} :- {} Message of the Day -", host, nick, host),
            Reply::EndOfMotd { host, nick } => write!(f, ":{} 376 {} :End of /MOTD command.", host, nick),
            Reply::Pong { host, token } => write!(f, ":{} PONG {} :{}", host, host, token)
        }
    }
}


#[test]
fn welcome_prints_correctly() {
    let reply = Reply::Welcome { host: "localhost", nick: "JIM" };
    let actual = reply.to_string();
    let expected = ":localhost 001 JIM :Welcome to the server JIM";
    assert_eq!(expected, actual);
}

#[test]
fn yourhost_prints_correctly() {
    let reply = Reply::YourHost { host: "localhost", nick: "JIM", version: "0.0.1" };
    let actual = reply.to_string();
    let expected = ":localhost 002 JIM :Your host is localhost, running version 0.0.1";
    assert_eq!(expected, actual);
}

#[test]
fn created_prints_correctly() {
    let now = Utc::now();
    let reply = Reply::Created { host: "localhost", nick: "JIM", created_at: &now };
    let actual = reply.to_string();
    let expected = format!(":localhost 003 JIM :This server was created {}", now);
    assert_eq!(expected, actual);
}

#[test]
fn myinfo_prints_correctly() {
    let reply = Reply::MyInfo { host: "localhost", nick: "JIM", version: "0.0.1", user_modes: "r", channel_modes: "i" };
    let actual = reply.to_string();
    let expected = format!(":localhost 004 JIM localhost 0.0.1 r i");
    assert_eq!(expected, actual);
}

#[test]
fn support_prints_correctly() {
    let reply = Reply::Support { host: "localhost", nick: "JIM", channel_len: 100 };
    let actual = reply.to_string();
    let expected = format!(":localhost 005 JIM CHANNELLEN=100 :are supported by this server");
    assert_eq!(expected, actual);
}

#[test]
fn luserclient_prints_correctly() {
    let reply = Reply::LuserClient { host: "localhost", nick: "JIM", visible_users: 100, invisible_users: 20, servers: 1 };
    let actual = reply.to_string();
    let expected = format!(":localhost 251 JIM :There are 100 users and 20 invisible on 1 servers");
    assert_eq!(expected, actual);
}

#[test]
fn luserop_prints_correctly() {
    let reply = Reply::LuserOp { host: "localhost", nick: "JIM", operators: 1337 };
    let actual = reply.to_string();
    let expected = format!(":localhost 252 JIM 1337 :IRC Operators online");
    assert_eq!(expected, actual);
}

#[test]
fn luserunknown_prints_correctly() {
    let reply = Reply::LuserUnknown { host: "localhost", nick: "JIM", unknown: 7 };
    let actual = reply.to_string();
    let expected = format!(":localhost 253 JIM 7 :unknown connection(s)");
    assert_eq!(expected, actual);
}

#[test]
fn luserchannels_prints_correctly() {
    let reply = Reply::LuserChannels { host: "localhost", nick: "JIM", channels: 9999 };
    let actual = reply.to_string();
    let expected = format!(":localhost 254 JIM 9999 :channels formed");
    assert_eq!(expected, actual);
}

#[test]
fn luserme_prints_correctly() {
    let reply = Reply::LuserMe { host: "localhost", nick: "JIM", clients: 900, servers: 1 };
    let actual = reply.to_string();
    let expected = format!(":localhost 255 JIM :I have 900 clients and 1 servers");
    assert_eq!(expected, actual);
}

#[test]
fn localusers_prints_correctly() {
    let reply = Reply::LocalUsers { host: "localhost", nick: "JIM", current: 845, max: 1000 };
    let actual = reply.to_string();
    let expected = format!(":localhost 265 JIM 845 1000 :Current local users 845, max 1000");
    assert_eq!(expected, actual);
}

#[test]
fn globalusers_prints_correctly() {
    let reply = Reply::GlobalUsers { host: "localhost", nick: "JIM", current: 9823, max: 23455 };
    let actual = reply.to_string();
    let expected = format!(":localhost 266 JIM 9823 23455 :Current global users 9823, max 23455");
    assert_eq!(expected, actual);
}

#[test]
fn statsdline_prints_correctly() {
    let reply = Reply::StatsDLine { host: "localhost", nick: "JIM", connections: 9998, clients: 9000, received: 99999 };
    let actual = reply.to_string();
    let expected = format!(":localhost 250 JIM :Highest connection count: 9998 (9000 clients) (99999 connections received)");
    assert_eq!(expected, actual);
}

#[test]
fn motdstart_prints_correctly() {
    let reply = Reply::MotdStart { host: "localhost", nick: "JIM" };
    let actual = reply.to_string();
    let expected = format!(":localhost 375 JIM :- localhost Message of the Day -");
    assert_eq!(expected, actual);
}

#[test]
fn endofmotd_prints_correctly() {
    let reply = Reply::EndOfMotd { host: "localhost", nick: "JIM" };
    let actual = reply.to_string();
    let expected = format!(":localhost 376 JIM :End of /MOTD command.");
    assert_eq!(expected, actual);
}

#[test]
fn motd_prints_correctly() {
    let reply = Reply::Motd { host: "localhost", nick: "JIM", line: "Foobar" };
    let actual = reply.to_string();
    let expected = format!(":localhost 372 JIM :- Foobar");
    assert_eq!(expected, actual);
}

#[test]
fn pong_prints_correctly() {
    let reply = Reply::Pong { host: "localhost", token: "LAG1238948394".to_string() };
    let actual = reply.to_string();
    let expected = format!(":localhost PONG localhost :LAG1238948394");
    assert_eq!(expected, actual);
}