use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Describes the configuration option for the metrics capability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Metrics {
    /// Path to the named pipe or file where the JSON-formatted metrics are flushed.
    /// Required: true
    #[serde(rename = "metrics_path")]
    pub metrics_path: PathBuf,
}
