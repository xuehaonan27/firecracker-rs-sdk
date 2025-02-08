use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActionType {
    #[serde(rename = "FlushMetrics")]
    FlushMetrics,
    #[serde(rename = "InstanceStart")]
    InstanceStart,
    #[serde(rename = "SendCtrlAltDel")]
    SendCtrlAtlDel,
}

/// Variant wrapper containing the real action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceActionInfo {
    /// Enumeration indicating what type of action is contained in the payload
    /// Required: true
    /// Enum: [FlushMetrics InstanceStart SendCtrlAltDel
    #[serde(rename = "action_type")]
    pub action_type: ActionType,
}
