use std::io;
use std::sync::mpsc::{Receiver};
use uuid::Uuid;

use crate::message_parsing::ClientToServerMessage;

pub fn run_sender(connection_uuid: &Uuid, receiver: Receiver<ClientToServerMessage>) -> io::Result<()> {
    /*
    let mut reply = String::new();

                for message in messages {
                    reply.push_str(&message.to_string());
                    reply.push_str("\r\n");                        
                }

                println!("SENDING {}", reply);
                write_handle.write_all(reply.as_bytes())?;
                write_handle.flush()?;
                */
    Ok(())
}
