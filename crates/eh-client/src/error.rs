use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ErrorKind {
    Network,
    Http,
    Parse,
    Cloudflare,
    Banned,
    LoginRequired,
    LoginFailed,
    NoExAccess,
    ImageLimit,
    NotFound,
    InvalidInput,
    Io,
    Db,
    Cancelled,
    Internal,
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("error[{kind:?}]: {message}")]
pub struct EhError {
    pub kind: ErrorKind,
    pub message: String,
}

impl EhError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn network(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Network, msg)
    }

    pub fn http(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Http, msg)
    }

    pub fn parse(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Parse, msg)
    }

    pub fn cloudflare(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Cloudflare, msg)
    }

    pub fn banned(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Banned, msg)
    }

    pub fn login_required(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::LoginRequired, msg)
    }

    pub fn login_failed(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::LoginFailed, msg)
    }

    pub fn no_ex_access(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::NoExAccess, msg)
    }

    pub fn image_limit(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::ImageLimit, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, msg)
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, msg)
    }

    pub fn io(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, msg)
    }

    pub fn db(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Db, msg)
    }

    pub fn cancelled(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Cancelled, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, msg)
    }
}

pub type Result<T, E = EhError> = std::result::Result<T, E>;
