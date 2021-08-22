pub mod message_parsing;

use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream}, str::FromStr, thread};
use chrono::{DateTime, Utc};
use crate::message_parsing::*;

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
                    let mut welcome_storm = vec![
                        ReplyWelcome::new(context.host.clone(), "WELCOME TO THE SERVER".to_string(), c.nick.clone()),
                        ReplyYourHost::new(context.host.clone(), context.version.clone(), c.nick.clone()),
                        ReplyCreated::new(context.host.clone(), c.nick.clone(), "This server was created".to_string(), context.start_time.clone()),
                        ReplyMyInfo::new(context.host.clone(), c.nick.clone(), context.version.clone(), "r".to_string(), "i".to_string()),
                        ReplySupport::new(context.host.clone(), c.nick.clone(), 32 /* TODO make this configurable */),
                        ReplyLUserClient::new(context.host.clone(), c.nick.clone(), 100, 20, 1 /* TODO make these live update */),
                        ReplyLUserOp::new(context.host.clone(), c.nick.clone(), 1337),
                        ReplyLUserUnknown::new(context.host.clone(), c.nick.clone(), 7),
                        ReplyLUserChannels::new(context.host.clone(), c.nick.clone(), 9999),
                        ReplyLUserMe::new(context.host.clone(), c.nick.clone(), 900, 1),
                        ReplyLocalUsers::new(context.host.clone(), c.nick.clone(), 845, 1000),
                        ReplyGlobalUsers::new(context.host.clone(), c.nick.clone(), 9823, 23455),
                        ReplyStatsDLine::new(context.host.clone(), c.nick.clone(), 9998, 9000, 99999)
                    ];

                    // TODO proper configurable MOTD
                    let mut motd_replies = vec![ReplyMotd::new(context.host.clone(), c.nick.clone(), "Foobar".to_string())];
                    welcome_storm.push(ReplyMotdStart::new(context.host.clone(), c.nick.clone()));
                    welcome_storm.append(&mut motd_replies);
                    welcome_storm.push(ReplyEndOfMotd::new(context.host.clone(), c.nick.clone()));
                    
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
