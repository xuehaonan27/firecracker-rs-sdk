use serde::{Deserialize, Serialize};

use super::token_bucket;
/// RateLimiter Defines an IO rate limiter with independent bytes/s and ops/s limits.
/// Limits are defined by configuring each of the _bandwidth_ and _ops_ token buckets.
/// This field is optional for virtio-block config and should be omitted for vhost-user-block configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RateLimiter {
    /// Token bucket with bytes as tokens
    #[serde(rename = "banwidth")]
    pub bandwidth: Option<token_bucket::TokenBucket>,
    /// Token bucket with operations as tokens
    #[serde(rename = "ops")]
    pub ops: Option<token_bucket::TokenBucket>,
}

impl RateLimiter {
    pub fn new(bandwidth: token_bucket::TokenBucket, ops: token_bucket::TokenBucket) -> Self {
        Self {
            bandwidth: Some(bandwidth),
            ops: Some(ops),
        }
    }
}

/// RateLimiterSet represents a pair of RateLimiters (inbound and outbound)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RateLimiterSet {
    /// InRateLimiter limits the incoming bytes.
    #[serde(rename = "in_rate_limiter", skip_serializing_if = "Option::is_none")]
    pub in_rate_limiter: Option<RateLimiter>,

    /// OutRateLimiter limits the outgoing bytes.
    #[serde(rename = "out_rate_limiter", skip_serializing_if = "Option::is_none")]
    pub out_rate_limiter: Option<RateLimiter>,
}
