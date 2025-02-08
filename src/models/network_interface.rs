use serde::{Deserialize, Serialize};

use super::rate_limiter;

/// Defines a network interface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetworkInterface {
    /// If this field is set, the device model will reply to
    /// HTTP GET requests sent to the MMDS address via this interface.
    /// In this case, both ARP requests for 169.254.169.254 and TCP
    /// segments heading to the same address are intercepted by the
    /// device model, and do not reach the associated TAP device.
    // pub allow_mmds_requests: Option<bool>,

    /// guest mac
    #[serde(rename = "guest_mac", skip_serializing_if = "Option::is_none")]
    pub guest_mac: Option<String>,

    /// Host level path for the guest network interface
    /// Required: true
    #[serde(rename = "host_dev_name")]
    pub host_dev_name: String,

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
