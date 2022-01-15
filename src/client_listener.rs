use crate::error::Error::*;
use crate::replies::Reply;
use crate::result::Result;
use crate::{
    message_parsing::{ClientToServerCommand, ClientToServerMessage},
    ServerContext,
};

use std::pin::Pin;
use std::task::{Context, Poll};
use std::{
    collections::VecDeque,
    io::{self, ErrorKind},
    time::Instant,
};
use tokio::io::AsyncBufRead;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use pin_project_lite::pin_project;

pub async fn run(
    context: ServerContext,
    connection_id: &Uuid,
    stream: &mut OwnedReadHalf,
    message_sender: &Sender<ClientToServerMessage>,
    reply_sender: Sender<Reply>,
    mut shutdown_receiver: Receiver<()>,
) -> Result<()> {
    // connection handler just runs a loop that reads bytes off the stream
    // and sends responses based on logic or until the connection has died
    // there also needs to be a ping loop going on that can stop this loop too

    let mut reader = BufReader::with_capacity(512, stream);
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

            if let Err(e) = reply_sender
                .send(Reply::Ping {
                    server_host: server_host.clone(),
                })
                .await
            {
                println!("Error forwarding PING to client sender channel {:?}", e);
            }
        }

        let raw_messages = tokio::select! {
            raw_messages = get_messages(&mut reader) => match raw_messages {
                Ok(m) => m,
                Err(e) => match e {
                    MessageReadingErrorStreamClosed => return Ok(()),
                    _ => {
                        continue;
                    }
                },
            },
            _ = shutdown_receiver.recv() => {
                return Ok(());
            }
        };

        for raw_message in &raw_messages {
            let message = match ClientToServerMessage::from_str(raw_message, *connection_id) {
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
                ClientToServerCommand::Pong => {
                    last_pong = Instant::now();
                    waiting_for_pong = false;
                }
                // If client is quitting, stop this thread but before that pass the message
                // down to the server so it can tell other clients of the QUIT and perform
                // any other necessary shutdown work
                ClientToServerCommand::Quit { .. } => {
                    if let Err(e) = message_sender.send(message.clone()).await {
                        println!("Error forwarding message to server {:?}", e);
                    }

                    return Ok(());
                }
                _ => {
                    if let Err(e) = message_sender.send(message.clone()).await {
                        println!("Error forwarding message to server {:?}", e);
                    }
                }
            }
        }
    }
}

async fn get_messages<T: AsyncBufRead + Unpin>(reader: &mut T) -> Result<Vec<String>> {
    let bytes = match reader.fill_buf().await {
        Ok(s) => Ok(s),
        Err(e) => {
            match e.kind() {
                // This particular ErrorKind is returned on Unix platforms
                // if the TcpStream timed out per the read_timeout setting
                // would need to test that on Windows if that became a goal to
                // support both of those.
                ErrorKind::WouldBlock => return Ok(vec![]),
                _ => Err(MessageReadingErrorIoFailure),
            }
        }
    }?;

    let bytes_read = bytes.len();

    if bytes_read == 0 {
        return Err(MessageReadingErrorStreamClosed);
    }

    match std::str::from_utf8(bytes) {
        Ok(s) => {
            let raw_payload = s.to_owned();

            // map to owned String so the ownership can be moved out of this function scope
            let mut split_messages: Vec<String> =
                raw_payload.split("\r\n").map(|s| s.to_string()).collect();

            if split_messages.len() <= 1 {
                Err(MessageReadingErrorNoMessageSeparatorProvided)
            } else if split_messages.last().unwrap_or(&"BLAH".to_string()) != &format!("") {
                Err(MessageReadingErrorLastMessageMissingSeparator)
            } else {
                split_messages.truncate(split_messages.len() - 1);
                reader.consume(bytes_read);
                Ok(split_messages)
            }
        }
        Err(_) => Err(MessageReadingErrorNotUtf8),
    }
}

#[tokio::test]
async fn get_messages_reads_from_buffer() {
    let fake_buffer = b"Hello world\r\n".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(13);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses,
    };

    let result = get_messages(&mut faked_bufreader).await.unwrap();
    assert_eq!(1, result.len());
    assert_eq!("Hello world", result.first().unwrap());
    assert_eq!(0, faked_bufreader.fake_buffer.len());
}

#[tokio::test]
async fn get_messages_multiplemessages_reads_from_buffer() {
    let fake_buffer = b"Hello world\r\nFoobar\r\n".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(21);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses,
    };

    let result = get_messages(&mut faked_bufreader).await.unwrap();
    assert_eq!(2, result.len());
    assert_eq!("Hello world", result.first().unwrap());
    assert_eq!("Foobar", result[1]);
    assert_eq!(0, faked_bufreader.fake_buffer.len());
}

#[tokio::test]
async fn get_messages_multiplemessages_noterminator_errors() {
    let fake_buffer = b"Hello world\r\nFoobar".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(19);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses,
    };

    let result = get_messages(&mut faked_bufreader)
        .await
        .expect_err("Testing expect an error to be returned here");
    assert_eq!(
        "Error reading message(s), last message is missing separator",
        result.to_string()
    );
}

#[tokio::test]
async fn get_messages_nolineterminator_errors() {
    let fake_buffer = b"Hello world".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(11);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses,
    };

    let result = get_messages(&mut faked_bufreader)
        .await
        .expect_err("Testing expect an error to be returned here");
    assert_eq!(
        "Error reading message(s), no message separator provided",
        result.to_string()
    );
}

#[tokio::test]
async fn get_messages_emptybuffer_errors() {
    let fake_buffer = b"".to_vec();
    let mut faked_responses = VecDeque::new();
    faked_responses.push_back(0);
    let mut faked_bufreader = FakeBufReader {
        fake_buffer,
        faked_responses,
    };

    get_messages(&mut faked_bufreader)
        .await
        .expect_err("Testing expect an error to be returned here");
}

pin_project! {
    struct FakeBufReader {
        fake_buffer: Vec<u8>,
        faked_responses: VecDeque<usize>, // implement as a queue so you can mock which bytes returned on each read call
    }
}

impl AsyncBufRead for FakeBufReader {
    fn poll_fill_buf(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<io::Result<&[u8]>> {
        let me = self.project();

        match me.faked_responses.pop_front() {
            Some(b) => Poll::Ready(Ok(&me.fake_buffer[..b])),
            None => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "bad test setup",
            ))),
        }
    }

    fn consume(self: Pin<&mut FakeBufReader>, amt: usize) {
        let me = self.project();
        me.fake_buffer.drain(..amt);
    }
}

// just adding this to make compiler happy, for testing we only need
// the methods in BufRead
impl AsyncRead for FakeBufReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        todo!()
    }
}
