pub mod credentials;
pub mod email_address;
pub mod mail_box;
pub(crate) mod parsers;
pub use email_address::EmailAddress;
pub type BoxSendFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
