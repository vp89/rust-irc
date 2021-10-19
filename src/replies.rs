use chrono::{DateTime, Utc};
use std::fmt::Display;

pub enum Reply {
    Welcome {
        host: String,
        nick: String,
    },
    YourHost {
        host: String,
        nick: String,
        version: String,
    },
    Created {
        host: String,
        nick: String,
        created_at: DateTime<Utc>,
    },
    MyInfo {
        host: String,
        nick: String,
        version: String,
        user_modes: String,
        channel_modes: String,
    },
    Support {
        host: String,
        nick: String,
        channel_len: u32,
    },
    StatsDLine {
        host: String,
        nick: String,
        connections: u32,
        clients: u32,
        received: u32,
    },
    LuserClient {
        host: String,
        nick: String,
        visible_users: u32,
        invisible_users: u32,
        servers: u32,
    },
    LuserOp {
        host: String,
        nick: String,
        operators: u32,
    },
    LuserUnknown {
        host: String,
        nick: String,
        unknown: u32,
    },
    LuserChannels {
        host: String,
        nick: String,
        channels: u32,
    },
    LuserMe {
        host: String,
        nick: String,
        clients: u32,
        servers: u32,
    },
    LocalUsers {
        host: String,
        nick: String,
        current: u32,
        max: u32,
    },
    GlobalUsers {
        host: String,
        nick: String,
        current: u32,
        max: u32,
    },
    EndOfWho {
        host: String,
        nick: String,
        channel: String,
    },
    ListEnd {
        host: String,
    },
    // TODO mode should not be plain strings
    ChannelModeIs {
        host: String,
        nick: String,
        channel: String,
        mode_string: String,
        mode_arguments: String,
    },
    CreationTime {
        host: String,
        nick: String,
        channel: String,
        created_at: DateTime<Utc>,
    },
    Topic {
        host: String,
        nick: String,
        channel: String,
        topic: String,
    },
    TopicWhoTime {
        host: String,
        channel: String,
        nick: String,
        set_at: DateTime<Utc>,
    },
    Who {
        host: String,
        nick: String,
        channel: String,
        client: String,
        other_nick: String,
    },
    Nam {
        host: String,
        channel: String,
        nick: String,
    },
    EndOfNames {
        host: String,
        nick: String,
        channel: String,
    },
    Motd {
        host: String,
        nick: String,
        line: String,
    },
    MotdStart {
        host: String,
        nick: String,
    },
    EndOfMotd {
        host: String,
        nick: String,
    },
    // TODO should these non-numerics be in a different file??
    Ping {
        host: String,
    },
    Pong {
        host: String,
        token: String,
    },
    Join {
        client: String,
        channel: String,
    },
    Mode {
        host: String,
        channel: String,
        mode_string: String,
    },
    PrivMsg {
        client: String,
        channel: String,
        message: String,
    },
}

impl Display for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reply::Welcome { host, nick } => {
                write!(f, ":{} 001 {} :Welcome to the server {}", host, nick, nick)
            }
            Reply::YourHost {
                host,
                nick,
                version,
            } => write!(
                f,
                ":{} 002 {} :Your host is {}, running version {}",
                host, nick, host, version
            ),
            Reply::Created {
                host,
                nick,
                created_at,
            } => write!(
                f,
                ":{} 003 {} :This server was created {}",
                host, nick, created_at
            ),
            Reply::MyInfo {
                host,
                nick,
                version,
                user_modes,
                channel_modes,
            } => {
                write!(
                    f,
                    ":{} 004 {} {} {} {} {}",
                    host, nick, host, version, user_modes, channel_modes
                )
            }
            Reply::Support {
                host,
                nick,
                channel_len,
            } => write!(
                f,
                ":{} 005 {} CHANNELLEN={} :are supported by this server",
                host, nick, channel_len
            ),
            Reply::StatsDLine {
                host,
                nick,
                connections,
                clients,
                received,
            } => {
                write!(f, ":{} 250 {} :Highest connection count: {} ({} clients) ({} connections received)", host, nick, connections, clients, received)
            }
            Reply::LuserClient {
                host,
                nick,
                visible_users,
                invisible_users,
                servers,
            } => {
                write!(
                    f,
                    ":{} 251 {} :There are {} users and {} invisible on {} servers",
                    host, nick, visible_users, invisible_users, servers
                )
            }
            Reply::LuserOp {
                host,
                nick,
                operators,
            } => write!(
                f,
                ":{} 252 {} {} :IRC Operators online",
                host, nick, operators
            ),
            Reply::LuserUnknown {
                host,
                nick,
                unknown,
            } => write!(
                f,
                ":{} 253 {} {} :unknown connection(s)",
                host, nick, unknown
            ),
            Reply::LuserChannels {
                host,
                nick,
                channels,
            } => write!(f, ":{} 254 {} {} :channels formed", host, nick, channels),
            Reply::LuserMe {
                host,
                nick,
                clients,
                servers,
            } => write!(
                f,
                ":{} 255 {} :I have {} clients and {} servers",
                host, nick, clients, servers
            ),
            Reply::LocalUsers {
                host,
                nick,
                current,
                max,
            } => {
                write!(
                    f,
                    ":{} 265 {} {} {} :Current local users {}, max {}",
                    host, nick, current, max, current, max
                )
            }
            Reply::GlobalUsers {
                host,
                nick,
                current,
                max,
            } => {
                write!(
                    f,
                    ":{} 266 {} {} {} :Current global users {}, max {}",
                    host, nick, current, max, current, max
                )
            }
            Reply::EndOfWho {
                host,
                nick,
                channel,
            } => write!(f, ":{} 315 {} {} :End of /WHO list", host, nick, channel),
            Reply::ListEnd { host } => write!(f, ":{} 323 :End of /LIST", host),
            // this may be duplicate of Mode?
            Reply::ChannelModeIs {
                host,
                nick,
                channel,
                mode_string,
                mode_arguments,
            } => {
                write!(
                    f,
                    ":{} 324 {} {} {} {}",
                    host, nick, channel, mode_string, mode_arguments
                )
            }
            Reply::CreationTime {
                host,
                nick,
                channel,
                created_at,
            } => write!(f, ":{} 329 {} {} {}", host, nick, channel, created_at),
            Reply::Topic {
                host,
                nick,
                channel,
                topic,
            } => write!(f, ":{} 332 {} {} :{}", host, nick, channel, topic),
            // TODO print set_at as UNIX time??
            Reply::TopicWhoTime {
                host,
                channel,
                nick,
                set_at,
            } => write!(f, ":{} 333 {} {} {}", host, nick, channel, set_at),
            // TODO remove hard-coding
            Reply::Who {
                host,
                nick,
                channel,
                other_nick,
                client,
            } => {
                write!(
                    f,
                    ":{} 352 {} {} {} {} {} {} H@ :0 realname",
                    host, nick, channel, other_nick, client, host, nick
                )
            }
            //RES -> :<source> 353 nick = #channel :listofusers with @
            Reply::Nam {
                host,
                channel,
                nick,
            } => write!(f, ":{} 353 {} = {} :@{}", host, nick, channel, nick),
            Reply::EndOfNames {
                host,
                nick,
                channel,
            } => write!(f, ":{} 366 {} {} :End of /NAMES list", host, nick, channel),
            Reply::Motd { host, nick, line } => write!(f, ":{} 372 {} :- {}", host, nick, line),
            Reply::MotdStart { host, nick } => {
                write!(f, ":{} 375 {} :- {} Message of the Day -", host, nick, host)
            }
            Reply::EndOfMotd { host, nick } => {
                write!(f, ":{} 376 {} :End of /MOTD command.", host, nick)
            }
            Reply::Ping { host } => write!(f, ":{} PING", host),
            Reply::Pong { host, token } => write!(f, ":{} PONG {} :{}", host, host, token),
            // this is sent to all users on the channel maybe should not be in this file?
            Reply::Join { client, channel } => write!(f, ":{} JOIN :{}", client, channel),
            // this one is not numeric not sure where to put it..
            Reply::Mode {
                host,
                channel,
                mode_string,
            } => write!(f, ":{} MODE {} {}", host, channel, mode_string),
            Reply::PrivMsg {
                client,
                channel,
                message,
            } => write!(f, ":{} PRIVMSG {} :{}", client, channel, message),
        }
    }
}

#[test]
fn welcome_prints_correctly() {
    let reply = Reply::Welcome {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
    };
    let actual = reply.to_string();
    let expected = ":localhost 001 JIM :Welcome to the server JIM";
    assert_eq!(expected, actual);
}

#[test]
fn yourhost_prints_correctly() {
    let reply = Reply::YourHost {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        version: "0.0.1".to_string(),
    };
    let actual = reply.to_string();
    let expected = ":localhost 002 JIM :Your host is localhost, running version 0.0.1";
    assert_eq!(expected, actual);
}

#[test]
fn created_prints_correctly() {
    let now = Utc::now();
    let reply = Reply::Created {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        created_at: now.clone(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 003 JIM :This server was created {}", now);
    assert_eq!(expected, actual);
}

#[test]
fn myinfo_prints_correctly() {
    let reply = Reply::MyInfo {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        version: "0.0.1".to_string(),
        user_modes: "r".to_string(),
        channel_modes: "i".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 004 JIM localhost 0.0.1 r i");
    assert_eq!(expected, actual);
}

#[test]
fn support_prints_correctly() {
    let reply = Reply::Support {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        channel_len: 100,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 005 JIM CHANNELLEN=100 :are supported by this server");
    assert_eq!(expected, actual);
}

#[test]
fn luserclient_prints_correctly() {
    let reply = Reply::LuserClient {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        visible_users: 100,
        invisible_users: 20,
        servers: 1,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 251 JIM :There are 100 users and 20 invisible on 1 servers");
    assert_eq!(expected, actual);
}

#[test]
fn luserop_prints_correctly() {
    let reply = Reply::LuserOp {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        operators: 1337,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 252 JIM 1337 :IRC Operators online");
    assert_eq!(expected, actual);
}

#[test]
fn luserunknown_prints_correctly() {
    let reply = Reply::LuserUnknown {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        unknown: 7,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 253 JIM 7 :unknown connection(s)");
    assert_eq!(expected, actual);
}

#[test]
fn luserchannels_prints_correctly() {
    let reply = Reply::LuserChannels {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        channels: 9999,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 254 JIM 9999 :channels formed");
    assert_eq!(expected, actual);
}

#[test]
fn luserme_prints_correctly() {
    let reply = Reply::LuserMe {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        clients: 900,
        servers: 1,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 255 JIM :I have 900 clients and 1 servers");
    assert_eq!(expected, actual);
}

#[test]
fn localusers_prints_correctly() {
    let reply = Reply::LocalUsers {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        current: 845,
        max: 1000,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 265 JIM 845 1000 :Current local users 845, max 1000");
    assert_eq!(expected, actual);
}

#[test]
fn globalusers_prints_correctly() {
    let reply = Reply::GlobalUsers {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        current: 9823,
        max: 23455,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 266 JIM 9823 23455 :Current global users 9823, max 23455");
    assert_eq!(expected, actual);
}

#[test]
fn statsdline_prints_correctly() {
    let reply = Reply::StatsDLine {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        connections: 9998,
        clients: 9000,
        received: 99999,
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 250 JIM :Highest connection count: 9998 (9000 clients) (99999 connections received)");
    assert_eq!(expected, actual);
}

#[test]
fn motdstart_prints_correctly() {
    let reply = Reply::MotdStart {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 375 JIM :- localhost Message of the Day -");
    assert_eq!(expected, actual);
}

#[test]
fn endofmotd_prints_correctly() {
    let reply = Reply::EndOfMotd {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 376 JIM :End of /MOTD command.");
    assert_eq!(expected, actual);
}

#[test]
fn motd_prints_correctly() {
    let reply = Reply::Motd {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        line: "Foobar".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 372 JIM :- Foobar");
    assert_eq!(expected, actual);
}

#[test]
fn pong_prints_correctly() {
    let reply = Reply::Pong {
        host: "localhost".to_string(),
        token: "LAG1238948394".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost PONG localhost :LAG1238948394");
    assert_eq!(expected, actual);
}

#[test]
fn listend_prints_correctly() {
    let reply = Reply::ListEnd {
        host: "localhost".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 323 :End of /LIST");
    assert_eq!(expected, actual);
}

#[test]
fn endofnames_prints_correctly() {
    let reply = Reply::EndOfNames {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        channel: "#foobar".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 366 JIM #foobar :End of /NAMES list");
    assert_eq!(expected, actual);
}

#[test]
fn endofwho_prints_correctly() {
    let reply = Reply::EndOfWho {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        channel: "#foobar".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 315 JIM #foobar :End of /WHO list");
    assert_eq!(expected, actual);
}

#[test]
fn topic_prints_correctly() {
    let reply = Reply::Topic {
        host: "localhost".to_string(),
        nick: "JIM".to_string(),
        channel: "#foobar".to_string(),
        topic: "hELLO WORLD".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 332 JIM #foobar :hELLO WORLD");
    assert_eq!(expected, actual);
}
