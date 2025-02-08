use serde::{Deserialize, Serialize};

/// Describes the balloon device statistics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BalloonStatsUpdate {
    /// Interval in seconds between refreshing statistics.
    #[serde(rename = "stats_polling_interval_s")]
    pub stats_polling_interval_s: i64,
}
