use std::io;
use std::sync::mpsc::{Receiver};
use uuid::Uuid;

pub fn run_sender(connection_uuid: &Uuid, receiver: Receiver<String>) -> io::Result<()> {
    Ok(())
}
