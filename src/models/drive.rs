use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::rate_limiter::RateLimiter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Drive {
    /// drive id
    /// Required: true
    #[serde(rename = "drive_id")]
    pub drive_id: String,

    /// partuuid
    /// Represents the unique id of the boot partition of this device.
    /// It is optional and it will be taken into account
    /// only if the is_root_device field is true.
    #[serde(rename = "partuuid", skip_serializing_if = "Option::is_none")]
    pub partuuid: Option<String>,

    /// is root device
    /// Required: true
    #[serde(rename = "is_root_device")]
    pub is_root_device: bool,

    /// cache type
    /// Represents the caching strategy for the block device.
    #[serde(rename = "cache_type", skip_serializing_if = "Option::is_none")]
    pub cache_type: Option<CacheType>,

    /// VirtioBlock specific parameters:
    /// Is block read only.
    /// This field is required for virtio-block config and should be omitted for vhost-user-block configuration.
    /// Required: true
    #[serde(rename = "is_read_only")]
    pub is_read_only: bool,

    /// VirtioBlock specific parameters:
    /// Host level path for the guest drive.
    /// This field is required for virtio-block config and should be omitted for vhost-user-block configuration.
    /// Required: true
    #[serde(rename = "path_on_host")]
    pub path_on_host: PathBuf,

    /// VirtioBlock specific parameters:
    /// rate limiter
    #[serde(rename = "rate_limiter", skip_serializing_if = "Option::is_none")]
    pub rate_limiter: Option<RateLimiter>,

    /// VirtioBlock specific parameters:
    /// Type of the IO engine used by the device. "Async" is supported on
    /// host kernels newer than 5.10.51.
    /// This field is optional for virtio-block config and should be omitted for vhost-user-block configuration.
    #[serde(rename = "io_engine", skip_serializing_if = "Option::is_none")]
    pub io_engine: Option<IoEngine>,

    /// VhostUserBlock specific parameters
    /// Path to the socket of vhost-user-block backend.
    /// This field is required for vhost-user-block config should be omitted for virtio-block configuration.
    #[serde(rename = "socket", skip_serializing_if = "Option::is_none")]
    pub socket: Option<PathBuf>,
}

/// Block device caching strategies, default to "Unsafe".
/// Firecracker offers the possiblity of choosing the block device caching strategy.
/// Caching strategy affects the path data written from inside the microVM takes to the host persistent storage.
///
/// The caching strategy should be used in order to make a trade-off:
/// + Unsafe
/// + + enhances performance as fewer syscalls and IO operations are performed when running workloads
/// + + sacrifices data integrity in situations where the host simply loses the contents of the page cache without committing them to the backing storage (such as a power outage)
/// + + recommended for use cases with ephemeral storage, such as serverless environments
/// + Writeback
/// + + ensures that once a flush request was acknowledged by the host, the data is committed to the backing storage
/// + + sacrifices performance, from boot time increases to greater emulation-related latencies when running workloads
/// + + recommended for use cases with low power environments, such as embedded environments
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
pub enum CacheType {
    #[default]
    #[serde(rename = "Unsafe")]
    Unsafe,
    #[serde(rename = "WriteBack")]
    WriteBack,
}

/// Block device IO engine, default to "Sync".
/// The Async engine leverages io_uring for executing requests in an async manner,
/// therefore getting overall higher throughput by taking better advantage of the block device hardware, which typically supports queue depths greater than 1.
/// The block IO engine is configured via the PUT /drives API call (pre-boot only), with the io_engine field taking two possible values:
/// + Sync (default)
/// + Async (in developer preview)
/// The Sync variant is the default, in order to provide backwards compatibility with older Firecracker versions.
/// Note vhost-user block device is another option for block IO that requires an external backend process.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IoEngine {
    #[serde(rename = "Sync")]
    Sync,
    /// Firecracker requires a minimum host kernel version of 5.10.51 for the Async IO engine.
    /// This requirement is based on the availability of the io_uring subsystem, as well as a
    /// couple of features and bugfixes that were added in newer kernel versions.
    /// If a block device is configured with the Async io_engine on a host kernel older than
    /// 5.10.51, the API call will return a 400 Bad Request, with a suggestive error message.
    #[serde(rename = "Async")]
    Async,
}
