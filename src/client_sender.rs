use std::io::{self, Error, ErrorKind, Write};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;

use crate::replies::Reply;

pub fn run_sender(receiver: Receiver<Reply>, write_handle: &mut TcpStream) -> io::Result<()> {
    loop {
        let received = match receiver.recv() {
            Ok(reply) => reply,
            Err(_e) => return Err(Error::new(ErrorKind::BrokenPipe, "Sender has disconnected")),
        };

        let reply = &format!("{}{}", &received.to_string(), "\r\n");
        println!("Sending {}", reply);

        if let Err(e) = write_handle.write_all(reply.as_bytes()) {
            println!("Error writing reply {} {:?}", reply, e);
        }

        if let Err(e) = write_handle.flush() {
            println!("Error flushing write handle {:?}", e);
        }
    }
}
