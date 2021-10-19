use std::fmt::Display;
use std::fmt::Debug;
use std::sync::mpsc::RecvError;

#[derive(Debug)]
pub enum Error {
    MessageParsingErrorMissingCommand,
    MessageParsingErrorMissingParameter { param_name: String },
    ServerToClientChannelReceiveError(RecvError)
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
            Error::ServerToClientChannelReceiveError(e) => {
                write!(
                    f,
                    "Error receiving outbound message from server worker {:?}",
                    e
                )
            }
        }
    }
}
