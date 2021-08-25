pub mod message_parsing;
pub mod replies;

use std::{collections::VecDeque, io::{self, BufRead, Read, Write}, net::{TcpListener, TcpStream}, str::{FromStr}, thread};
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

fn get_messages<T: BufRead>(reader: &mut T) -> io::Result<Vec<String>> {
    let bytes = reader.fill_buf()?;
    let bytes_read = bytes.len();

    match std::str::from_utf8(bytes) {
        Ok(s) => {
            let raw_payload = s.to_owned();

            println!("RECEIVED {}", raw_payload);

            // map to owned String so the ownership can be moved out of this function scope
            let raw_messages = raw_payload.split_terminator("\r\n").map(|s| s.to_string()).collect();
            reader.consume(bytes_read);
            Ok(raw_messages)
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

// TODO finish this test
/*
#[test]
fn get_messages_larger_than_maxlen_throwsawaybytes() {
    let fake_buffer = b"Hello world".repeat(47).to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(512);
    faked_responses.push_back(5);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses
    };

    let result = get_messages(&mut faked_bufreader).unwrap();
    assert_eq!(1000, result.len());
    assert_eq!("Hello world", result.first().unwrap());
}
*/

struct FakeBufReader {
    fake_buffer: Vec<u8>,
    faked_responses: VecDeque<usize> // implement as a queue so you can mock which bytes returned on each read call
}

impl BufRead for FakeBufReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let result = match self.faked_responses.pop_front() {
            Some(b) => {
                Ok(&self.fake_buffer[..b])
            },
            None => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "bad test setup"))
        };

        result
    }

    fn consume(&mut self, amt: usize) {
        self.fake_buffer.drain(..amt);
        ()
    }
}

impl Read for FakeBufReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }
}

fn handle_connection(mut stream: TcpStream, context: ServerContext) -> io::Result<()> {
    // connection handler just runs a loop that reads bytes off the stream
    // and sends responses based on logic or until the connection has died
    // there also needs to be a ping loop going on that can stop this loop too

    let mut write_handle = stream.try_clone()?;
    let mut reader = io::BufReader::new(stream);

    loop {
        
        let raw_messages = get_messages(&mut reader)?;

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
                    write_handle.write(reply.as_bytes())?;
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
    pub version: String
}
