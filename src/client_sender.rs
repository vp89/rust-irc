use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;

use crate::replies::Reply;
use crate::result::Result;
use crate::error::Error::ServerToClientChannelFailedToReceive;

pub fn run_sender(receiver: Receiver<Reply>, write_handle: &mut TcpStream) -> Result<()> {
    loop {
        let received = receiver
            .recv()
            .map_err(ServerToClientChannelFailedToReceive)?;

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
