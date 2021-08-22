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
                    Some(vec![
                        ReplyWelcome::new(context.host.clone(), "WELCOME TO THE SERVER".to_string(), c.nick.clone()),
                        ReplyYourHost::new(context.host.clone(), context.version.clone(), c.nick.clone()),
                        ReplyCreated::new(context.host.clone(), c.nick.clone(), "This server was created".to_string(), context.start_time.clone()),
                        ReplyMyInfo::new(context.host.clone(), c.nick.clone(), context.version.clone(), "r".to_string(), "i".to_string()),
                        ReplySupport::new(context.host.clone(), c.nick.clone(), 32 /* TODO make this configurable */),
                        ReplyLUserClient::new(context.host.clone(), c.nick.clone(), 100, 20, 1 /* TODO make these live update */),
                        ReplyLUserOp::new(context.host.clone(), c.nick.clone(), 1337),
                        ReplyLUserUnknown::new(context.host.clone(), c.nick.clone(), 7),
                    ])
                    /*
                    let rplmsgs = format!(
                        ":localhost 254 {nick} 9999 :channels formed\r\n
                        :localhost 255 {nick} :I have 900 clients and 1 servers\r\n
                        :localhost 265 {nick} 845 1000 :Current local users 845, max 1000\r\n
                        :localhost 266 {nick} 9823 23455 :Current global users 9823, max 23455\r\n
                        :localhost 250 {nick} :Highest connection count: 9998 (9000 clients) (99999 connections received)\r\n
                        :localhost 375 {nick} :- localhost Message of the Day - \r\n
                        :localhost 372 {nick} :- Foobar\r\n
                        :localhost 376 {nick} :End of /MOTD command.");
                    */
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
