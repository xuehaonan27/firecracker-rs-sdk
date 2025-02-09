use std::{env, fs, sync::LazyLock};

use firecracker_sdk::{firecracker::FirecrackerOption, Result};

pub(crate) const FIRECRACKER: LazyLock<String> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    env::var("FIRECRACKER").unwrap()
});

fn main() -> Result<()> {
    const API_SOCK: &'static str =
        "/tmp/firecracker-sdk-integration-test-std-firecracker-spawn-plain.socket";
    let firecracker_bin = &*FIRECRACKER;

    // firecracker --api-sock <API_SOCK>
    let mut instance = FirecrackerOption::new(firecracker_bin)
        .api_sock(API_SOCK)
        .spawn()?;

    // remove possible existing <API_SOCK>
    let _ = fs::remove_file(API_SOCK);

    // start firecracker process
    instance.start_vmm()?;

    // send `getFirecrackerVersion` operation to the firecracker
    let version = instance.get_firecracker_version()?;

    println!("{:?}", version);

    Ok(())
}
