use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EarError {
    #[error("serial port is not connected")]
    NotConnected,
    #[error("serial session already active")]
    AlreadyConnected,
    #[error("no active session")]
    NoSession,
    #[error("operation '{0}' is not supported by the connected model")]
    Unsupported(&'static str),
    #[error("model metadata is missing")]
    UnknownModel,
    #[error("timed out while waiting for {0}")]
    Timeout(&'static str),
    #[error("failed to decode packet header")]
    InvalidPacket,
    #[error("incorrect packet checksum")]
    CrcMismatch,
    #[error("failed to detect device identity: {0}")]
    Detection(String),
    #[error("command `{command}` failed: {output}")]
    CommandFailed { command: String, output: String },
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
