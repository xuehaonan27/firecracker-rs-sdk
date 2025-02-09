#[cfg(not(any(feature = "_rt-std", feature = "_rt-tokio", feature = "_rt-async-std")))]
use std::{path::Path, time::Duration};

#[cfg(feature = "_rt-async-std")]
mod rt_async_std;
#[cfg(feature = "_rt-std")]
mod rt_std;
#[cfg(feature = "_rt-tokio")]
mod rt_tokio;

pub const MAX_BUFFER_SIZE: usize = 64;

pub(crate) struct SocketAgent {
    #[cfg(feature = "_rt-std")]
    stream: std::os::unix::net::UnixStream,
    #[cfg(feature = "_rt-tokio")]
    stream: tokio::net::UnixStream,
    #[cfg(feature = "_rt-async-std")]
    stream: async_std::os::unix::net::UnixStream,
}

#[cfg(not(any(feature = "_rt-std", feature = "_rt-tokio", feature = "_rt-async-std")))]
impl SocketAgent {
    #[allow(unused)]
    pub(crate) fn new<P: AsRef<Path>>(_socket_path: P, _timeout: Duration) -> crate::Result<Self> {
        crate::missing_rt!()
    }
}
