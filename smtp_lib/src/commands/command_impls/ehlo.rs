use crate::commands::SMTPCommand;
use crate::error::SMTPError;
use crate::server_response::{MultilineServerResponse, ResponseCode};
use crate::smtp_server::SMTPServerExtension;

use std::ops::Deref;

/// The data that is in the EHLO command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EhloCommandData {
    pub client_hostname: String,
}
impl Deref for EhloCommandData {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.client_hostname
    }
}
impl From<String> for EhloCommandData {
    fn from(value: String) -> Self {
        EhloCommandData {
            client_hostname: value,
        }
    }
}
impl Into<String> for EhloCommandData {
    fn into(self) -> String {
        self.client_hostname
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EhloResponse {
    Error {
        code: ResponseCode,
        message: String,
    },
    Success {
        // TODO can hello have no message?
        hello: (ResponseCode, String),
        extensions: Vec<SMTPServerExtension>,
    },
}
impl TryFrom<MultilineServerResponse> for EhloResponse {
    type Error = SMTPError;

    fn try_from(value: MultilineServerResponse) -> Result<Self, SMTPError> {
        let mut lines = value.0;
        if lines.len() == 0 {
            return Ok(EhloResponse::Error {
                code: 500.into(),
                message: "No response lines".to_string(),
            });
        }
        if lines[0].code != 250u16 {
            return Ok(EhloResponse::Error {
                code: lines[0].code,
                message: lines[0]
                    .message
                    .clone()
                    .unwrap_or_else(|| "No message".to_string()),
            });
        }
        let hello = lines.remove(0);
        let mut extensions = Vec::with_capacity(lines.len());
        for line in lines {
            let Some(message) = line.message else {
                return Err(SMTPError::InvalidResponse("Expected extension, got none"))
            };
            let extension = SMTPServerExtension::try_from(message)?;
            extensions.push(extension);
        }
        let Some(hello_message) = hello.message else{
            return Err(SMTPError::InvalidResponse("Expected hello message, got none"))
        };
        Ok(EhloResponse::Success {
            hello: (hello.code, hello_message),
            extensions,
        })
    }
}

/// The EHLO Command as specified [here](https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.1)
pub struct EhloCommand;
impl SMTPCommand for EhloCommand {
    type ClientCommand = EhloCommandData;
    type ServerResponse = EhloResponse;

    fn command() -> &'static str
    where
        Self: Sized,
    {
        "EHLO"
    }
}

#[cfg(feature = "async")]
mod async_impl {
    use crate::commands::async_traits::AsyncSMTPCommand;
    use super::{EhloCommand, EhloCommandData, EhloResponse};

    use crate::server_response::{MultilineServerResponse, ResponseCode, ServerResponseLine};
    use crate::smtp_server::async_traits::AsyncSMTPConnection;
    use crate::statement::async_statement::AsyncMultilineStatement;
    use crate::CRLF;
    use futures::future::{ready, BoxFuture, Ready};
    use crate::smtp_client::async_traits::AsyncSMTPClient;

    impl<'a> AsyncSMTPCommand<'a> for EhloCommand {
        type ServerHandleRead = Ready<crate::Result<Self::ClientCommand>>;
        type SendFuture = BoxFuture<'a, crate::Result<()>>;
        type HandleServerResponse = BoxFuture<'a, crate::Result<EhloResponse>>;

        fn server_read<'b, C: AsyncSMTPConnection<'b>>(
            _: &mut C,
            line: String,
        ) -> Self::ServerHandleRead
        where
            Self: Sized,
        {
            let (_, host_name) = line.split_at(5);

            ready(Ok(EhloCommandData::from(host_name.to_string())))
        }

        fn server_send<C: AsyncSMTPConnection<'a>>(
            response: Self::ServerResponse,
            client: &'a mut C,
        ) -> Self::SendFuture {
            Box::pin(async move {
                match response {
                    EhloResponse::Error { code, message } => {
                        let statement = ServerResponseLine::new(code, Some(message));
                        client.write_statement(statement).await?;
                    }
                    EhloResponse::Success { hello, extensions } => {
                        let mut statement = MultilineServerResponse::new_with_code(
                            hello.0,
                            Some(hello.1),
                            extensions.len(),
                        );
                        for extension in extensions {
                            statement.add_line(ResponseCode::Ok, Some(extension.to_string()));
                        }
                        client.write_statement(statement).await?;
                    }
                }
                Ok(())
            })
        }

        fn client_send<C: AsyncSMTPClient<'a>>(
            command: Self::ClientCommand,
            client: &'a mut C,
        ) -> Self::SendFuture {
            Box::pin(async move {
                let command = format!("EHLO {}{}", command.client_hostname, CRLF);
                client.write_string(command).await?;
                Ok(())
            })
        }

        fn client_read<C: AsyncSMTPClient<'a>>(client: &'a mut C) -> Self::HandleServerResponse {
            Box::pin(async {
                let multiline =
                    MultilineServerResponse::read_til_non_hyphenated_line(client).await?;
                let response = EhloResponse::try_from(multiline)?;
                Ok(response)
            })
        }
    }
}
