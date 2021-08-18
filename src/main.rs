pub mod message_parsing; // TODO does this do anything?

use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream}, str::FromStr, thread};
use chrono::{DateTime, Utc};
use crate::message_parsing::{ClientToServerCommand, ClientToServerMessage, NumericReply, RplWelcome, ServerReplyMessage, RplCreated, RplYourHost};

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
        let mut buffer = [0;512];
        stream.read(&mut buffer)?;
        let raw_payload = std::str::from_utf8(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        println!("RECEIVED {}", raw_payload);

        let raw_messages = raw_payload.lines();

        // TODO do we need to clone here?
        println!("SPLIT INTO {} MESSAGES", raw_messages.clone().count());

        for raw_message in raw_messages {
            let message = ClientToServerMessage::from_str(raw_message).expect("FOO"); // TODO
            println!("RAW MESSAGE {:?}", message);
            match message.command {
                ClientToServerCommand::Nick(c) => {
                    let nick = c.nick;

                    let rpl_welcome_message = ServerReplyMessage {
                        source: &context.host,
                        target: &nick,
                        reply_number: "001",
                        reply: NumericReply::RplWelcome(RplWelcome {
                            welcome_message: "WELCOME TO THE SERVER",
                            nick: &nick
                        })
                    };

                    let rpl_yourhost_message = ServerReplyMessage {
                        source: &context.host,
                        target: &nick,
                        reply_number: "002",
                        reply: NumericReply::RplYourHost(RplYourHost {
                            host: &context.host,
                            version: &context.version
                        })
                    };

                    let rpl_created_message = ServerReplyMessage {
                        source: &context.host,
                        target: &nick,
                        reply_number: "003",
                        reply: NumericReply::RplCreated(RplCreated {
                            created_message: "This server was created",
                            created_at: &context.start_time
                        })
                    };

                    let rplmsgs = format!(
                        "{}\r\n{}\r\n{}\r\n:localhost 004 {nick} localhost 0.0.1 r i\r\n:localhost 005 {nick} CHANNELLEN=32 :are supported by this server\r\n:localhost 251 {nick} :There are 100 users and 20 invisible on 1 servers\r\n:localhost 252 {nick} 1337 :IRC Operators online\r\n:localhost 253 {nick} 7 :unknown connection(s)\r\n:localhost 254 {nick} 9999 :channels formed\r\n:localhost 255 {nick} :I have 900 clients and 1 servers\r\n:localhost 265 {nick} 845 1000 :Current local users 845, max 1000\r\n:localhost 266 {nick} 9823 23455 :Current global users 9823, max 23455\r\n:localhost 250 {nick} :Highest connection count: 9998 (9000 clients) (99999 connections received)\r\n:localhost 375 {nick} :- localhost Message of the Day - \r\n:localhost 372 {nick} :- Foobar\r\n:localhost 376 {nick} :End of /MOTD command.",
                        rpl_welcome_message.to_string(),
                        rpl_yourhost_message.to_string(),
                        rpl_created_message.to_string(),
                        nick = nick);

                    println!("SENDING {}", rplmsgs);
                    stream.write(rplmsgs.as_bytes())?;
                    stream.flush()?;
                },
                ClientToServerCommand::Quit => {
                    println!("RECEIVED QUIT");
                    return Ok(());
                }
                ClientToServerCommand::Unhandled => {
                    println!("MESSAGE UNHANDLED {:?}", message);
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
