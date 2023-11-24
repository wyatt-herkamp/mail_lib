/*!
# mail_lib_types

Types Representing the email standard

 */

pub mod email_address;
pub mod mail_box;
pub mod parsers;
pub use email_address::EmailAddress;

/// A type alias for a boxed future that can be sent across threads.
///
/// Once [rust-lang/rust/100013](https://github.com/rust-lang/rust/issues/100013) is fixed we will use `impl Future` instead of `Box<dyn Future>
pub type BoxSendFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
