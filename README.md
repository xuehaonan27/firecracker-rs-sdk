# <h1 align="center">Firecracker🧨 Rust🦀 SDK🚀</h1>

## 🔍 Overview

This crate provides a convenient and powerful Rust-based Software Development Kit (SDK) for interacting with Firecracker, a lightweight virtual machine monitor designed for running multiple secure, isolated virtual machines (VMs) on a single host. With this SDK, developers can easily start, manage, and control Firecracker instances using Rust code, abstracting away the complexities of the underlying Firecracker API and providing a more intuitive and Rustic programming experience.

## ⭐️ Key Capabilities

- **Instance Management**:
  - Start and stop Firecracker instances effortlessly. You can spawn a new virtual machine process and stop to gracefully shut it down.
  - Pause and resume instances as needed. The `pause` and `resume` methods provide a convenient way to control the execution state of the virtual machines.
- **Configuration Management**:
  - Configure various aspects of the Firecracker instances, such as CPU configuration, network interfaces, drives, and more. For example, you can use `put_cpu_configuration` to set the CPU parameters for an instance.
  - Retrieve information about the instance configuration, like getting the machine configuration using `get_machine_configuration`.
- **Event Handling**:
  - Interact with Firecracker events through the `event` method. This allows you to handle different events and perform actions based on them.
- **Unified Instance**:
  - Unified management of instances launched from `firecracker` and `jailer`.
- **Multiple Runtimes Supported**:
  - The crate supports `std`, `tokio` and `async-std` runtimes, which means you can
  write code with `tokio` or `async-std` asynchronous runtime or totally based on
  Rust standard library.

## 📋 Prerequisites

- **Rust**: Make sure you have Rust installed on your system. You can install Rust by following the official instructions on the [Rust website](https://www.rust-lang.org/tools/install).
- **Firecracker**: You need to have Firecracker installed and running on your target system. Refer to the [Firecracker documentation](https://github.com/firecracker-microvm/firecracker) for installation instructions.

## 🚀 Quick Start

1. **Add the Dependency**:
   Add the Firecracker Rust SDK to your `Cargo.toml` file:
   ```toml
   [dependencies]
   # Replace x.y.z with the actual version
   # Enable one of these feature flags:
   # `_rt-std`, `_rt-tokio`, `_rt-async-std`
   firecracker-rs-sdk = { version = "x.y.z", features = ["_rt-std"] }
   ```

2. **Create a Simple Example**:
   Here is a basic example of starting a Firecracker instance:
   ```rust
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
   ```

3. **Run the Code**:
   Run your Rust code using `cargo run`.

4. **More examples**:
   More examples are available under `examples` directory.

## 💻 Why `firecracker-rs-sdk`
### Launch Instances Without Manipulating with HTTP API

The SDK interacts with the Firecracker HTTP API under the hood. For example, when you call the `start` method, it sends an appropriate HTTP request to the Firecracker API to start the instance. You can use the various methods provided by the SDK to perform different operations on the Firecracker instance without having to deal with the low-level HTTP requests and responses directly.

Here is an example of using the SDK to configure a network interface:
```rust
use firecracker_rs_sdk::{Instance, NetworkInterface, Result};

fn main() -> Result<()> {
    let mut instance = Instance::new();
    let network_interface = NetworkInterface {
        // Configure network interface parameters here
        // ...
    };
    instance.put_guest_network_interface_by_id(&network_interface)?;
    Ok(())
}
```

Without the SDK, you might have to perform some HTTP communications via `curl` or other HTTP tools, and
handle all those complexities on your own! Why not just invoke a method!


### Committed to Shielding You from Complexity
You can configure how to run the Firecracker instance by setting the appropriate properties of the
`FirecrackerOption` or `JailerOption` struct and `build` it.
For example, if you want to remove the jailer workspace directory after dropping the `Instance`,
you can invoke `remove_jailer_workspace_dir` method of `JailerOption` and the crate will do this for you.

```rust
use firecracker_rs_sdk::jailer::JailerOption;

fn main() {
    let mut instance = JailerOption::new(JAILER, FIRECRACKER, "test-instance", 100, 123)
        .remove_jailer_workspace_dir() // remove jailer workspace directory after instance is dropped
        .firecracker_option(Some(FirecrackerOption::new(FIRECRACKER).api_sock(API_SOCK)))
        .build()?;
}
```

## 🔧 Feature flags
You must enable one (and only one) of the following three feature flags to use this library.

- `_rt-std`: When there's no asynchronous runtime (blocking IO but fits well into synchronous context)
- `_rt-tokio`: `tokio` runtime context
- `_rt-async-std`: `async-std` runtime context

## 📜 License
Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

## 🤝 Contributing

Welcome contributions!

If you find a bug or have a feature request, please open an issue on the [GitHub repository](https://github.com/your-repo/firecracker-rs-sdk). If you want to contribute code, please fork the repository, make your changes, and submit a pull request.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

## ❌ Former Crate Deprecated

It's worth noting that this library was transferred from [rustcracker](https://github.com/xuehaonan27/rustcracker) that has been deprecated. 

The original deprecated library had been a useful tool in the past, but due to various reasons such as lack of maintenance, compatibility issues with the latest Rust ecosystem and Firecracker versions, I decided to create this new library.

I've carefully migrated the core functional components, improved the code structure, and added new features to better serve the needs of developers working with Firecracker in Rust projects. With this SDK, developers can easily start, manage, and control Firecracker instances using Rust code, abstracting away the complexities of the underlying Firecracker API and providing a more intuitive and Rustic programming experience.
