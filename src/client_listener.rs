use crate::{
    message_parsing::{ClientToServerCommand, ClientToServerMessage},
    replies::Reply,
    ServerContext,
};
use std::{
    collections::VecDeque,
    io::{self, BufRead, ErrorKind, Read, Write},
    net::TcpStream,
    time::Instant,
};
use std::{net::SocketAddr, sync::mpsc::Sender};
use uuid::Uuid;

pub fn run_listener(
    connection_id: &Uuid,
    stream: &TcpStream,
    server_sender: Sender<ClientToServerMessage>,
    client_sender: Sender<Reply>,
    client_ip: Option<SocketAddr>,
    context: ServerContext,
) -> io::Result<()> {
    // connection handler just runs a loop that reads bytes off the stream
    // and sends responses based on logic or until the connection has died
    // there also needs to be a ping loop going on that can stop this loop too
    let mut write_handle = stream.try_clone()?;
    let mut reader = io::BufReader::with_capacity(512, stream);
    let mut last_pong = Instant::now();
    let mut waiting_for_pong = false;
    let server_host = &context.server_host;

    loop {
        if waiting_for_pong && last_pong.elapsed().as_secs() > context.ping_frequency.as_secs() + 5
        {
            println!(
                "No pong received, last pong received {} secs ago. Closing down listener",
                last_pong.elapsed().as_secs()
            );
            return Ok(());
        }

        if last_pong.elapsed().as_secs() > context.ping_frequency.as_secs() {
            println!("Sending ping");

            waiting_for_pong = true;
            let ping = format!(
                "{}\r\n",
                Reply::Ping {
                    server_host: server_host.clone()
                }
                .to_string()
            );
            write_handle.write_all(ping.as_bytes())?;
            write_handle.flush()?;
        }

        let raw_messages = match get_messages(&mut reader) {
            Ok(m) => m,
            Err(e) => {
                println!("Error getting valid messages from the reader {:?}", e);
                continue;
            }
        };

        for raw_message in &raw_messages {
            // TODO is there a way to clean this up so that I only pass in sender/IP on parsing
            // when I need it? Like a multi-pass parsing?
            let message = match ClientToServerMessage::from_str(
                raw_message,
                *connection_id,
                &client_sender,
                client_ip,
            ) {
                Ok(m) => m,
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            };

            match &message.command {
                ClientToServerCommand::Unhandled => {
                    println!("Unhandled message received {:?} {}", message, raw_message);
                }
                ClientToServerCommand::Ping { token } => {
                    let pong = format!(
                        "{}\r\n",
                        Reply::Pong {
                            server_host: server_host.clone(),
                            token: token.clone()
                        }
                        .to_string()
                    );
                    write_handle.write_all(pong.as_bytes())?;
                    write_handle.flush()?;
                }
                ClientToServerCommand::Pong => {
                    last_pong = Instant::now();
                    waiting_for_pong = false;
                }
                ClientToServerCommand::Quit => {
                    // TODO should this send something to the server worker?
                    return Ok(());
                }
                _ => {
                    if let Err(e) = server_sender.send(message.clone()) {
                        println!("Error forwarding message to server {:?}", e);
                    }
                }
            }
        }
    }
}

fn get_messages<T: BufRead>(reader: &mut T) -> io::Result<Vec<String>> {
    // TODO can this be cleaned up?
    let bytes = match reader.fill_buf() {
        Ok(s) => Ok(s),
        Err(e) => match e.kind() {
            // This particular ErrorKind is returned on Unix platforms
            // if the TcpStream timed out per the read_timeout setting
            // would need to test that on Windows if that became a goal to
            // support both of those.
            ErrorKind::WouldBlock => return Ok(vec![]),
            _ => Err(e),
        },
    }?;

    let bytes_read = bytes.len();

    match std::str::from_utf8(bytes) {
        Ok(s) => {
            let raw_payload = s.to_owned();

            // map to owned String so the ownership can be moved out of this function scope
            let mut split_messages: Vec<String> =
                raw_payload.split("\r\n").map(|s| s.to_string()).collect();

            // TODO create own Error kinds/type??
            if split_messages.len() <= 1 {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "No message separator provided",
                ))
            } else if split_messages.last().unwrap_or(&"BLAH".to_string()) != &format!("") {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Last message did not have expected separator",
                ))
            } else {
                split_messages.truncate(split_messages.len() - 1);
                reader.consume(bytes_read);
                Ok(split_messages)
            }
        }
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[test]
fn get_messages_reads_from_buffer() {
    let fake_buffer = b"Hello world\r\n".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(13);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses,
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
        faked_responses,
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
        faked_responses,
    };

    let result = get_messages(&mut faked_bufreader)
        .expect_err("Testing expect an error to be returned here");
    assert_eq!(
        "Last message did not have expected separator",
        result.to_string()
    );
}

#[test]
fn get_messages_nolineterminator_errors() {
    let fake_buffer = b"Hello world".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(11);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses,
    };

    let result = get_messages(&mut faked_bufreader)
        .expect_err("Testing expect an error to be returned here");
    assert_eq!("No message separator provided", result.to_string());
}

#[test]
fn get_messages_emptybuffer_errors() {
    let fake_buffer = b"".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(0);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses,
    };

    get_messages(&mut faked_bufreader).expect_err("Testing expect an error to be returned here");
}

struct FakeBufReader {
    fake_buffer: Vec<u8>,
    faked_responses: VecDeque<usize>, // implement as a queue so you can mock which bytes returned on each read call
}

impl BufRead for FakeBufReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self.faked_responses.pop_front() {
            Some(b) => Ok(&self.fake_buffer[..b]),
            None => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "bad test setup",
            )),
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
