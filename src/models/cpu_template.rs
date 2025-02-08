use serde::{Deserialize, Serialize};

/// The CPU Template defines a set of flags to be disabled from the microvm so that
/// the features exposed to the guest are the same as in the selected instance type.
/// This parameter has been deprecated and it will be removed in future Firecracker
/// release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CPUTemplate(
    /// default: "None"
    pub CPUTemplateString,
);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CPUTemplateString {
    #[serde(rename = "C3")]
    C3,
    #[serde(rename = "T2")]
    T2,
    #[serde(rename = "T2S")]
    T2S,
    #[serde(rename = "T2CL")]
    T2CL,
    #[serde(rename = "T2A")]
    T2A,
    #[serde(rename = "V1N1")]
    V1N1,
    #[serde(rename = "None")]
    None,
}

/// The CPU configuration template defines a set of bit maps as modifiers
/// of flags accessed by register to be disabled/enabled for the microvm.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CPUConfig {
    // A collection of CPUIDs to be modified. (x86_64)
    pub cpuid_modifiers: Vec<CpuIdModifier>,

    // A collection of model specific registers to be modified. (x86_64)
    pub msr_modifiers: Vec<MsrModifier>,

    // A collection of registers to be modified. (aarch64)
    pub reg_modifiers: Vec<RegModifier>,
}

/// CPUID modifiers. Only for x86_64.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CpuIdModifier {
    /// CPUID leaf index (or function). Must be a string containing an integer.
    /// Examples: ["0x1", "0x2"]
    pub leaf: String,

    /// CPUID subleaf index (or subfunction). Must be a string containing an integer.
    /// Examples: ["0x1", "0x2"]
    pub subleaf: String,

    /// KVM CPUID flags, see https://docs.kernel.org/virt/kvm/api.html#kvm-get-supported-cpuid
    pub flags: u32,

    /// CPUID register modifiers.
    pub modifiers: Vec<Modifiers>,
}

/// CPUID register modifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Modifiers {
    /// CPUID register name
    /// One of ["eax", "ebx", "ecx", "edx"]
    pub register: ModifierRegisterName,

    /// CPUID register value bitmap.
    /// Must be in format `0b[01x]{32}`.
    /// Corresponding bits will be cleared (`0`), set (`1`) or left intact (`x`). (`_`) can be used as a separator.
    /// Examples: ["0bxxxx000000000011xx00011011110010", "0bxxxxxxxxxxxxx0xx00xx00x0_0000_00xx"]
    pub bitmap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModifierRegisterName {
    #[serde(rename = "eax")]
    EAX,
    #[serde(rename = "ebx")]
    EBX,
    #[serde(rename = "ecx")]
    ECX,
    #[serde(rename = "edx")]
    EDX,
}

/// MSR modifiers. Only for x86_64.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MsrModifier {
    /// MSR address/identifier. Must be a string containing an integer.
    /// Example: ["0x10a"]
    pub addr: String,

    /// MSR value bitmap.
    /// Must be in format `0b[01x]{64}`.
    /// Corresponding bits will be cleared (`0`), set (`1`) or left intact (`x`). (`_`) can be used as a separator.
    /// Example: ["0bxxxx0000000000000000000000000000000000000000000000000000_11101011"]
    pub bitmap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegModifier {
    /// ARM register address/identifier. Must be a string containing an integer. See https://docs.kernel.org/virt/kvm/api.html#kvm-set-one-reg
    /// Example: ["0x603000000013c020"]
    pub addr: String,

    /// ARM register value bitmap. Must be in format `0b[01x]{1,128}`. The actual length of the bitmap should be less or equal to the size of the register in bits. Corresponding bits will be cleared (`0`), set (`1`) or left intact (`x`). (`_`) can be used as a separator.
    /// Example: ["0bxxxxxxxxxxxx_0000_xxxx_xxxx_xxxx_0000_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx_xxxx"]
    pub bitmap: String,
}
