use serde::{Deserialize, Serialize};

/// The current detailed state (Not started, Running, Paused) of the Firecracker instance.
/// This value is read-only for the control-plane.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum State {
    #[serde(rename = "Not started")]
    NotStarted,
    #[serde(rename = "Running")]
    Running,
    #[serde(rename = "Paused")]
    Paused,
}

/// Describes MicroVM instance information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceInfo {
    /// Application name.
    /// Required: true
    #[serde(rename = "app_name")]
    pub app_name: String,

    /// MicroVM / instance ID.
    /// Required: true
    #[serde(rename = "id")]
    pub id: String,

    /// The current detailed state (Not started, Running, Paused) of the Firecracker instance.
    /// This value is read-only for the control-plane.
    /// Required: true
    /// Enum: [Not started Running Paused]
    #[serde(rename = "state")]
    pub state: State,

    /// MicroVM hypervisor build version.
    /// Required: true
    #[serde(rename = "vmm_version")]
    pub vmm_version: String,
}
