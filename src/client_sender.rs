use tokio::sync::mpsc;
use tokio::sync::broadcast;

use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use uuid::Uuid;

use crate::replies::Reply;
use crate::result::Result;

pub async fn run_sender(
    connection_id: &Uuid,
    write_handle: &mut OwnedWriteHalf,
    mut reply_receiver: mpsc::Receiver<Reply>,
    mut shutdown_receiver: broadcast::Receiver<()>
) -> Result<()> {
    let sender_connection_id = connection_id;

    loop {
        let received = tokio::select! {
            received = reply_receiver.recv() => match received {
                Some(r) => r,
                // TODO RecvError just seems harmless just means channel has been dropped?
                None => return Ok(()),
            },
            _ = shutdown_receiver.recv() => {
                return Ok(());
            }
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

        if let Err(e) = write_handle.write_all(reply.as_bytes()).await {
            println!("Error writing reply {} {:?}", reply, e);
        }
    }
}
