use eh_client::{EhError, ErrorKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Error)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[error("error[{kind:?}]: {message}")]
pub struct IpcError {
    pub kind: ErrorKind,
    pub message: String,
}

impl IpcError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, msg)
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, msg)
    }

    pub fn io(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, msg)
    }
}

impl From<EhError> for IpcError {
    fn from(err: EhError) -> Self {
        Self {
            kind: err.kind,
            message: err.message,
        }
    }
}

impl From<std::io::Error> for IpcError {
    fn from(err: std::io::Error) -> Self {
        Self::new(ErrorKind::Io, err.to_string())
    }
}

impl From<serde_json::Error> for IpcError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(ErrorKind::Parse, err.to_string())
    }
}

pub type Result<T, E = IpcError> = std::result::Result<T, E>;
