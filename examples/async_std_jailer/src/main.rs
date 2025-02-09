//! Run firecracker instance with jailer and async-std runtime

use firecracker_rs_sdk::firecracker::FirecrackerOption;
use firecracker_rs_sdk::Result;
use firecracker_rs_sdk::{jailer::JailerOption, models::*};

#[async_std::main]
async fn main() -> Result<()> {
    // Path to the `firecracker` binary
    const FIRECRACKER: &'static str = "/usr/bin/firecracker";

    // Path to the `jailer` binary
    const JAILER: &'static str = "/usr/bin/jailer";

    // Path at which you want to place the socket at
    const API_SOCK: &'static str = "/tmp/firecracker.socket";

    // Path to the kernel image
    const KERNEL: &'static str = "/foo/bar/vmlinux.bin";

    // Path to the rootfs
    const ROOTFS: &'static str = "/foo/bar/rootfs.ext4";

    // Build an instance with desired options
    let mut instance = JailerOption::new(JAILER, FIRECRACKER, "test-instance", 100, 123)
        .remove_jailer_workspace_dir() // remove jailer workspace directory after instance is dropped
        .firecracker_option(Some(FirecrackerOption::new(FIRECRACKER).api_sock(API_SOCK)))
        .build()?;

    // First start the `jailer` process and then `firecracker` process is spawned by `jailer`.
    instance.start_vmm().await?;

    // Try to get firecracker version as sanity checking
    let version = instance.get_firecracker_version().await?;
    println!("{:?}", version);

    // Then put some configuration to it
    // (1) Machine Configuration
    instance
        .put_machine_configuration(&MachineConfiguration {
            cpu_template: None,
            smt: None,
            mem_size_mib: 1024,
            track_dirty_pages: None,
            vcpu_count: 1,
            huge_pages: None,
        })
        .await?;

    // (2) Guest Boot Source
    instance
        .put_guest_boot_source(&BootSource {
            boot_args: Some("console=ttyS0 reboot=k panic=1 pci=off".into()),
            initrd_path: None,
            kernel_image_path: KERNEL.into(),
        })
        .await?;

    // (3) Guest Drives
    instance
        .put_guest_drive_by_id(&Drive {
            drive_id: "rootfs".into(),
            partuuid: None,
            is_root_device: true,
            cache_type: None,
            is_read_only: false,
            path_on_host: ROOTFS.into(),
            rate_limiter: None,
            io_engine: None,
            socket: None,
        })
        .await?;

    // Get jailer and firecracker pids
    let jailer_pid = instance.jailer_pid().unwrap();
    let firecracker_pid = instance.firecracker_pid().unwrap();
    println!(
        "jailer_pid = {}, firecracker_pid = {}",
        jailer_pid, firecracker_pid
    );

    // Start the instance
    instance.start().await?;
    async_std::task::sleep(std::time::Duration::from_secs(3)).await;

    // Pause the instance
    instance.pause().await?;
    async_std::task::sleep(std::time::Duration::from_secs(1)).await;

    // Resume the instance
    instance.resume().await?;
    async_std::task::sleep(std::time::Duration::from_secs(3)).await;

    // Stop the instance
    instance.stop().await?;

    // Jailer workspace directory is removed by [`Instance::drop`] automatically.
    // Turn off `remove_jailer_workspace_dir` to disable this behavior.

    Ok(())
}
