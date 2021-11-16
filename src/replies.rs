use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::{fmt::Display, net::SocketAddr};

pub enum Reply {
    Welcome {
        server_host: String,
        nick: String,
    },
    YourHost {
        server_host: String,
        nick: String,
        version: String,
    },
    Created {
        server_host: String,
        nick: String,
        created_at: DateTime<Utc>,
    },
    MyInfo {
        server_host: String,
        nick: String,
        version: String,
        user_modes: String,
        channel_modes: String,
    },
    Support {
        server_host: String,
        nick: String,
        channel_len: u32,
    },
    StatsDLine {
        server_host: String,
        nick: String,
        connections: u32,
        clients: u32,
        received: u32,
    },
    LuserClient {
        server_host: String,
        nick: String,
        visible_users: u32,
        invisible_users: u32,
        servers: u32,
    },
    LuserOp {
        server_host: String,
        nick: String,
        operators: u32,
    },
    LuserUnknown {
        server_host: String,
        nick: String,
        unknown: u32,
    },
    LuserChannels {
        server_host: String,
        nick: String,
        channels: u32,
    },
    LuserMe {
        server_host: String,
        nick: String,
        clients: u32,
        servers: u32,
    },
    LocalUsers {
        server_host: String,
        nick: String,
        current: u32,
        max: u32,
    },
    GlobalUsers {
        server_host: String,
        nick: String,
        current: u32,
        max: u32,
    },
    EndOfWho {
        server_host: String,
        nick: String,
        mask: String,
    },
    ListEnd {
        server_host: String,
    },
    // TODO mode should not be plain strings
    ChannelModeIs {
        server_host: String,
        nick: String,
        channel: String,
        mode_string: String,
        mode_arguments: String,
    },
    CreationTime {
        server_host: String,
        nick: String,
        channel: String,
        created_at: DateTime<Utc>,
    },
    Topic {
        server_host: String,
        nick: String,
        channel: String,
        topic: String,
    },
    TopicWhoTime {
        server_host: String,
        channel: String,
        nick: String,
        set_at: DateTime<Utc>,
    },
    Who {
        server_host: String,
        nick: String,
        channel: String,
        other_user: String,
        other_host: String,
        other_server: String,
        other_nick: String,
        other_realname: String,
    },
    Nam {
        server_host: String,
        nick: String,
        channel: String,
        channel_users: Vec<String>,
    },
    EndOfNames {
        server_host: String,
        nick: String,
        channel: String,
    },
    Motd {
        server_host: String,
        nick: String,
        line: String,
    },
    MotdStart {
        server_host: String,
        nick: String,
    },
    EndOfMotd {
        server_host: String,
        nick: String,
    },
    // TODO should these non-numerics be in a different file??
    Ping {
        server_host: String,
    },
    Pong {
        server_host: String,
        token: String,
    },
    Join {
        client: String,
        channel: String,
    },
    PrivMsg {
        client_host: Option<SocketAddr>,
        nick: Option<String>,
        user: Option<String>,
        channel: String,
        message: String,
    },
    Quit {
        connection_id: Uuid,
        client_host: Option<SocketAddr>,
        nick: Option<String>,
        user: Option<String>,
        message: String,
    },
    ErrNeedMoreParams {
        server_host: String,
        nick: String,
        command: String,
    },
    ErrNoNickGiven {
        server_host: String,
    },
}

impl Display for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reply::Welcome { server_host, nick } => {
                write!(
                    f,
                    ":{} 001 {} :Welcome to the server {}",
                    server_host, nick, nick
                )
            }
            // TODO should this show client host too?
            Reply::YourHost {
                server_host,
                nick,
                version,
            } => write!(
                f,
                ":{} 002 {} :Your host is {}, running version {}",
                server_host, nick, server_host, version
            ),
            Reply::Created {
                server_host,
                nick,
                created_at,
            } => write!(
                f,
                ":{} 003 {} :This server was created {}",
                server_host, nick, created_at
            ),
            Reply::MyInfo {
                server_host,
                nick,
                version,
                user_modes,
                channel_modes,
            } => {
                write!(
                    f,
                    ":{} 004 {} {} {} {} {}",
                    server_host, nick, server_host, version, user_modes, channel_modes
                )
            }
            Reply::Support {
                server_host,
                nick,
                channel_len,
            } => write!(
                f,
                ":{} 005 {} CHANNELLEN={} :are supported by this server",
                server_host, nick, channel_len
            ),
            Reply::StatsDLine {
                server_host,
                nick,
                connections,
                clients,
                received,
            } => {
                write!(f, ":{} 250 {} :Highest connection count: {} ({} clients) ({} connections received)", server_host, nick, connections, clients, received)
            }
            Reply::LuserClient {
                server_host,
                nick,
                visible_users,
                invisible_users,
                servers,
            } => {
                write!(
                    f,
                    ":{} 251 {} :There are {} users and {} invisible on {} servers",
                    server_host, nick, visible_users, invisible_users, servers
                )
            }
            Reply::LuserOp {
                server_host,
                nick,
                operators,
            } => write!(
                f,
                ":{} 252 {} {} :IRC Operators online",
                server_host, nick, operators
            ),
            Reply::LuserUnknown {
                server_host,
                nick,
                unknown,
            } => write!(
                f,
                ":{} 253 {} {} :unknown connection(s)",
                server_host, nick, unknown
            ),
            Reply::LuserChannels {
                server_host,
                nick,
                channels,
            } => write!(
                f,
                ":{} 254 {} {} :channels formed",
                server_host, nick, channels
            ),
            Reply::LuserMe {
                server_host,
                nick,
                clients,
                servers,
            } => write!(
                f,
                ":{} 255 {} :I have {} clients and {} servers",
                server_host, nick, clients, servers
            ),
            Reply::LocalUsers {
                server_host,
                nick,
                current,
                max,
            } => {
                write!(
                    f,
                    ":{} 265 {} {} {} :Current local users {}, max {}",
                    server_host, nick, current, max, current, max
                )
            }
            Reply::GlobalUsers {
                server_host,
                nick,
                current,
                max,
            } => {
                write!(
                    f,
                    ":{} 266 {} {} {} :Current global users {}, max {}",
                    server_host, nick, current, max, current, max
                )
            }
            Reply::EndOfWho {
                server_host,
                nick,
                mask,
            } => write!(
                f,
                ":{} 315 {} {} :End of /WHO list.",
                server_host, nick, mask
            ),
            Reply::ListEnd { server_host } => write!(f, ":{} 323 :End of /LIST", server_host),
            // this may be duplicate of Mode?
            Reply::ChannelModeIs {
                server_host,
                nick,
                channel,
                mode_string,
                mode_arguments,
            } => {
                write!(
                    f,
                    ":{} 324 {} {} {} {}",
                    server_host, nick, channel, mode_string, mode_arguments
                )
            }
            Reply::CreationTime {
                server_host,
                nick,
                channel,
                created_at,
            } => write!(
                f,
                ":{} 329 {} {} {}",
                server_host, nick, channel, created_at
            ),
            Reply::Topic {
                server_host,
                nick,
                channel,
                topic,
            } => write!(f, ":{} 332 {} {} :{}", server_host, nick, channel, topic),
            // TODO print set_at as UNIX time??
            Reply::TopicWhoTime {
                server_host,
                channel,
                nick,
                set_at,
            } => write!(f, ":{} 333 {} {} {}", server_host, nick, channel, set_at),
            // TODO remove hard-coding
            Reply::Who {
                server_host,
                nick,
                channel,
                other_user,
                other_host,
                other_server,
                other_nick,
                other_realname,
            } => {
                write!(
                    f,
                    ":{} 352 {} {} {} {} {} {} H@ :0 {}",
                    server_host,
                    nick,
                    channel,
                    other_user,
                    other_host,
                    other_server,
                    other_nick,
                    other_realname
                )
            }
            Reply::Nam {
                server_host,
                nick,
                channel,
                channel_users,
            } => {
                let mut printed_users = String::new();
                let mut first = true;

                for user in channel_users {
                    if first {
                        first = false;
                        printed_users.push_str(user);
                    } else {
                        printed_users.push_str(&format!(" {}", user));
                    }
                }

                write!(
                    f,
                    ":{} 353 {} = {} {}",
                    server_host, nick, channel, printed_users
                )
            }
            Reply::EndOfNames {
                server_host,
                nick,
                channel,
            } => write!(
                f,
                ":{} 366 {} {} :End of /NAMES list.",
                server_host, nick, channel
            ),
            Reply::Motd {
                server_host,
                nick,
                line,
            } => write!(f, ":{} 372 {} :- {}", server_host, nick, line),
            Reply::MotdStart { server_host, nick } => {
                write!(
                    f,
                    ":{} 375 {} :- {} Message of the Day -",
                    server_host, nick, server_host
                )
            }
            Reply::EndOfMotd { server_host, nick } => {
                write!(f, ":{} 376 {} :End of /MOTD command.", server_host, nick)
            }
            Reply::Ping { server_host } => write!(f, ":{} PING", server_host),
            Reply::Pong { server_host, token } => {
                write!(f, ":{} PONG {} :{}", server_host, server_host, token)
            }
            // this is sent to all users on the channel maybe should not be in this file?
            Reply::Join { client, channel } => write!(f, ":{} JOIN :{}", client, channel),
            Reply::PrivMsg {
                client_host,
                nick,
                user,
                channel,
                message,
            } => {
                // TODO this isnt strictly quite right
                let mut prefix = format!("");
                if let Some(n) = nick {
                    prefix.push_str(&format!(":{}", &n.to_string()));

                    if let Some(u) = user {
                        prefix.push_str(&format!("!{}", u))
                    }

                    if let Some(ch) = client_host {
                        prefix.push_str(&format!("@{}", &ch.to_string()))
                    }
                }

                write!(f, "{} PRIVMSG {} :{}", prefix, channel, message)
            }
            Reply::Quit { connection_id, nick, user, client_host, message } => {
                // TODO this isnt strictly quite right
                let mut prefix = format!("");
                if let Some(n) = nick {
                    prefix.push_str(&format!(":{}", &n.to_string()));

                    if let Some(u) = user {
                        prefix.push_str(&format!("!{}", u))
                    }

                    if let Some(ch) = client_host {
                        prefix.push_str(&format!("@{}", &ch.to_string()))
                    }
                }

                write!(f, "{} QUIT :{}", prefix, message)
            }
            Reply::ErrNeedMoreParams {
                server_host,
                nick,
                command,
            } => {
                write!(
                    f,
                    "{} 461 {} {} :Not enough parameters",
                    server_host, nick, command
                )
            }
            Reply::ErrNoNickGiven { server_host } => {
                write!(f, "{} 431 :No nickname given", server_host)
            }
        }
    }
}

#[test]
fn welcome_prints_correctly() {
    let reply = Reply::Welcome {
        server_host: "localhost".to_string(),
        nick: "JIM".to_string(),
    };
    let actual = reply.to_string();
    let expected = ":localhost 001 JIM :Welcome to the server JIM";
    assert_eq!(expected, actual);
}

#[test]
fn yourhost_prints_correctly() {
    let reply = Reply::YourHost {
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
        nick: "JIM".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 375 JIM :- localhost Message of the Day -");
    assert_eq!(expected, actual);
}

#[test]
fn endofmotd_prints_correctly() {
    let reply = Reply::EndOfMotd {
        server_host: "localhost".to_string(),
        nick: "JIM".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 376 JIM :End of /MOTD command.");
    assert_eq!(expected, actual);
}

#[test]
fn motd_prints_correctly() {
    let reply = Reply::Motd {
        server_host: "localhost".to_string(),
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
        server_host: "localhost".to_string(),
        token: "LAG1238948394".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost PONG localhost :LAG1238948394");
    assert_eq!(expected, actual);
}

#[test]
fn listend_prints_correctly() {
    let reply = Reply::ListEnd {
        server_host: "localhost".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 323 :End of /LIST");
    assert_eq!(expected, actual);
}

#[test]
fn endofnames_prints_correctly() {
    let reply = Reply::EndOfNames {
        server_host: "localhost".to_string(),
        nick: "JIM".to_string(),
        channel: "#foobar".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 366 JIM #foobar :End of /NAMES list.");
    assert_eq!(expected, actual);
}

#[test]
fn endofwho_prints_correctly() {
    let reply = Reply::EndOfWho {
        server_host: "localhost".to_string(),
        nick: "JIM".to_string(),
        mask: "#foobar".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 315 JIM #foobar :End of /WHO list.");
    assert_eq!(expected, actual);
}

#[test]
fn topic_prints_correctly() {
    let reply = Reply::Topic {
        server_host: "localhost".to_string(),
        nick: "JIM".to_string(),
        channel: "#foobar".to_string(),
        topic: "hELLO WORLD".to_string(),
    };
    let actual = reply.to_string();
    let expected = format!(":localhost 332 JIM #foobar :hELLO WORLD");
    assert_eq!(expected, actual);
}
