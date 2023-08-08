use std::future::Future;

pub trait SMTPClient {
    fn get_hostname(&self) -> &str;

    fn get_end_of_multiline_command(&self) -> &str;
}
/// ### Notes
/// The Future Types will be dropped when Rust 1.74 goes into beta https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html#timeline-and-roadmap
#[cfg(feature = "async")]
pub trait AsyncSMTPClient<'a>: SMTPClient + Send {
    type ReadLineFuture: Future<Output = crate::Result<String>> + 'a + Send;
    type WriteFuture: Future<Output = crate::Result<()>> + 'a + Send;
    type ReadTilEndFuture: Future<Output = crate::Result<String>> + 'a + Send;
    type ReadTilNonHyphenatedLine: Future<Output = crate::Result<Vec<String>>> + 'a + Send;

    /// Reads the next line from the SMTP Server
    fn read_line(&'a mut self) -> Self::ReadLineFuture;
    ///
    fn write_string(&'a mut self, command: String) -> Self::WriteFuture;

    fn read_til_non_hyphenated_line(&'a mut self) -> Self::ReadTilNonHyphenatedLine;

    /// Reads til
    fn read_til_end(&'a mut self) -> Self::ReadTilEndFuture;
}
