use serde::{Deserialize, Serialize};

/// Balloon device descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Balloon {
    /// Target balloon size in MiB
    /// Required: true
    #[serde(rename = "amount_mib")]
    pub amount_mib: i64,

    /// Whether the balloon should deflate when then guest has memory pressure
    /// Required: true
    #[serde(rename = "deflate_on_oom")]
    pub deflate_on_oom: bool,

    /// Interval in seconds between refreshing statistics
    /// non-zero value will enable statistics
    /// Defaults to 0
    #[serde(
        rename = "stats_polling_interval_s",
        skip_serializing_if = "Option::is_none"
    )]
    pub stats_polling_interval_s: Option<i64>,
}
