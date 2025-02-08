use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::rate_limiter;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartialDrive {
    /// drive id
    /// Required: true
    #[serde(rename = "drive_id")]
    pub drive_id: String,

    /// Host level path for the guest drive
    /// This field is optional for virtio-block config
    /// and should be omitted for vhost-user-block configuration.
    #[serde(rename = "path_on_host", skip_serializing_if = "Option::is_none")]
    pub path_on_host: Option<PathBuf>,

    /// rate limiter
    #[serde(rename = "rate_limiter", skip_serializing_if = "Option::is_none")]
    pub rate_limiter: Option<rate_limiter::RateLimiter>,
}
