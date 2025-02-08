use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::memory_backend;

/// Defines the configuration used for handling snapshot resume. Exactly one of
/// the two `mem_*` fields must be present in the body of the request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnapshotLoadParams {
    /// Enable support for incremental (diff) snapshots
    /// by tracking dirty guest pages.
    #[serde(
        rename = "enable_diff_snapshots",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_diff_snapshots: Option<bool>,

    /// Path to the file that contains the guest memory to be loaded.
    /// It is only allowed if `mem_backend` is not present. This parameter has
    /// been deprecated and it will be removed in future Firecracker release.
    /// Required: true
    #[serde(rename = "mem_file_path", skip_serializing_if = "Option::is_none")]
    pub mem_file_path: Option<PathBuf>,

    /// Configuration for the backend that handles memory load. If this field
    // is specified, `mem_file_path` is forbidden. Either `mem_backend` or
    // `mem_file_path` must be present at a time.
    #[serde(rename = "mem_backend", skip_serializing_if = "Option::is_none")]
    pub mem_backend: Option<memory_backend::MemoryBackend>,

    /// When set to true, the vm is also resumed
    /// if the snapshot load is successful.
    #[serde(rename = "resume_vm", skip_serializing_if = "Option::is_none")]
    pub resume_vm: Option<bool>,

    /// Path to the file that contains the microVM state to be loaded.
    /// Required: true
    #[serde(rename = "snapshot_path")]
    pub snapshot_path: PathBuf,
}
