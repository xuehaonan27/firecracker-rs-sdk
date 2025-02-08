use serde::{Deserialize, Serialize};

/// TokenBucket Defines a token bucket with a maximum capacity (size),
/// an initial burst size (one_time_burst) and an interval for refilling purposes (refill_time).
/// The refill-rate is derived from size and refill_time,
/// and it is the constant rate at which the tokens replenish.
/// The refill process only starts happening after the initial burst budget is consumed.
/// Consumption from the token bucket is unbounded in speed which allows for bursts bound in size
/// by the amount of tokens available. Once the token bucket is empty, consumption speed is bound
/// by the refill_rate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenBucket {
    /// The initial size of a token bucket.
    /// Minimum: 0
    pub one_time_burst: Option<u64>,

    /// The amount of milliseconds it takes for the bucket to refill.
    /// Required: true
    /// Minimum: 0
    pub refill_time: u64,

    /// The total number of tokens this bucket can hold.
    /// Required: true
    /// Minimum: 0
    pub size: u64,
}
