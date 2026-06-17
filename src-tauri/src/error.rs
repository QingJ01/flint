use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlintError {
    #[error("command not found: {0}")]
    NotFound(String),
    #[error("command failed ({code}): {stderr}")]
    CommandFailed { code: i32, stderr: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Serialize, Clone)]
pub struct ErrorPayload {
    pub kind: &'static str,
    pub message: String,
}

impl From<&FlintError> for ErrorPayload {
    fn from(e: &FlintError) -> Self {
        let kind = match e {
            FlintError::NotFound(_) => "not_found",
            FlintError::CommandFailed { .. } => "command_failed",
            FlintError::Io(_) => "io",
            FlintError::Parse(_) => "parse",
            FlintError::Other(_) => "other",
        };
        ErrorPayload { kind, message: e.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, FlintError>;
