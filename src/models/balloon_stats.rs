use serde::{Deserialize, Serialize};

/// Describes the balloon device statistics.
///
/// This structure represents the return value requested
/// by `GET /balloon/statistics`, which describes detailed
/// information of the balloon device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BalloonStats {
    /// Target number of pages the device aims to hold.
    /// Required: true
    #[serde(rename = "target_pages")]
    pub target_pages: u64,

    /// Actual number of pages the device is holding.
    /// Required: true
    #[serde(rename = "actual_pages")]
    pub actual_pages: u64,

    /// Target amount of memory (in MiB) the device aims to hold.
    /// Required: true
    #[serde(rename = "target_mib")]
    pub target_mib: u64,

    /// Actual amount of memory (in MiB) the device is holding.
    /// Required: true
    #[serde(rename = "actual_mib")]
    pub actual_mib: u64,

    /// The amount of memory that has been swapped in (in bytes).
    #[serde(rename = "swap_in", skip_serializing_if = "Option::is_none")]
    pub swap_in: Option<u64>,

    /// The amount of memory that has been swapped out to disk (in bytes).
    #[serde(rename = "swap_out", skip_serializing_if = "Option::is_none")]
    pub swap_out: Option<u64>,

    /// The number of major page faults that have occurred.
    #[serde(rename = "major_faults", skip_serializing_if = "Option::is_none")]
    pub major_faults: Option<u64>,

    /// The number of minor page faults that have occurred.
    #[serde(rename = "minor_faults", skip_serializing_if = "Option::is_none")]
    pub minor_faults: Option<u64>,

    /// The amount of memory not being used for any purpose (in bytes).
    #[serde(rename = "free_memory", skip_serializing_if = "Option::is_none")]
    pub free_memory: Option<u64>,

    /// The total amount of memory available (in bytes).
    #[serde(rename = "total_memory", skip_serializing_if = "Option::is_none")]
    pub total_memory: Option<u64>,

    /// An estimate of how much memory is available (in bytes) for starting new applications
    /// without pushing the system to swap;
    #[serde(rename = "available_memory", skip_serializing_if = "Option::is_none")]
    pub available_memory: Option<u64>,

    /// The amount of memory, in bytes, that can be quickly reclaimed without additional I/O.
    /// Typically these pages are used for caching files from disk.
    #[serde(rename = "disk_caches", skip_serializing_if = "Option::is_none")]
    pub disk_caches: Option<u64>,

    /// The number of successful hugetlb page allocations in the guest.
    #[serde(
        rename = "hugetlb_allocations",
        skip_serializing_if = "Option::is_none"
    )]
    pub hugetlb_allocations: Option<u64>,

    /// The number of failed hugetlb page allocations in the guest.
    #[serde(rename = "hugetlb_failures", skip_serializing_if = "Option::is_none")]
    pub hugetlb_failures: Option<u64>,
}
