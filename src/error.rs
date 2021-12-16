use std::fmt::Debug;
use std::fmt::Display;
use std::io;
use std::sync::mpsc::RecvError;

// should these be separate enums? what is idiomatic way to manage a large
// error enum?
#[derive(Debug, PartialEq)]
pub enum Error {
    MessageReadingErrorNotUtf8,
    MessageReadingErrorNoMessageSeparatorProvided,
    MessageReadingErrorLastMessageMissingSeparator,
    MessageReadingErrorStreamClosed,
    MessageReadingErrorIoFailure,
    MessageParsingErrorMissingCommand,
    MessageParsingErrorMissingParameter { param_name: String },
    MessageParsingErrorInvalidChannelFormat { provided_channel: String },
    ClientListenerFailed(IoError),
    ClientToServerChannelFailedToReceive(RecvError),
    TestErrorNoMoreMessagesInReceiver,
}

// there isn't an impl for PartialEq for io::Error (probably for good reason)
// but we can wrap around it using "new type" pattern and impl the traits
// we need to fit with the rest of the enum above
// the type its wrapping over needs to be declared as pub to be used "publicly"
pub struct IoError(pub io::Error);

// this is sufficient we just want the Error enum to impl PartialEq for unit testing
// so we can just use assert_eq! against an expected struct
impl PartialEq for IoError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

impl Debug for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("IoError").field(&self.0).finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MessageReadingErrorNotUtf8 => {
                write!(f, "Error reading message(s), must be valid UTF-8")
            }
            Error::MessageReadingErrorNoMessageSeparatorProvided => {
                write!(f, "Error reading message(s), no message separator provided")
            }
            Error::MessageReadingErrorLastMessageMissingSeparator => {
                write!(
                    f,
                    "Error reading message(s), last message is missing separator"
                )
            }
            Error::MessageReadingErrorStreamClosed => {
                write!(f, "Error reading message(s), stream is closed")
            }
            Error::MessageReadingErrorIoFailure => {
                write!(f, "Error reading message(s), IO failure")
            }
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
            Error::ClientListenerFailed(e) => {
                write!(
                    f,
                    "Error from client listener {:?}",
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
            Error::TestErrorNoMoreMessagesInReceiver => {
                write!(f, "")
            }
        }
    }
}
