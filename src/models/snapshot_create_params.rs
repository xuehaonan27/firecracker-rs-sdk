use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotCreateParams {
    /// Path to the file that will contain the guest memory.
    /// Required: true
    #[serde(rename = "mem_file_path")]
    pub mem_file_path: PathBuf,

    /// Path to the file that will contain the microVM state.
    /// Required: true
    #[serde(rename = "snapshot_path")]
    pub snapshot_path: PathBuf,

    /// Type of snapshot to create. It is optional and by default, a full snapshot is created.
    /// Enum: [Full Diff]
    #[serde(rename = "snapshot_type", skip_serializing_if = "Option::is_none")]
    pub snapshot_type: Option<SnapshotType>,

    /// The microVM version for which we want to create the snapshot.
    /// It is optional and it defaults to the current version.
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum SnapshotType {
    #[default]
    #[serde(rename = "Full")]
    Full,
    #[serde(rename = "Diff")]
    Diff,
}
