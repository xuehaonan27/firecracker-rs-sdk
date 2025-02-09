use std::{
    io::{ErrorKind, Read, Write},
    os::unix::net::UnixStream,
    path::Path,
    time::{Duration, Instant},
};

use crate::{
    events::{EventTrait, ResponseTrait},
    Error, Result,
};

use super::{SocketAgent, MAX_BUFFER_SIZE};

impl SocketAgent {
    pub(crate) fn new<P: AsRef<Path>>(socket_path: P, timeout: Duration) -> Result<Self> {
        let start = Instant::now();

        loop {
            match UnixStream::connect(socket_path.as_ref()) {
                Ok(stream) => {
                    stream.set_nonblocking(true)?;
                    return Ok(Self { stream });
                }
                Err(e)
                    if e.kind() == ErrorKind::NotFound
                        || e.kind() == ErrorKind::ConnectionRefused =>
                {
                    if start.elapsed() >= timeout {
                        return Err(Error::Agent(format!("Connection timed out: {e}")));
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

#[cfg(feature = "_rt-std")]
#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        io::{Read, Write},
        os::unix::net::UnixListener,
        path::Path,
        process::Command,
        sync::LazyLock,
        time::Duration,
    };

    use crate::{
        agent::MAX_BUFFER_SIZE,
        events::{GetFirecrackerVersion, ResponseTrait},
        models::Empty,
        Result,
    };

    use super::SocketAgent;

    const FIRECRACKER: LazyLock<String> = LazyLock::new(|| {
        dotenvy::dotenv().ok();
        env::var("FIRECRACKER").unwrap()
    });

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

        let mut child = Command::new(&*FIRECRACKER)
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

        let mut child = Command::new(&*FIRECRACKER)
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
