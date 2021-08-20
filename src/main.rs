pub mod message_parsing; // TODO does this do anything?

use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream}, str::FromStr, thread};
use chrono::{DateTime, Utc};
use crate::message_parsing::{ClientToServerCommand, ClientToServerMessage, NumericReply, RplWelcome, ServerReplyMessage, RplCreated, RplYourHost, RplMyInfo, RplISupport};

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

fn handle_connection(mut stream: TcpStream, context: ServerContext) -> io::Result<()> {
    loop {
        // connection handler just runs a loop that reads bytes off the stream
        // and sends responses based on logic or until the connection has died
        // there also needs to be a ping loop going on that can stop this loop too
        let mut buffer = [0;512];
        stream.read(&mut buffer)?;
        let raw_payload = std::str::from_utf8(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        println!("RECEIVED {}", raw_payload);

        let mut raw_messages = raw_payload.lines();

        println!("SPLIT INTO {} MESSAGES", raw_messages.by_ref().count());

        for raw_message in raw_messages {
            let message = ClientToServerMessage::from_str(raw_message).expect("FOO"); // TODO

            println!("RAW MESSAGE {:?}", message);

            let replies = match message.command {
                ClientToServerCommand::Quit => {
                    println!("RECEIVED QUIT");
                    return Ok(()); // is using return not idiomatic?? Look into that
                }
                ClientToServerCommand::Unhandled => {
                    println!("MESSAGE UNHANDLED {:?}", message);
                    None
                },
                ClientToServerCommand::Nick(c) => {
                    let rpl_welcome_message = ServerReplyMessage {
                        source: &context.host,
                        target: c.nick.clone(),
                        reply_number: "001",
                        reply: NumericReply::RplWelcome(RplWelcome {
                            welcome_message: "WELCOME TO THE SERVER",
                            nick: c.nick.clone()
                        })
                    };

                    let rpl_yourhost_message = ServerReplyMessage {
                        source: &context.host,
                        target: c.nick.clone(),
                        reply_number: "002",
                        reply: NumericReply::RplYourHost(RplYourHost {
                            host: &context.host,
                            version: &context.version
                        })
                    };

                    let rpl_created_message = ServerReplyMessage {
                        source: &context.host,
                        target: c.nick.clone(),
                        reply_number: "003",
                        reply: NumericReply::RplCreated(RplCreated {
                            created_message: "This server was created",
                            created_at: &context.start_time
                        })
                    };

                    let rpl_myinfo_message = ServerReplyMessage {
                        source: &context.host,
                        target: c.nick.clone(),
                        reply_number: "004",
                        reply: NumericReply::RplMyInfo(RplMyInfo {
                            host: &context.host,
                            version: &context.version,
                            available_user_modes: "r",
                            available_channel_modes: "i"
                        })
                    };

                    let rpl_isupport_message = ServerReplyMessage {
                        source: &context.host,
                        target: c.nick.clone(),
                        reply_number: "005",
                        reply: NumericReply::RplISupport(RplISupport {
                            channel_len: 32 // TODO make this configurable
                        })
                    };

                    /*
                    let rplmsgs = format!(
                        "
                        :localhost 251 {nick} :There are 100 users and 20 invisible on 1 servers\r\n
                        :localhost 252 {nick} 1337 :IRC Operators online\r\n
                        :localhost 253 {nick} 7 :unknown connection(s)\r\n
                        :localhost 254 {nick} 9999 :channels formed\r\n
                        :localhost 255 {nick} :I have 900 clients and 1 servers\r\n
                        :localhost 265 {nick} 845 1000 :Current local users 845, max 1000\r\n
                        :localhost 266 {nick} 9823 23455 :Current global users 9823, max 23455\r\n
                        :localhost 250 {nick} :Highest connection count: 9998 (9000 clients) (99999 connections received)\r\n
                        :localhost 375 {nick} :- localhost Message of the Day - \r\n
                        :localhost 372 {nick} :- Foobar\r\n
                        :localhost 376 {nick} :End of /MOTD command.");
                    */

                    Some(vec![rpl_welcome_message, rpl_yourhost_message, rpl_created_message, rpl_myinfo_message, rpl_isupport_message])
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
