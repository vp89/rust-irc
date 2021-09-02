pub mod message_parsing;
pub mod replies;

use std::{collections::VecDeque, io::{self, BufRead, ErrorKind, Read, Write}, net::{Shutdown, TcpListener, TcpStream}, str::{FromStr}, thread, time::{Duration, Instant}};
use chrono::{DateTime, Utc};
use crate::message_parsing::*;
use crate::replies::*;

fn main() -> io::Result<()> {
    let host = "localhost".to_string();
    let context = ServerContext {
        start_time: Utc::now(),
        host: host.clone(),
        version: "0.0.1".to_string(),
        ping_frequency: Duration::from_secs(10)
    };

    println!("STARTING SERVER ON {}:6667", host);

    let listener = TcpListener::bind(format!("{}:6667", host))?;
    let mut connection_handles = vec![];

    for connection_attempt in listener.incoming() {
        let server_context = context.clone();

        match connection_attempt {
            Ok(stream) => {
                connection_handles.push(
                    thread::spawn(move || {
                        if let Err(e) = stream.set_read_timeout(Some(server_context.ping_frequency)) {
                            println!("ERROR SETTING READ TIMEOUT {:?}", e);
                            return;
                        }

                        if let Err(e) = handle_connection(&stream, server_context) {
                            println!("ERROR FROM CONNECTION HANDLER {:?}", e)
                        }
                        
                        if let Err(e) = stream.shutdown(Shutdown::Both) {
                            println!("ERROR SHUTTING DOWN SOCKET {:?}", e)
                        }

                        println!("FINISHED HANDLING THIS CONNECTION")
                    }));
            },
            Err(e) => println!("ERROR CONNECTING {:?}", e),
        };
    }

    for handle in connection_handles {
        if let Err(e) = handle.join() {
            println!("Error joining thread handle {:?}", e);
        }
    }

    Ok(())
}

fn get_messages<T: BufRead>(reader: &mut T) -> io::Result<Vec<String>> {
    // TODO can this be cleaned up?
    let bytes = match reader.fill_buf() {
        Ok(s) => {
            Ok(s)
        }
        Err(e) => match e.kind() {
            // This particular ErrorKind is returned on Unix platforms
            // if the TcpStream timed out per the read_timeout setting
            // would need to test that on Windows if that became a goal to
            // support both of those.
            ErrorKind::WouldBlock => {
                return Ok(vec![])
            },
            _ => Err(e)
        }
    }?;

    let bytes_read = bytes.len();

    match std::str::from_utf8(bytes) {
        Ok(s) => {
            let raw_payload = s.to_owned();

            println!("RECEIVED {}", raw_payload);

            // map to owned String so the ownership can be moved out of this function scope
            let mut split_messages: Vec<String> = raw_payload.split("\r\n").map(|s| s.to_string()).collect();

            // TODO create own Error kinds/type??
            if split_messages.len() <= 1 {
                Err(io::Error::new(io::ErrorKind::InvalidData, "No message separator provided"))
            } else if split_messages.last().unwrap_or(&"BLAH".to_string()) != &format!("") {
                Err(io::Error::new(io::ErrorKind::InvalidData, "Last message did not have expected separator"))
            } else {
                split_messages.truncate(split_messages.len() - 1);
                reader.consume(bytes_read);
                Ok(split_messages)
            }
        },
        Err(e) => {
            Err(io::Error::new(io::ErrorKind::InvalidData, e))
        }
    }
}

#[test]
fn get_messages_reads_from_buffer() {
    let fake_buffer = b"Hello world\r\n".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(13);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses
    };

    let result = get_messages(&mut faked_bufreader).unwrap();
    assert_eq!(1, result.len());
    assert_eq!("Hello world", result.first().unwrap());
    assert_eq!(0, faked_bufreader.fake_buffer.len());
}

#[test]
fn get_messages_multiplemessages_reads_from_buffer() {
    let fake_buffer = b"Hello world\r\nFoobar\r\n".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(21);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses
    };

    let result = get_messages(&mut faked_bufreader).unwrap();
    assert_eq!(2, result.len());
    assert_eq!("Hello world", result.first().unwrap());
    assert_eq!("Foobar", result[1]);
    assert_eq!(0, faked_bufreader.fake_buffer.len());
}

#[test]
fn get_messages_multiplemessages_noterminator_errors() {
    let fake_buffer = b"Hello world\r\nFoobar".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(19);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses
    };

    let result = get_messages(&mut faked_bufreader).expect_err("Testing expect an error to be returned here");
    assert_eq!("Last message did not have expected separator", result.to_string());
}

#[test]
fn get_messages_nolineterminator_errors() {
    let fake_buffer = b"Hello world".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(11);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses
    };

    let result = get_messages(&mut faked_bufreader).expect_err("Testing expect an error to be returned here");
    assert_eq!("No message separator provided", result.to_string());
}

#[test]
fn get_messages_emptybuffer_errors() {
    let fake_buffer = b"".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(0);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses
    };

    get_messages(&mut faked_bufreader).expect_err("Testing expect an error to be returned here");
}

struct FakeBufReader {
    fake_buffer: Vec<u8>,
    faked_responses: VecDeque<usize> // implement as a queue so you can mock which bytes returned on each read call
}

impl BufRead for FakeBufReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self.faked_responses.pop_front() {
            Some(b) => {
                Ok(&self.fake_buffer[..b])
            },
            None => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "bad test setup"))
        }
    }

    fn consume(&mut self, amt: usize) {
        self.fake_buffer.drain(..amt);
    }
}

// just adding this to make compiler happy, for testing we only need
// the methods in BufRead
impl Read for FakeBufReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }
}

fn handle_connection(stream: &TcpStream, context: ServerContext) -> io::Result<()> {
    // connection handler just runs a loop that reads bytes off the stream
    // and sends responses based on logic or until the connection has died
    // there also needs to be a ping loop going on that can stop this loop too

    let mut write_handle = stream.try_clone()?;
    let mut reader = io::BufReader::with_capacity(512, stream);
    let mut last_pong = Instant::now();
    let mut waiting_for_pong = false;

    loop {
        if waiting_for_pong && last_pong.elapsed().as_secs() > context.ping_frequency.as_secs() + 5 {
            println!("NO PONG RECEIVED CLOSING DOWN HANDLER");
            return Ok(());
        }

        if last_pong.elapsed().as_secs() > context.ping_frequency.as_secs() {
            waiting_for_pong = true;
            println!("SENDING PING");
            let ping = format!("{}\r\n", Reply::Ping { host: &context.host }.to_string());
            write_handle.write_all(ping.as_bytes())?;
            write_handle.flush()?;
        }

        let raw_messages = get_messages(&mut reader)?;
        let now = Utc::now();

        for raw_message in &raw_messages {
            let message = ClientToServerMessage::from_str(raw_message).expect("FOO"); // TODO

            let host = &context.host;
            let version = &context.version;
            let created_at = &context.start_time;

            let replies = match &message.command {
                ClientToServerCommand::Quit => {
                    return Ok(());
                },
                ClientToServerCommand::Unhandled => {
                    println!("MESSAGE UNHANDLED {:?} {}", message, raw_message);
                    None
                },
                ClientToServerCommand::Ping { token } => {
                    Some(vec![ Reply::Pong { host, token: token.clone() } ])
                },
                ClientToServerCommand::Pong => {
                    last_pong = Instant::now();
                    waiting_for_pong = false;
                    None
                },
                ClientToServerCommand::Join { channels} => {
                    // add channel join handling
                    let mut replies: Vec::<Reply> = Vec::new();

                    for channel in channels {
                        let mut channel_replies = vec![
                            Reply::Join { client: "vince!~vince@localhost", channel },
                            // TODO have Nick available here
                            // TODO persist the channel metadata
                            Reply::Topic { host, nick: "vince", channel, topic: "foobar topic" },
                            Reply::TopicWhoTime { host, channel, nick: "vince", set_at: &now },
                            Reply::NamReply { host, channel, nick: "vince" },
                        ];

                        replies.append(&mut channel_replies);
                    }
                    
                    Some(replies)
                },
                ClientToServerCommand::Mode { channel } => {
                    Some(vec![ 
                        Reply::Mode { host, channel, mode_string: "+tn" },
                        Reply::ChannelModeIs { host, nick: "vince", channel, mode_string: "+mtn1", mode_arguments: "100" },
                        Reply::CreationTime { host, nick: "vince", channel, created_at: &now } 
                    ])
                },
                ClientToServerCommand::Who { channel } => {
                    Some(vec![
                        Reply::WhoReply { host, channel, nick: "vince", other_nick: "~vince", client: "localhost" },
                        Reply::EndOfWho { host, nick: "vince", channel },
                    ])
                },
                ClientToServerCommand::Nick { nick } => {                    
                    let mut welcome_storm = vec![
                        Reply::Welcome { host, nick },
                        Reply::YourHost { host, nick, version },
                        Reply::Created { host, nick, created_at },
                        Reply::MyInfo { host, nick, version, user_modes: "r", channel_modes: "i" },
                        Reply::Support { host, nick, channel_len: 32 },
                        Reply::LuserClient { host, nick, visible_users: 100, invisible_users: 20, servers: 1 },
                        Reply::LuserOp { host, nick, operators: 1337 },
                        Reply::LuserUnknown { host, nick, unknown: 7 },
                        Reply::LuserChannels { host, nick, channels: 9999 },
                        Reply::LuserMe { host, nick, clients: 900, servers: 1 },
                        Reply::LocalUsers { host, nick, current: 845, max: 1000 },
                        Reply::GlobalUsers { host, nick, current: 9832, max: 23455 },
                        Reply::StatsDLine { host, nick, connections: 9998, clients: 9000, received: 99999 }
                    ];

                    // TODO proper configurable MOTD
                    let mut motd_replies = vec![ Reply::Motd { host, nick, line: "Foobar" }];
                    welcome_storm.push(Reply::MotdStart { host, nick });
                    welcome_storm.append(&mut motd_replies);
                    welcome_storm.push(Reply::EndOfMotd { host, nick });
                    
                    Some(welcome_storm)
                }
            };

            match replies {
                None => {},
                Some(messages) => {
                    let mut reply = String::new();

                    for message in messages {
                        reply.push_str(&message.to_string());
                        reply.push_str("\r\n");                        
                    }

                    println!("SENDING {}", reply);
                    write_handle.write_all(reply.as_bytes())?;
                    write_handle.flush()?;
                }
            }
        }
    }
}

#[derive(Clone)]
struct ServerContext {
    pub start_time: DateTime<Utc>,
    pub host: String,
    pub version: String,
    pub ping_frequency: Duration
}
