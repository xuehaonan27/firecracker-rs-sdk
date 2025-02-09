#![cfg(feature = "_rt-std")]

use std::fs;

use firecracker_sdk::{firecracker::FirecrackerOption, Result};

mod common;

#[test]
fn spawn_plain() -> Result<()> {
    const API_SOCK: &'static str =
        "/tmp/firecracker-sdk-integration-test-std-firecracker-spawn-plain.socket";
    let firecracker_bin = &*common::FIRECRACKER;

    let mut instance = FirecrackerOption::new(firecracker_bin)
        .api_sock(API_SOCK)
        .spawn()?;

    let _ = fs::remove_file(API_SOCK);
    instance.start_vmm()?;

    let version = instance.get_firecracker_version()?;

    println!("{:?}", version);
    fs::remove_file(API_SOCK)?;

    Ok(())
}

#[test]
fn spawn_and_config() -> Result<()> {
    use firecracker_sdk::models::*; // import all models for use

    const API_SOCK: &'static str =
        "/tmp/firecracker-sdk-integration-test-std-firecracker-spawn-and-config.socket";
    let firecracker_bin = &*common::FIRECRACKER;

    let mut instance = FirecrackerOption::new(firecracker_bin)
        .api_sock(API_SOCK)
        .spawn()?;

    let _ = fs::remove_file(API_SOCK);
    instance.start_vmm()?;

    // put some configuration to it
    instance.put_machine_configuration(&MachineConfiguration {
        cpu_template: None,
        ht_enabled: Some(true),
        mem_size_mib: 1024,
        track_dirty_pages: None,
        vcpu_count: 1,
        huge_pages: None,
    })?;

    let version = instance.get_firecracker_version()?;

    println!("{:?}", version);
    fs::remove_file(API_SOCK)?;

    Ok(())
}

#[test]
fn basic_launch() -> Result<()> {
    use firecracker_sdk::models::*; // import all models for use

    const API_SOCK: &'static str =
        "/tmp/firecracker-sdk-integration-test-std-firecracker-basic-launch.socket";
    let firecracker_bin = &*common::FIRECRACKER;
    let kernel = &*common::KERNEL;
    let rootfs = &*common::ROOTFS;

    let mut instance = FirecrackerOption::new(firecracker_bin)
        .api_sock(API_SOCK)
        .stdin("/dev/null")
        .stdout("/dev/null")
        .stderr("/dev/null")
        .spawn()?;

    instance.start_vmm()?;

    // put some configuration to it
    instance.put_machine_configuration(&MachineConfiguration {
        cpu_template: None,
        ht_enabled: None,
        mem_size_mib: 1024,
        track_dirty_pages: None,
        vcpu_count: 1,
        huge_pages: None,
    })?;

    instance.put_guest_boot_source(&BootSource {
        boot_args: Some("console=ttyS0 reboot=k panic=1 pci=off".into()),
        initrd_path: None,
        kernel_image_path: kernel.into(),
    })?;

    instance.put_guest_drive_by_id(&Drive {
        drive_id: "rootfs".into(),
        partuuid: None,
        is_root_device: true,
        cache_type: None,
        is_read_only: false,
        path_on_host: rootfs.into(),
        rate_limiter: None,
        io_engine: None,
        socket: None,
    })?;

    let version = instance.get_firecracker_version()?;
    println!("{:?}", version);

    instance.start()?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    instance.pause()?;
    std::thread::sleep(std::time::Duration::from_secs(1));

    instance.resume()?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    instance.stop()?;

    // fs::remove_dir_all(instance.jailer_workspace_dir().unwrap())?;
    fs::remove_file(API_SOCK)?;

    Ok(())
}
