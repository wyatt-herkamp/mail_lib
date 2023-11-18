/*!
 * # Mail_lib
 */
#[doc(inline)]
pub use mail_lib_types;
#[cfg(feature = "async")]
pub use smtp_lib::{
    commands::async_traits::*, smtp_client::async_traits::*, smtp_server::async_traits::*,
};
pub use smtp_lib::{
    commands::{command_impls::*, *},
    error::SMTPError,
    server_response::*,
    smtp_client::*,
    smtp_server::*,
    statement::*,
    Result as SMTPResult, SMTPConnectionState, CRLF, END_OF_MULTILINE_COMMAND,
};

#[test]
pub fn test() {}
