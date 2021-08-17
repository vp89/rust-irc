pub mod message_parsing;

use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream}, str::FromStr, thread};

use crate::message_parsing::{ClientToServerCommand, ClientToServerMessage, NumericReply, RplWelcome, ServerReplyMessage, ServerToClientMessage, Source};

fn main() -> io::Result<()> {
    println!("STARTING SERVER ON 127.0.0.1:6667");

    let listener = TcpListener::bind("127.0.0.1:6667")?;
    let mut connection_handles = vec![];

    for connection_attempt in listener.incoming() {
        match connection_attempt {
            Ok(stream) => {
                connection_handles.push(
                    thread::spawn(|| {
                        let conn_outcome = handle_connection(stream);
                        match conn_outcome {
                            Ok(_) => (),
                            Err(e) => println!("ERROR {}", e)
                        }
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

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
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

            match message.command {
                ClientToServerCommand::Nick => {
                    println!("MESSAGE {:?}", message);

                    let nick = message.params;

                    let rpl_welcome_message = ServerReplyMessage {
                        source: "localhost",
                        target: &nick,
                        reply_number: 101,
                        reply: NumericReply::RplWelcome(RplWelcome {
                            welcome_message: "WELCOME TO THE SERVER",
                            nick: &nick
                        })
                    };

                    let rplmsgs = format!(
                        "{}
                        :localhost 002 {nick} :Your host is localhost, running version 0.0.1\r\n
                        :localhost 003 {nick} :This server was created now\r\n
                        :localhost 004 {nick} localhost 0.0.1 r i\r\n
                        :localhost 005 {nick} CHANNELLEN=32 :are supported by this server\r\n
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
                        :localhost 376 {nick} :End of /MOTD command.",
                        rpl_welcome_message.to_string(),
                        nick = nick);

                    println!("SENDING {}", rplmsgs);
                    stream.write(rplmsgs.as_bytes())?;
                    stream.flush()?;
                },
                ClientToServerCommand::Unhandled => {
                    println!("MESSAGE UNHANDLED {:?}", message);
                }
            }
        }
    }
}
