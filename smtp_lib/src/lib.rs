pub mod commands;
pub mod error;
pub mod server_response;
pub mod smtp_client;
pub mod smtp_server;
pub mod statement;

pub type Result<T> = std::result::Result<T, error::SMTPError>;
/// By default, SMTP uses '.\n' as the end of a multiline command
pub static END_OF_MULTILINE_COMMAND: &str = ".\n";
pub static END_OF_COMMAND: &str = "\n";
pub static CRLF: &str = "\r\n";

pub type BoxSendFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SMTPConnectionState {
    Connected,
    Helo,
}
