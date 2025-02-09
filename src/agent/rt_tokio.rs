use std::{io::ErrorKind, path::Path, time::Duration};

use tokio::{io::AsyncWriteExt, net::UnixStream};

use crate::{
    events::{EventTrait, ResponseTrait},
    Error, Result,
};

use super::{SocketAgent, MAX_BUFFER_SIZE};

impl SocketAgent {
    pub(crate) async fn new<P: AsRef<Path>>(socket_path: P, timeout: Duration) -> Result<Self> {
        // wait the socket
        let wait_future = async { while !std::fs::exists(&socket_path).is_ok_and(|x| x) {} };

        match tokio::time::timeout(timeout, wait_future).await {
            Ok(()) => {
                let stream = UnixStream::connect(socket_path).await?;
                Ok(Self { stream })
            }
            Err(e) => Err(Error::Agent(format!("Connection timed out: {e}"))),
        }
    }

    pub(crate) async fn send_request(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub(crate) async fn recv_response(&mut self) -> Result<Vec<u8>> {
        let mut buf = [0u8; MAX_BUFFER_SIZE];
        let mut vec: Vec<u8> = Vec::new();

        loop {
            self.stream
                .readable()
                .await
                .map_err(|e| Error::Agent(format!("Waiting for stream become readable: {e}")))?;

            match self.stream.try_read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    vec.extend_from_slice(&mut buf);
                    if n < MAX_BUFFER_SIZE {
                        // No need for checking again
                        break;
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(e) => return Err(Error::Agent(format!("Bad read from socket: {e}"))),
            }
        }

        Ok(vec)
    }

    pub(crate) async fn event<E: EventTrait>(
        &mut self,
        event: E,
    ) -> Result<<E as ResponseTrait>::Payload> {
        self.send_request(&event.encode()?).await?;
        let response = self.recv_response().await?;
        E::decode(&response)
    }
}

#[cfg(feature = "_rt-tokio")]
#[cfg(test)]
mod tests {
    use std::{env, fs, path::Path, process::Command, sync::LazyLock, time::Duration};

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UnixListener,
    };

    use crate::{
        agent::SocketAgent,
        events::{GetFirecrackerVersion, ResponseTrait},
        models::Empty,
        Result,
    };

    const FIRECRACKER: LazyLock<String> = LazyLock::new(|| {
        dotenvy::dotenv().ok();
        env::var("FIRECRACKER").unwrap()
    });

    async fn echo_server<P: AsRef<Path>>(api_sock: P) -> Result<()> {
        let listener = UnixListener::bind(&api_sock)?;
        println!("Server listening on {}", api_sock.as_ref().display());
        let (mut stream, _) = listener.accept().await?;
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await {
            Ok(n) if n > 0 => {
                if let Err(e) = stream.write_all(&buffer[0..n]).await {
                    eprintln!("Error writing to stream: {}", e);
                }
            }
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
            }
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")] // important to add `flavor = "multi_thread"`
    async fn test_echo() {
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-tokio-echo.socket";
        const DATA: &'static str = "Hello, world!";
        let _ = fs::remove_file(API_SOCK);

        let server_handle = tokio::spawn(echo_server(API_SOCK));
        let mut agent = SocketAgent::new(API_SOCK, Duration::from_secs(3))
            .await
            .unwrap();
        agent.send_request(DATA.as_bytes()).await.unwrap();
        let response = agent.recv_response().await.unwrap();

        assert_eq!(&response[0..DATA.len()], DATA.as_bytes());

        server_handle.await.unwrap().unwrap();
        let _ = fs::remove_file(API_SOCK);
    }

    #[tokio::test]
    async fn test_get_firecracker_version() {
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-tokio-version.socket";
        const DATA: &'static str = "GET /version HTTP/1.0\r\n\r\n";

        let _ = fs::remove_file(API_SOCK);

        let mut child = Command::new(&*FIRECRACKER)
            .arg("--api-sock")
            .arg(API_SOCK)
            .spawn()
            .unwrap();

        let mut agent = SocketAgent::new(API_SOCK, Duration::from_secs(3))
            .await
            .unwrap();

        agent.send_request(DATA.as_bytes()).await.unwrap();
        let response = agent.recv_response().await.unwrap();

        let body = GetFirecrackerVersion::decode(&response).unwrap();

        println!("{:?}", body);

        child.kill().unwrap();

        let _ = fs::remove_file(API_SOCK);
    }

    #[tokio::test]
    async fn test_get_firecracker_version_event() {
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-tokio-version-event.socket";

        let _ = fs::remove_file(API_SOCK);

        let mut child = Command::new(&*FIRECRACKER)
            .arg("--api-sock")
            .arg(API_SOCK)
            .spawn()
            .unwrap();

        let mut agent = SocketAgent::new(API_SOCK, Duration::from_secs(3))
            .await
            .unwrap();

        let response = agent.event(GetFirecrackerVersion(&Empty)).await.unwrap();

        println!("{:?}", response);

        child.kill().unwrap();

        let _ = fs::remove_file(API_SOCK);
    }
}
