use crate::smtp_server::ServerExtensionParseError;
use crate::statement::StatementParseError;

use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SMTPError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("UTF8 Error: {0}")]
    UTF8(#[from] FromUtf8Error),
    #[error("Unable to parse: {0}")]
    InvalidStatement(#[from] StatementParseError),
    #[error("Invalid Extension: {0}")]
    InvalidExtension(#[from] ServerExtensionParseError),
    #[error("Invalid Response: {0}")]
    InvalidResponse(&'static str),
}
impl SMTPError {
    pub fn get_error_code(&self) -> u16 {
        match self {
            SMTPError::InvalidCommand(_) => 502,
            _ => 0,
        }
    }
}
