use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;

use uuid::Uuid;

use crate::replies::Reply;
use crate::result::Result;

pub fn run_sender(
    receiver: Receiver<Reply>,
    write_handle: &mut TcpStream,
    connection_id: &Uuid,
) -> Result<()> {
    let sender_connection_id = connection_id;

    loop {
        let received = match receiver.recv() {
            Ok(r) => r,
            // TODO RecvError just seems harmless just means channel has been dropped?
            Err(_) => return Ok(()),
        };

        // The Quit message handler always sends at least 1 message
        // to the quitting user, so that this thread is able to stop
        // itself
        // TODO is this necessary, according to docs channel will
        // be usable even if disconnected until its flushed??
        if let Reply::Quit { connection_id, .. } = received {
            if &connection_id == sender_connection_id {
                return Ok(());
            }
        }

        let reply = &format!("{}{}", &received.to_string(), "\r\n");

        if let Err(e) = write_handle.write_all(reply.as_bytes()) {
            println!("Error writing reply {} {:?}", reply, e);
        }

        if let Err(e) = write_handle.flush() {
            println!("Error flushing write handle {:?}", e);
        }
    }
}
