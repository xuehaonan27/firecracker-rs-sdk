#![cfg(feature = "_rt-async-std")]

use std::fs;

use firecracker_rs_sdk::{firecracker::FirecrackerOption, jailer::JailerOption, Result};

mod common;

#[async_std::test]
async fn spawn_plain() -> Result<()> {
    const API_SOCK: &'static str = "/run/firecracker.socket";
    let firecracker_bin = &*common::FIRECRACKER;
    let jailer_bin = &*common::JAILER;

    let mut instance = JailerOption::new(
        jailer_bin,
        firecracker_bin,
        "integration-test-std-jailer-spawn-plain",
        100,
        123,
    )
    .remove_jailer_workspace_dir()
    .firecracker_option(Some(
        FirecrackerOption::new(firecracker_bin).api_sock(API_SOCK),
    ))
    .build()?;

    let _ = fs::remove_dir_all(instance.jailer_workspace_dir().unwrap());

    instance.start_vmm().await?;

    let version = instance.get_firecracker_version().await?;

    println!("{:?}", version);

    Ok(())
}

#[async_std::test]
async fn spawn_and_config() -> Result<()> {
    use firecracker_rs_sdk::models::*; // import all models for use

    const API_SOCK: &'static str = "/run/firecracker.socket";
    let firecracker_bin = &*common::FIRECRACKER;
    let jailer_bin = &*common::JAILER;

    let mut instance = JailerOption::new(
        jailer_bin,
        firecracker_bin,
        "integration-test-std-jailer-spawn-and-config",
        100,
        123,
    )
    .remove_jailer_workspace_dir()
    .firecracker_option(Some(
        FirecrackerOption::new(firecracker_bin).api_sock(API_SOCK),
    ))
    .build()?;

    let _ = fs::remove_dir_all(instance.jailer_workspace_dir().unwrap());

    instance.start_vmm().await?;

    // put some configuration to it
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

    let version = instance.get_firecracker_version().await?;

    println!("{:?}", version);

    Ok(())
}

#[async_std::test]
async fn basic_launch() -> Result<()> {
    use firecracker_rs_sdk::models::*; // import all models for use

    const API_SOCK: &'static str = "/run/firecracker.socket";
    let firecracker_bin = &*common::FIRECRACKER;
    let jailer_bin = &*common::JAILER;
    let kernel = &*common::KERNEL;
    let rootfs = &*common::ROOTFS;

    let mut instance = JailerOption::new(
        jailer_bin,
        firecracker_bin,
        "integration-test-std-jailer-basic-launch",
        100,
        123,
    )
    // .daemonize()
    .stdin("/dev/null")
    .stdout("/dev/null")
    .stderr("/dev/null")
    .remove_jailer_workspace_dir()
    .firecracker_option(Some(
        FirecrackerOption::new(firecracker_bin).api_sock(API_SOCK),
    ))
    .build()?;

    let _ = fs::remove_dir_all(instance.jailer_workspace_dir().unwrap());

    instance.start_vmm().await?;

    // put some configuration to it
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

    instance
        .put_guest_boot_source(&BootSource {
            boot_args: Some("console=ttyS0 reboot=k panic=1 pci=off".into()),
            initrd_path: None,
            kernel_image_path: kernel.into(),
        })
        .await?;

    instance
        .put_guest_drive_by_id(&Drive {
            drive_id: "rootfs".into(),
            partuuid: None,
            is_root_device: true,
            cache_type: None,
            is_read_only: false,
            path_on_host: rootfs.into(),
            rate_limiter: None,
            io_engine: None,
            socket: None,
        })
        .await?;

    let version = instance.get_firecracker_version().await?;
    println!("{:?}", version);

    let jailer_pid = instance.jailer_pid().unwrap();
    let firecracker_pid = instance.firecracker_pid().unwrap();
    println!(
        "jailer_pid = {}, firecracker_pid = {}",
        jailer_pid, firecracker_pid
    );

    instance.start().await?;
    async_std::task::sleep(std::time::Duration::from_secs(3)).await;

    instance.pause().await?;
    async_std::task::sleep(std::time::Duration::from_secs(1)).await;

    instance.resume().await?;
    async_std::task::sleep(std::time::Duration::from_secs(3)).await;

    instance.stop().await?;

    Ok(())
}
