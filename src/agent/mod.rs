#[cfg(feature = "_rt_async_std")]
use async_std::{
    io::{ReadExt, WriteExt},
    os::unix::net::UnixStream as AsyncUnixStream,
};
#[cfg(feature = "_rt_std")]
use std::{io::Read, io::Write, os::unix::net::UnixStream as StdUnixStream, time::Instant};
#[cfg(feature = "_rt_tokio")]
use tokio::{io::AsyncWriteExt, net::UnixStream as TokioUnixStream};
#[cfg(feature = "_rt_async")]
use std::fs;

use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use crate::events::{EventTrait, ResponseTrait};
use crate::{Error, Result};

const MAX_BUFFER_SIZE: usize = 64;

pub(crate) struct SocketAgent {
    #[cfg(feature = "_rt_std")]
    stream: StdUnixStream,
    #[cfg(feature = "_rt_tokio")]
    stream: TokioUnixStream,
    #[cfg(feature = "_rt_async_std")]
    stream: AsyncUnixStream,
}

#[cfg(feature = "_rt_std")]
impl SocketAgent {
    pub(crate) fn new<P: AsRef<Path>>(socket_path: P, timeout: Duration) -> Result<Self> {
        let start = Instant::now();

        loop {
            match StdUnixStream::connect(socket_path.as_ref()) {
                Ok(stream) => {
                    stream.set_nonblocking(true)?;
                    return Ok(Self { stream });
                }
                Err(e)
                    if e.kind() == ErrorKind::NotFound
                        || e.kind() == ErrorKind::ConnectionRefused =>
                {
                    if start.elapsed() >= timeout {
                        return Err(Error::Agent("Connection timed out".into()));
                    }
                    std::thread::sleep(Duration::from_millis(100)); // wait before retry
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub(crate) fn send_request(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data)?;
        self.stream.flush()?;
        Ok(())
    }

    pub(crate) fn recv_response(&mut self) -> Result<Vec<u8>> {
        let mut buf = [0u8; MAX_BUFFER_SIZE];
        let mut vec: Vec<u8> = Vec::new();

        loop {
            match self.stream.read(&mut buf) {
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

    pub(crate) fn event<E: EventTrait>(
        &mut self,
        event: E,
    ) -> Result<<E as ResponseTrait>::Payload> {
        self.send_request(&event.encode()?)?;
        let response = self.recv_response()?;
        E::decode(&response)
    }
}

#[cfg(feature = "_rt_tokio")]
impl SocketAgent {
    pub(crate) async fn new<P: AsRef<Path>>(socket_path: P, timeout: Duration) -> Result<Self> {
        // wait the socket
        let wait_future = async { while !fs::exists(&socket_path).is_ok_and(|x| x) {} };

        match tokio::time::timeout(timeout, wait_future).await {
            Ok(()) => {
                let stream = TokioUnixStream::connect(socket_path).await?;
                Ok(Self { stream })
            }
            Err(_) => Err(Error::Agent("Connection timed out".into())),
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
}

#[cfg(feature = "_rt_async_std")]
impl SocketAgent {
    pub(crate) async fn new<P: AsRef<Path>>(socket_path: P, timeout: Duration) -> Result<Self> {
        // wait the socket
        let wait_future = async { while !fs::exists(&socket_path).is_ok_and(|x| x) {} };

        match async_std::future::timeout(timeout, wait_future).await {
            Ok(()) => {
                let stream = AsyncUnixStream::connect(socket_path.as_ref().as_os_str()).await?;
                Ok(Self { stream })
            }
            Err(_) => Err(Error::Agent("Connection timed out".into())),
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
            match self.stream.read(&mut buf).await {
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
}

#[cfg(any(feature = "_rt_tokio", feature = "_rt_async_std"))]
impl SocketAgent {
    pub(crate) async fn event<E: EventTrait>(
        &mut self,
        event: E,
    ) -> Result<<E as ResponseTrait>::Payload> {
        self.send_request(&event.encode()?).await?;
        let response = self.recv_response().await?;
        E::decode(&response)
    }
}

#[cfg(feature = "_rt_std")]
#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        io::{Read, Write},
        os::unix::net::UnixListener,
        path::Path,
        process::Command,
        time::Duration,
    };

    use crate::{
        agent::MAX_BUFFER_SIZE,
        events::{GetFirecrackerVersion, ResponseTrait},
        models::Empty,
        Result,
    };

    use super::SocketAgent;

    fn echo_server<P: AsRef<Path>>(api_sock: P) -> Result<()> {
        let listener = UnixListener::bind(&api_sock)?;
        println!("Server listening on {}", api_sock.as_ref().display());

        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0; MAX_BUFFER_SIZE];
        let n = stream.read(&mut buffer)?;
        if n > 0 {
            stream.write_all(&buffer[0..n])?;
        }

        Ok(())
    }

    #[test]
    fn test_echo() {
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-std-echo.socket";
        const DATA: &'static str = "Hello, world!";
        while fs::exists(API_SOCK).unwrap() {
            let _ = fs::remove_file(API_SOCK);
        }

        let server_handle = std::thread::spawn(|| echo_server(API_SOCK));
        let mut agent = SocketAgent::new(API_SOCK, Duration::from_secs(3)).unwrap();
        agent.send_request(DATA.as_bytes()).unwrap();
        let response = agent.recv_response().unwrap();

        assert_eq!(&response[0..DATA.len()], DATA.as_bytes());

        server_handle.join().unwrap().unwrap();
        while fs::exists(API_SOCK).unwrap() {
            let _ = fs::remove_file(API_SOCK);
        }
    }

    #[test]
    fn test_get_firecracker_version() {
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-std-version.socket";
        const DATA: &'static str = "GET /version HTTP/1.0\r\n\r\n";

        while fs::exists(API_SOCK).unwrap() {
            let _ = fs::remove_file(API_SOCK);
        }

        let mut child = Command::new(env::var("FIRECRACKER").unwrap())
            .arg("--api-sock")
            .arg(API_SOCK)
            .spawn()
            .unwrap();

        let mut agent = SocketAgent::new(API_SOCK, Duration::from_secs(3)).unwrap();
        agent.send_request(DATA.as_bytes()).unwrap();
        let response = agent.recv_response().unwrap();

        let body = GetFirecrackerVersion::decode(&response).unwrap();

        println!("{:?}", body);

        child.kill().unwrap();

        while fs::exists(API_SOCK).unwrap() {
            let _ = fs::remove_file(API_SOCK);
        }
    }

    #[test]
    fn test_get_firecracker_version_event() {
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-std-version-event.socket";

        let _ = fs::remove_file(API_SOCK);

        let mut child = Command::new(env::var("FIRECRACKER").unwrap())
            .arg("--api-sock")
            .arg(API_SOCK)
            .spawn()
            .unwrap();

        let mut agent = SocketAgent::new(API_SOCK, Duration::from_secs(3)).unwrap();

        let response = agent.event(GetFirecrackerVersion(&Empty)).unwrap();

        println!("{:?}", response);

        child.kill().unwrap();

        let _ = fs::remove_file(API_SOCK);
    }
}

#[cfg(feature = "_rt_tokio")]
#[cfg(test)]
mod tests {
    use std::{env, fs, path::Path, process::Command, time::Duration};

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

        let mut child = Command::new(env::var("FIRECRACKER").unwrap())
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

        let mut child = Command::new(env::var("FIRECRACKER").unwrap())
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

#[cfg(feature = "_rt_async_std")]
#[cfg(test)]
mod tests {
    use std::{env, fs, process::Command, time::Duration};

    use async_std::{
        io::{ReadExt, WriteExt},
        os::unix::net::UnixListener,
        path::Path,
    };

    use crate::{
        agent::SocketAgent,
        events::{GetFirecrackerVersion, ResponseTrait},
        models::Empty,
        Result,
    };

    async fn echo_server<P: AsRef<Path>>(api_sock: P) -> Result<()> {
        let listener = UnixListener::bind(&api_sock).await?;
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

    #[async_std::test]
    async fn test_echo() {
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-async-std-echo.socket";
        const DATA: &'static str = "Hello, world!";
        let _ = fs::remove_file(API_SOCK);

        let server_handle = async_std::task::spawn(echo_server(API_SOCK));
        let mut agent = SocketAgent::new(API_SOCK, Duration::from_secs(3))
            .await
            .unwrap();
        agent.send_request(DATA.as_bytes()).await.unwrap();
        let response = agent.recv_response().await.unwrap();

        assert_eq!(&response[0..DATA.len()], DATA.as_bytes());

        server_handle.await.unwrap();
        let _ = fs::remove_file(API_SOCK);
    }

    #[async_std::test]
    async fn test_get_firecracker_version() {
        const DATA: &'static str = "GET /version HTTP/1.0\r\n\r\n";
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-async-std-version.socket";

        let _ = fs::remove_file(API_SOCK);

        let mut child = Command::new(env::var("FIRECRACKER").unwrap())
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

    #[async_std::test]
    async fn test_get_firecracker_version_event() {
        const API_SOCK: &'static str = "/tmp/firecracker-sdk-test-agent-async-std-version-event.socket";

        let _ = fs::remove_file(API_SOCK);

        let mut child = Command::new(env::var("FIRECRACKER").unwrap())
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
