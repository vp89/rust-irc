pub mod message_parsing;
pub mod replies;

use std::{cmp::min, io::{self, Read, Write}, net::{TcpListener, TcpStream}, str::{FromStr}, thread};
use chrono::{DateTime, Utc};
use crate::message_parsing::*;
use crate::replies::*;

fn main() -> io::Result<()> {
    let host = format!("localhost");
    let context = ServerContext {
        start_time: Utc::now(),
        host: host.clone(),
        version: format!("0.0.1")
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
                        let conn_outcome = handle_connection(stream, server_context);
                        match conn_outcome {
                            Ok(_) => (),
                            Err(e) => println!("ERROR {}", e)
                        }
                        println!("FINISHED HANDLING THIS CONNECTION")
                    }));
            },
            Err(error) => println!("ERROR CONNECTING {}", error),
        };
    }

    for handle in connection_handles {
        handle.join().expect("TODO")
    }

    Ok(())
}

// TODO figure out how to structure this code
// take a dyn Read so we can mock it for unit tests
fn get_messages(stream: &mut dyn Read) -> io::Result<Vec<String>> {
    let mut buffer = [0;512];
    let bytes_read = stream.read(&mut buffer)?; // TcpStream
    let raw_payload = std::str::from_utf8(&buffer[..bytes_read])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .to_owned();

    println!("RECEIVED {}", raw_payload);

    // map to owned String so the ownership can be moved out of this function scope
    let raw_messages = raw_payload.split_terminator("\r\n").map(|s| s.to_string()).collect();
    Ok(raw_messages)
}

#[test]
fn get_messages_reads_from_buffer() {
    let message = b"Hello world\r\n".to_vec();
    let mut mocked_stream = MockTcpStream {
        bytes_to_read: message
    };

    let result = get_messages(&mut mocked_stream).unwrap();
    assert_eq!(1, result.len());
    assert_eq!("Hello world", result.first().unwrap());
}

struct MockTcpStream {
    bytes_to_read: Vec<u8>
}

impl Read for MockTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size: usize = min(self.bytes_to_read.len(), buf.len());
        buf[..size].copy_from_slice(&self.bytes_to_read[..size]);
        Ok(size)
    }
}

fn handle_connection(mut stream: TcpStream, context: ServerContext) -> io::Result<()> {
    // connection handler just runs a loop that reads bytes off the stream
    // and sends responses based on logic or until the connection has died
    // there also needs to be a ping loop going on that can stop this loop too
    loop {
        let raw_messages = get_messages(&mut stream)?;

        for raw_message in &raw_messages {
            let message = ClientToServerMessage::from_str(raw_message).expect("FOO"); // TODO

            let replies = match &message.command {
                ClientToServerCommand::Quit => {
                    return Ok(()); // is using return not idiomatic?? Look into that
                }
                ClientToServerCommand::Unhandled => {
                    println!("MESSAGE UNHANDLED {:?} {}", message, raw_message);
                    None
                },
                ClientToServerCommand::Nick(c) => {
                    let host = &context.host;
                    let version = &context.version;
                    let nick = &c.nick;
                    let created_at = &context.start_time;

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
                    stream.write(reply.as_bytes())?;
                    stream.flush()?;
                }
            }
        }
    }
}

#[derive(Clone)]
struct ServerContext {
    pub start_time: DateTime<Utc>,
    pub host: String,
    pub version: String
}
