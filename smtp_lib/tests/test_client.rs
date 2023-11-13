use smtp_lib::{
    commands::{
        async_traits::AsyncSMTPCommand,
        command_impls::{EhloCommand, EhloCommandData},
    },
    smtp_client::{async_traits::AsyncSMTPClient, SMTPClient},
    BoxSendFuture, END_OF_MULTILINE_COMMAND,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug)]
pub struct TestClient {
    pub connection: TcpStream,
}
impl SMTPClient for TestClient {
    fn get_hostname(&self) -> &str {
        "test.smtp_lib.rs"
    }

    fn get_end_of_multiline_command(&self) -> &str {
        END_OF_MULTILINE_COMMAND
    }
}
impl<'a> AsyncSMTPClient<'a> for TestClient {
    type ReadLineFuture = BoxSendFuture<'a, smtp_lib::Result<String>>;
    type WriteFuture = BoxSendFuture<'a, smtp_lib::Result<()>>;
    type ReadTilEndFuture = BoxSendFuture<'a, smtp_lib::Result<String>>;
    type ReadTilNonHyphenatedLine = BoxSendFuture<'a, smtp_lib::Result<Vec<String>>>;

    fn read_line(&'a mut self) -> Self::ReadLineFuture {
        Box::pin(async move {
            let mut total_bytes_read = 0;
            let mut buffer = Vec::with_capacity(50);
            loop {
                let bytes_read = self.connection.read_buf(&mut buffer).await?;
                total_bytes_read += bytes_read;
                let end = std::str::from_utf8(&(buffer[(total_bytes_read - 2)..total_bytes_read]));
                if let Ok(end) = end {
                    if end == "\r\n" {
                        break;
                    }
                }
            }
            let line = String::from_utf8(buffer.to_vec())?;
            Ok(line)
        })
    }

    fn write_string(&'a mut self, command: String) -> Self::WriteFuture {
        Box::pin(async move {
            self.connection.write_all(command.as_bytes()).await?;
            Ok(())
        })
    }

    fn read_til_non_hyphenated_line(&'a mut self) -> Self::ReadTilNonHyphenatedLine {
        Box::pin(async move {
            let mut total_bytes_read = 0;
            let mut buffer = Vec::with_capacity(50);
            loop {
                let bytes_read = self.connection.read_buf(&mut buffer).await?;
                total_bytes_read += bytes_read;
                if buffer.ends_with(b"\r\n") {
                    let content = std::str::from_utf8(&(buffer[..total_bytes_read]));
                    if let Ok(content) = content {
                        let mut lines: Vec<_> = content.split("\r\n").collect();
                        if let Some(last_line) = lines.get(lines.len() - 2) {
                            if last_line.chars().nth(3) == Some(' ') {
                                buffer.truncate(total_bytes_read - 2);
                                break;
                            }
                        }
                    }
                }
            }
            // We checked the UTF-8 validity of the buffer above, so this is safe
            unsafe {
                let lines = String::from_utf8_unchecked(buffer);
                Ok(lines.split("\r\n").map(|s| s.to_string()).collect())
            }
        })
    }

    fn read_til_end(&'a mut self) -> Self::ReadTilEndFuture {
        Box::pin(async move {
            let mut buffer = [0u8; 1024];
            let mut line = String::new();
            loop {
                let bytes_read = self.connection.read(&mut buffer).await?;
                let line_read = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                line.push_str(&line_read);
                if line.ends_with(END_OF_MULTILINE_COMMAND) {
                    break;
                }
            }
            Ok(line)
        })
    }
}
#[tokio::test]
pub async fn test_ehlo() -> anyhow::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:25").await?;
    let mut client = TestClient { connection: stream };
    let string = client.read_line().await?;
    println!("Server said: {}", string);
    EhloCommand::client_send(
        EhloCommandData {
            client_hostname: "test.smtp_lib.rs".to_string(),
        },
        &mut client,
    )
    .await?;
    let ehlo_response = EhloCommand::client_read(&mut client).await?;
    println!("{:#?}", ehlo_response);

    Ok(())
}
