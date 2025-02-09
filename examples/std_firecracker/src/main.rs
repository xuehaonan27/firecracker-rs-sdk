//! Run firecracker instance with std runtime

use firecracker_rs_sdk::firecracker::FirecrackerOption;
use firecracker_rs_sdk::models::*;
use firecracker_rs_sdk::Result;

fn main() -> Result<()> {
    // Path to the `firecracker` binary
    const FIRECRACKER: &'static str = "/usr/bin/firecracker";

    // Path at which you want to place the socket at
    const API_SOCK: &'static str = "/tmp/firecracker.socket";

    // Path to the kernel image
    const KERNEL: &'static str = "/foo/bar/vmlinux.bin";

    // Path to the rootfs
    const ROOTFS: &'static str = "/foo/bar/rootfs.ext4";

    // Build an instance with desired options
    let mut instance = FirecrackerOption::new(FIRECRACKER)
        .api_sock(API_SOCK)
        .id("test-instance")
        .build()?;

    // First start the `firecracker` process
    instance.start_vmm()?;

    // Try to get firecracker version as sanity checking
    let version = instance.get_firecracker_version()?;
    println!("{:?}", version);

    // Then put some configuration to it
    // (1) Machine Configuration
    instance.put_machine_configuration(&MachineConfiguration {
        cpu_template: None,
        smt: None,
        mem_size_mib: 1024,
        track_dirty_pages: None,
        vcpu_count: 1,
        huge_pages: None,
    })?;

    // (2) Guest Boot Source
    instance.put_guest_boot_source(&BootSource {
        boot_args: Some("console=ttyS0 reboot=k panic=1 pci=off".into()),
        initrd_path: None,
        kernel_image_path: KERNEL.into(),
    })?;

    // (3) Guest Drives
    instance.put_guest_drive_by_id(&Drive {
        drive_id: "rootfs".into(),
        partuuid: None,
        is_root_device: true,
        cache_type: None,
        is_read_only: false,
        path_on_host: ROOTFS.into(),
        rate_limiter: None,
        io_engine: None,
        socket: None,
    })?;

    // Start the instance
    instance.start()?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Pause the instance
    instance.pause()?;
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Resume the instance
    instance.resume()?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Stop the instance
    instance.stop()?;

    let _ = std::fs::remove_file(API_SOCK);

    Ok(())
}
