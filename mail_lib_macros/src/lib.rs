use proc_macro::TokenStream;
pub(crate) mod macros;
/// This macro is used to create a static email address.
///
/// ```rust, no_run
/// struct MyConfig {
///     email: EmailAddress,
/// }
/// impl Default for MyConfig {
///     fn default() -> Self {
///         Self {
///             email: from_static_email!("my_email@my_domain"),
///         }
///     }
/// }
/// ```
/// ## Use Cases
/// This macro is useful when you are creating default values for structs. It will avoid checking the email address at runtime.
///
/// ## No Checking
/// This macro does not check the email address. So if you pass an email that is not valid but still has a `@` symbol, it will still compile.
/// # Note
/// The EmailAddress type must be in scope for this macro to work.
#[cfg(feature = "types_macros")]
#[proc_macro]
pub fn from_static_email(input: TokenStream) -> TokenStream {
    let email_address = syn::parse_macro_input!(input as macros::email_address::EmailAddressMacro);
    email_address.output().into()
}
