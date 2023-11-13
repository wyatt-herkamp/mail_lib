use std::{fmt::Write, str::FromStr};

use auto_impl::auto_impl;
use bytes::Bytes;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum StatementWriteError {
    #[error("Failed to write statement")]
    GenericWriteError,
    #[error("Failed to write statement: {0}")]
    UTF8Error(#[from] std::str::Utf8Error),
}
#[auto_impl(&mut, Box)]
pub trait StatementWriter {
    fn write_str(&mut self, str: &str) -> Result<(), StatementWriteError>;

    fn write_bytes(&mut self, bytes: impl AsRef<[u8]>) -> Result<(), StatementWriteError>;

    fn write_string(&mut self, string: impl Into<String>) -> Result<(), StatementWriteError>;
}
#[derive(Debug, Error)]
pub enum StatementParseError {
    #[error("Failed to parse statement")]
    TooBig,
    #[error("Failed to parse statement: {0}")]
    InvalidResponseCode(String),
}

/// Infallible except for `write_bytes`
impl StatementWriter for String {
    fn write_str(&mut self, str: &str) -> Result<(), StatementWriteError> {
        self.push_str(str);
        Ok(())
    }

    fn write_bytes(&mut self, bytes: impl AsRef<[u8]>) -> Result<(), StatementWriteError> {
        self.push_str(std::str::from_utf8(bytes.as_ref())?);
        Ok(())
    }

    fn write_string(&mut self, string: impl Into<String>) -> Result<(), StatementWriteError> {
        let string = string.into();
        self.push_str(&string);
        Ok(())
    }
}
#[auto_impl(&,&mut, Box, Arc)]
pub trait Statement: FromStr<Err = StatementParseError> {
    // TODO Maybe Add fn to_bytes(&self) -> &[u8];

    fn statement_size(&self) -> usize;

    fn to_bytes(&self) -> Bytes;
}
pub trait MultiLineStatement: Statement + Send + 'static {
    fn from_lines(lines: Vec<String>) -> Result<Self, StatementParseError>
    where
        Self: Sized;
}
#[auto_impl(&,&mut, Box, Arc)]
pub trait SingleLineStatement: Statement + Send + 'static {
    fn to_hyphenated_line(&self) -> String;

    fn write_hyphenated_line(
        &self,
        buf: &mut impl StatementWriter,
    ) -> Result<(), StatementWriteError>;

    fn to_spaced_line(&self) -> String;

    fn write_spaced_line(&self, buf: &mut impl StatementWriter) -> Result<(), StatementWriteError>;
}

#[cfg(feature = "async")]
pub mod async_statement {
    use crate::{
        error::SMTPError,
        smtp_client::async_traits::AsyncSMTPClient,
        smtp_server::async_traits::AsyncSMTPConnection,
        statement::{MultiLineStatement, Statement},
    };

    /// This Trait is automatically implemented for everything that implements `Statement`
    ///
    /// This Trait is not stable and will changed when Async Traits are stable
    pub trait AsyncStatement<'s>: Sized {
        type ReadClientFuture: futures::Future<Output = Result<Self, SMTPError>> + Send + 's;
        type WriteToClientFuture: futures::Future<Output = Result<(), SMTPError>> + Send + 's;

        fn read_client<'a: 's, C: AsyncSMTPClient<'a>>(client: &'a mut C) -> Self::ReadClientFuture
        where
            Self: Sized;

        fn write_to_client<'b: 's, C: AsyncSMTPConnection<'b>>(
            self,
            client: &'b mut C,
        ) -> Self::WriteToClientFuture;
    }
    pub trait AsyncMultilineStatement<'s>: Sized {
        type ReadTilNonHyphenatedLine: futures::Future<Output = Result<Self, SMTPError>> + Send + 's;

        fn read_til_non_hyphenated_line<'a: 's, C: AsyncSMTPClient<'a>>(
            client: &'a mut C,
        ) -> Self::ReadTilNonHyphenatedLine
        where
            Self: Sized;
    }
    impl<'s, S> AsyncMultilineStatement<'s> for S
    where
        S: MultiLineStatement + Send + 'static,
    {
        type ReadTilNonHyphenatedLine = crate::BoxSendFuture<'s, crate::Result<Self>>;

        fn read_til_non_hyphenated_line<'a: 's, C: AsyncSMTPClient<'a>>(
            client: &'a mut C,
        ) -> Self::ReadTilNonHyphenatedLine
        where
            Self: Sized,
        {
            Box::pin(async {
                let value = client.read_til_non_hyphenated_line().await?;
                Ok(S::from_lines(value).map_err(|e| SMTPError::from(e))?)
            })
        }
    }

    impl<'s, S> AsyncStatement<'s> for S
    where
        S: Statement + Send + 'static,
    {
        type ReadClientFuture = crate::BoxSendFuture<'s, crate::Result<Self>>;
        type WriteToClientFuture = crate::BoxSendFuture<'s, crate::Result<()>>;

        fn read_client<'a: 's, C: AsyncSMTPClient<'a>>(_client: &'a mut C) -> Self::ReadClientFuture
        where
            Self: Sized,
        {
            todo!()
        }

        fn write_to_client<'b: 's, C: AsyncSMTPConnection<'b>>(
            self,
            _client: &'b mut C,
        ) -> Self::WriteToClientFuture {
            todo!()
        }
    }
}
