pub mod command_impls;

pub trait SMTPCommand: Sized {
    /// What type of data a Server would get from a Client
    type ClientCommand;
    /// The Response that a Server would send to a Client
    type ServerResponse;

    fn command() -> &'static str
    where
        Self: Sized;
    fn can_handle(command: &str) -> bool
    where
        Self: Sized,
    {
        command.starts_with(Self::command())
    }
}

pub mod async_traits {
    use crate::commands::SMTPCommand;
    use crate::smtp_client::AsyncSMTPClient;
    use crate::smtp_server::async_traits::AsyncSMTPConnection;
    use std::future::Future;

    /// ### Notes
    /// The Future Types will be dropped when Rust 1.74 goes into beta https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html#timeline-and-roadmap

    pub trait AsyncSMTPCommand<'a>: SMTPCommand {
        type ServerHandleRead: Future<Output = crate::Result<Self::ClientCommand>> + Send + 'a;
        type SendFuture: Future<Output = crate::Result<()>> + Send + 'a;
        type HandleServerResponse: Future<Output = crate::Result<Self::ServerResponse>> + Send + 'a;
        /// Read a command from the client
        fn server_read<'b, C: AsyncSMTPConnection<'b>>(
            client: &mut C,
            line: String,
        ) -> Self::ServerHandleRead
        where
            Self: Sized;

        /// Send a response to the client
        fn server_send<C: AsyncSMTPConnection<'a>>(
            response: Self::ServerResponse,
            client: &'a mut C,
        ) -> Self::SendFuture;
        /// Send a command to the server
        fn client_send<C: AsyncSMTPClient<'a>>(
            command: Self::ClientCommand,
            client: &'a mut C,
        ) -> Self::SendFuture;
        /// Read a response from the server
        fn client_read<C: AsyncSMTPClient<'a>>(client: &'a mut C) -> Self::HandleServerResponse;
    }
}
