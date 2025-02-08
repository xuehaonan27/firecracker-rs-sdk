use serde::{Deserialize, Serialize};

use super::rate_limiter::RateLimiter;

/// Defines an entropy device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntropyDevice {
    #[serde(rename = "rate_limiter")]
    pub rate_limiter: Option<RateLimiter>,
}
