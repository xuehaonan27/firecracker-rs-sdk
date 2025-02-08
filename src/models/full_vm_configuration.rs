use serde::{Deserialize, Serialize};

use super::*;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FullVmConfiguration {
    #[serde(rename = "balloon", skip_serializing_if = "Option::is_none")]
    pub balloon: Option<balloon::Balloon>,

    /// Configurations for all block devices.
    #[serde(rename = "drive", skip_serializing_if = "Option::is_none")]
    pub drives: Option<Vec<drive::Drive>>,

    #[serde(rename = "boot-source", skip_serializing_if = "Option::is_none")]
    pub boot_source: Option<boot_source::BootSource>,

    #[serde(rename = "logger", skip_serializing_if = "Option::is_none")]
    pub logger: Option<logger::Logger>,

    #[serde(rename = "machine-config", skip_serializing_if = "Option::is_none")]
    pub machine_config: Option<machine_configuration::MachineConfiguration>,

    #[serde(rename = "metrics", skip_serializing_if = "Option::is_none")]
    pub metrics: Option<metrics::Metrics>,

    #[serde(rename = "mmds-config", skip_serializing_if = "Option::is_none")]
    pub mmds_config: Option<mmds_config::MmdsConfig>,

    /// Configurations for all net devices.
    #[serde(rename = "network-interfaces", skip_serializing_if = "Option::is_none")]
    pub network_interfaces: Option<Vec<network_interface::NetworkInterface>>,

    #[serde(rename = "vsock", skip_serializing_if = "Option::is_none")]
    pub vsock: Option<vsock::Vsock>,
}
