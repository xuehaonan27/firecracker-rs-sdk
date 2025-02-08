use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Boot source descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BootSource {
    /// Kernel boot arguments defines the command-line arguments
    /// that should be passed to the kernel.
    pub boot_args: Option<String>,

    /// Host level path to the initrd image used to boot the guest
    pub initrd_path: Option<PathBuf>,

    /// Host level path to the kernel image used to boot the guest
    /// The kernel image must be an uncompressed ELF image.
    /// Required: true
    pub kernel_image_path: PathBuf,
}
