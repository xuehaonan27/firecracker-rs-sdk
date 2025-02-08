use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/*
Vsock Defines a vsock device, backed by a set of Unix Domain Sockets, on the host side.

For host-initiated connections, Firecracker will be
listening on the Unix socket identified by the path `uds_path`.
Firecracker will create this socket, bind and listen on it.

Host-initiated connections will be performed by connection
to this socket and issuing a connection forwarding
request to the desired guest-side vsock port
(i.e. `CONNECT 52\n`, to connect to port 52).

For guest-initiated connections, Firecracker will expect host
software to be bound and listening on Unix sockets at `uds_path_<PORT>`.
E.g. "/path/to/host_vsock.sock_52" for port number 52.
*/

/// Defines a vsock device, backed by a set of Unix Domain Sockets, on the host side.
/// For host-initiated connections, Firecracker will be listening on the Unix socket
/// identified by the path `uds_path`. Firecracker will create this socket, bind and
/// listen on it. Host-initiated connections will be performed by connection to this
/// socket and issuing a connection forwarding request to the desired guest-side vsock
/// port (i.e. `CONNECT 52\n`, to connect to port 52).
/// For guest-initiated connections, Firecracker will expect host software to be
/// bound and listening on Unix sockets at `uds_path_<PORT>`.
/// E.g. "/path/to/host_vsock.sock_52" for port number 52.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vsock {
    /// Guest Vsock CID
    /// Required: true
    /// Minimum: 3
    /// CID defines the 32-bit Context Identifier for the vsock device.  See
    /// the vsock(7) manual page for more information.
    #[serde(rename = "guest_cid")]
    pub guest_cid: u32,

    /// Path to UNIX domain socket, used to proxy vsock connections.
    /// Required: true
    /// Path defines the filesystem path of the vsock device on the host.
    #[serde(rename = "uds_path")]
    pub uds_path: PathBuf,

    /// vsock id
    /// Required: true
    /// ID defines the vsock's device ID for firecracker.
    /// This parameter has been deprecated and it will be removed in future
    /// Firecracker release.
    #[serde(rename = "vsock_id", skip_serializing_if = "Option::is_none")]
    pub vsock_id: Option<String>,
}
