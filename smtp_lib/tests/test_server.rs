use bytes::Bytes;
use smtp_lib::smtp_server::async_traits::AsyncSMTPConnection;
use smtp_lib::smtp_server::{SMTPConnection, SMTPServer};
use smtp_lib::SMTPConnectionState;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub struct TestSMTPServer {}

impl SMTPServer for TestSMTPServer {
    fn get_hostname(&self) -> &str {
        "test.com"
    }

    fn name(&self) -> &str {
        "Test Server"
    }

    fn get_greeting(&self) -> Option<&str> {
        None
    }
}
#[derive(Debug)]
pub struct TestSMTPConnection {
    stream: TcpStream,
    server: Arc<TestSMTPServer>,
}
impl SMTPConnection for TestSMTPConnection {
    type Server = TestSMTPServer;

    fn get_server(&self) -> &Self::Server {
        self.server.as_ref()
    }

    fn get_state(&self) -> &SMTPConnectionState {
        todo!()
    }

    fn set_state(&mut self, state: SMTPConnectionState) {
        todo!()
    }

    fn get_end_of_multiline_command(&self) -> &str {
        todo!()
    }
}

impl<'b> AsyncSMTPConnection<'b> for TestSMTPConnection {
    type ReadLineFuture = smtp_lib::BoxSendFuture<'b, smtp_lib::Result<String>>;
    type WriteFuture = smtp_lib::BoxSendFuture<'b, smtp_lib::Result<()>>;
    type ReadTilEndFuture = smtp_lib::BoxSendFuture<'b, smtp_lib::Result<String>>;

    fn read_line(&mut self) -> Self::ReadLineFuture {
        todo!()
    }

    fn write(&mut self, command: Bytes) -> Self::WriteFuture {
        todo!()
    }

    fn read_til_end(&mut self) -> Self::ReadTilEndFuture {
        todo!()
    }
}
#[tokio::test]
async fn test() {
    let server = TestSMTPServer {};
    let stream = TcpListener::bind("0.0.0.0:2525").await.unwrap();
}
