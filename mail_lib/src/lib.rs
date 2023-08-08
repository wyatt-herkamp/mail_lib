pub use smtp_lib::{
    Result as SMTPResult,
    smtp_client::*,
    smtp_server::*,
    commands::*,
    commands::command_impls::*,
};

#[cfg(feature = "async")]
pub use smtp_lib::{
    smtp_server::async_traits::*,
    smtp_client::AsyncSMTPClient,
    commands::async_traits::*,
};
pub use common::{
    credentials::*,
    email_address::*
};

#[test]
pub fn test() {}
