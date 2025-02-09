use std::{env, sync::LazyLock};

pub(crate) const FIRECRACKER: LazyLock<String> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    env::var("FIRECRACKER").unwrap()
});

pub(crate) const JAILER: LazyLock<String> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    env::var("JAILER").unwrap()
});

pub(crate) const KERNEL: LazyLock<String> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    env::var("KERNEL").unwrap()
});

pub(crate) const ROOTFS: LazyLock<String> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    env::var("ROOTFS").unwrap()
});

fn load_envs() {
    dotenvy::dotenv().ok();
}
