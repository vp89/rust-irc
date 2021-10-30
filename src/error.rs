use std::fmt::Debug;
use std::fmt::Display;
use std::sync::mpsc::RecvError;

#[derive(Debug, PartialEq)]
pub enum Error {
    MessageParsingErrorMissingCommand,
    MessageParsingErrorMissingParameter { param_name: String },
    MessageParsingErrorInvalidChannelFormat { provided_channel: String },
    ServerToClientChannelFailedToReceive(RecvError),
    ClientToServerChannelFailedToReceive(RecvError),
    TestErrorNoMoreMessagesInReceiver
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MessageParsingErrorMissingCommand => {
                write!(f, "Error parsing message, command is missing")
            }
            Error::MessageParsingErrorMissingParameter { param_name } => {
                write!(
                    f,
                    "Error parsing message, {} parameter is missing",
                    param_name
                )
            }
            Error::MessageParsingErrorInvalidChannelFormat { provided_channel } => {
                write!(
                    f,
                    "Error parsing message, channel {} does not begin with #",
                    provided_channel
                )
            }
            Error::ServerToClientChannelFailedToReceive(e) => {
                write!(
                    f,
                    "Error receiving outbound message from server worker {:?}",
                    e
                )
            }
            Error::ClientToServerChannelFailedToReceive(e) => {
                write!(
                    f,
                    "Error receiving inbound message to server worker {:?}",
                    e
                )
            }
            Error::TestErrorNoMoreMessagesInReceiver => { write!(f, "") }
        }
    }
}
