use std::io::{self, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver};
use uuid::Uuid;

use crate::replies::Reply;

pub fn run_sender<'a>(connection_uuid: &Uuid, receiver: Receiver<Reply>, write_handle: &mut TcpStream) -> io::Result<()> {
    loop {
        for received in receiver.recv() {
            let mut reply = String::new();
            reply.push_str(&received.to_string());
            reply.push_str("\r\n");
            println!("SENDING {}", reply);
            write_handle.write_all(reply.as_bytes())?;
            write_handle.flush()?;
        }
    }

    Ok(())
}
