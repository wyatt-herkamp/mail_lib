use std::{
    fmt::{Display, Formatter, Write},
    str::FromStr,
};

use bytes::{BufMut, Bytes, BytesMut};

use crate::statement::{
    MultiLineStatement, SingleLineStatement, Statement, StatementParseError, StatementWriteError,
    StatementWriter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseCode {
    Ok,
    // TODO
    Other(u16),
}

impl PartialEq<u16> for ResponseCode {
    fn eq(&self, other: &u16) -> bool {
        match self {
            ResponseCode::Other(code) => code == other,
            ResponseCode::Ok => *other == 250,
        }
    }
}

impl Display for ResponseCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseCode::Other(code) => {
                write!(f, "{}", code)
            }
            ResponseCode::Ok => {
                write!(f, "250")
            }
        }
    }
}
impl From<u16> for ResponseCode {
    fn from(code: u16) -> Self {
        match code {
            250 => ResponseCode::Ok,
            _ => ResponseCode::Other(code),
        }
    }
}
#[derive(Debug)]
pub struct MultilineServerResponse(pub Vec<ServerResponseLine>);

impl MultilineServerResponse {
    pub fn new(lines: Vec<ServerResponseLine>) -> Self {
        Self(lines)
    }

    pub fn new_with_code(code: ResponseCode, message: Option<String>, capacity: usize) -> Self {
        let mut lines = Vec::with_capacity(capacity + 1);
        lines.push(ServerResponseLine::new(code, message));
        Self(lines)
    }
    pub fn add_line(&mut self, code: ResponseCode, message: Option<String>) {
        self.0.push(ServerResponseLine::new(code, message));
    }
}

impl FromStr for MultilineServerResponse {
    type Err = StatementParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.split("\r\n");
        let mut lines_result = Vec::with_capacity(lines.size_hint().0);

        for line in lines {
            lines_result.push(ServerResponseLine::from_str(line)?);
        }
        Ok(MultilineServerResponse(lines_result))
    }
}

impl Statement for MultilineServerResponse {
    fn statement_size(&self) -> usize {
        self.0
            .iter()
            .map(|line| line.statement_size())
            .sum::<usize>()
            + (self.0.len() * 2)
    }

    fn to_bytes(&self) -> Bytes {
        todo!()
    }
}
impl MultiLineStatement for MultilineServerResponse {
    fn from_lines(lines: Vec<String>) -> Result<Self, StatementParseError>
    where
        Self: Sized,
    {
        let mut lines_result = Vec::with_capacity(lines.len());
        for line in lines {
            lines_result.push(ServerResponseLine::from_str(&line)?);
        }
        Ok(MultilineServerResponse(lines_result))
    }
}
#[derive(Debug)]
pub struct ServerResponseLine {
    pub(crate) code: ResponseCode,
    pub(crate) message: Option<String>,
}
impl ServerResponseLine {
    pub fn new(code: ResponseCode, message: Option<String>) -> Self {
        Self { code, message }
    }
}
impl FromStr for ServerResponseLine {
    type Err = StatementParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let code = u16::from_str(&string[0..3])
            .map_err(|_| StatementParseError::InvalidResponseCode(string.to_string()))
            .map(ResponseCode::from)?;
        if string.len() == 3 {
            Ok(Self {
                code,
                message: None,
            })
        } else {
            Ok(Self {
                code,
                message: Some(string[4..].to_string()),
            })
        }
    }
}
impl SingleLineStatement for ServerResponseLine {
    fn to_hyphenated_line(&self) -> String {
        let mut string = String::with_capacity(self.statement_size());
        // Infallible
        let _ = self.write_hyphenated_line(&mut string);
        string
    }

    fn write_hyphenated_line(
        &self,
        buf: &mut impl StatementWriter,
    ) -> Result<(), StatementWriteError> {
        buf.write_str(self.code.to_string().as_str())?;
        buf.write_str("-")?;
        if let Some(message) = &self.message {
            buf.write_str(message.as_str())?;
        }
        buf.write_str("\r\n")?;
        Ok(())
    }

    fn to_spaced_line(&self) -> String {
        let mut string = String::with_capacity(self.statement_size());
        // Infallible
        let _ = self.write_spaced_line(&mut string);
        string
    }

    fn write_spaced_line(&self, buf: &mut impl StatementWriter) -> Result<(), StatementWriteError> {
        buf.write_str(self.code.to_string().as_str())?;
        buf.write_str(" ")?;
        if let Some(message) = &self.message {
            buf.write_str(message.as_str())?;
        }
        buf.write_str("\r\n")?;
        Ok(())
    }
}
impl Statement for ServerResponseLine {
    fn statement_size(&self) -> usize {
        4 + self.message.as_ref().map(|s| s.len()).unwrap_or(0)
    }

    fn to_bytes(&self) -> Bytes {
        let mut bytes =
            BytesMut::with_capacity(4 + self.message.as_ref().map(|s| s.len()).unwrap_or(0));
        bytes
            .write_str(self.code.to_string().as_str())
            .expect("Failed to write code");
        bytes.put_u8(b' ');
        if let Some(message) = &self.message {
            bytes
                .write_str(message.as_str())
                .expect("Failed to write message");
        }
        bytes.put_u8(b'\r');
        bytes.put_u8(b'\n');
        bytes.freeze()
    }
}
