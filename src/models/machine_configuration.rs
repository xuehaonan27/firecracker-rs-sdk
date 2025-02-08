use serde::{Deserialize, Serialize};

use super::cpu_template::CPUTemplate;

/// # Example
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MachineConfiguration {
    /// cpu template
    #[serde(rename = "cpu_template", skip_serializing_if = "Option::is_none")]
    pub cpu_template: Option<CPUTemplate>,

    /// Flag for enabling/disabling Hyperthreading
    /// Required: true
    #[serde(rename = "smt", skip_serializing_if = "Option::is_none")]
    pub ht_enabled: Option<bool>,

    /// Memory size of VM
    /// Required: true
    #[serde(rename = "mem_size_mib")]
    pub mem_size_mib: isize,

    /// Enable dirty page tracking.
    /// If this is enabled, then incremental guest memory snapshots can be created.
    /// These belong to diff snapshots, which contain, besides the microVM state, only the memory dirtied since
    /// a previous snapshot. Full snapshots each contain a full copy of the guest memory.
    #[serde(rename = "track_dirty_pages", skip_serializing_if = "Option::is_none")]
    pub track_dirty_pages: Option<bool>,

    /// Number of vCPUs (either 1 or an even number)
    /// Required: true
    /// Maximum: 32
    /// Minimum: 1
    #[serde(rename = "vcpu_count")]
    pub vcpu_count: isize,

    /// Which huge pages configuration (if any) should be used to back guest memory.
    /// enum:
    /// - None
    /// - 2M
    #[serde(rename = "huge_pages", skip_serializing_if = "Option::is_none")]
    pub huge_pages: Option<HugePageOption>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HugePageOption {
    #[serde(rename = "None")]
    None,
    #[serde(rename = "2M")]
    HugePage2M,
}
