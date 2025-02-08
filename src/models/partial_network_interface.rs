use serde::{Deserialize, Serialize};

use super::rate_limiter;
/// PartialNetworkInterface Defines a partial network interface structure,
/// used to update the rate limiters for that interface, after microvm start.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartialNetworkInterface {
    /// iface id
    /// Required: true
    #[serde(rename = "iface_id")]
    pub iface_id: String,

    /// rx rate limiter
    #[serde(rename = "rx_rate_limiter", skip_serializing_if = "Option::is_none")]
    pub rx_rate_limiter: Option<rate_limiter::RateLimiter>,

    /// tx rate limiter
    #[serde(rename = "tx_rate_limiter", skip_serializing_if = "Option::is_none")]
    pub tx_rate_limiter: Option<rate_limiter::RateLimiter>,
}
