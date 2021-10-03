use std::{collections::HashMap, sync::{Arc, Mutex, RwLock, mpsc::{Receiver}}};
use uuid::Uuid;

use crate::ConnectionContext;

pub fn run_server(receiver_channel: Receiver<String>, connections: Arc<RwLock<HashMap<Uuid, ConnectionContext>>>) {
    // who needs to be pinged?
    // who should I have received a ping for?
    // whats in my inbox?

    /*
    connections
        .read()
        .unwrap()
        .get(&Uuid::new_v4())
        .unwrap()
        .client_sender_channel
        .lock()
        .unwrap()
        .send("BLARGH!".to_string());
    */
    
    for received in receiver_channel.recv() {

    }
}
