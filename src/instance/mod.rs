#[cfg(any(feature = "_rt-std", feature = "_rt-async"))]
use std::{path::Path, process::Child};
use std::{path::PathBuf, process::Command};

use crate::jailer::ChrootStrategy;
#[cfg(any(feature = "_rt-std", feature = "_rt-async"))]
use crate::{agent::SocketAgent, fstack::FStack, Error, Result};

#[cfg(feature = "_rt-async")]
mod rt_async;
#[cfg(feature = "_rt-std")]
mod rt_std;

#[cfg(not(any(feature = "_rt-std", feature = "_rt-async")))]
pub struct Instance {}

#[cfg(not(any(feature = "_rt-std", feature = "_rt-async")))]
impl Instance {
    pub(crate) fn new(
        _socket_on_host: PathBuf,
        _jailer_workspace_dir: Option<PathBuf>,
        _chroot_strategy: Option<ChrootStrategy>,
        _remove_jailer_workspace_dir: Option<bool>,
        _command: Command,
        _exec_file_name: PathBuf,
    ) -> Self {
        crate::missing_rt_panic!()
    }
}

#[cfg(any(feature = "_rt-std", feature = "_rt-async"))]
pub struct Instance {
    socket_on_host: PathBuf,

    jailer_workspace_dir: Option<PathBuf>,

    chroot_strategy: Option<ChrootStrategy>,

    remove_jailer_workspace_dir: Option<bool>,

    command: Command,

    child: Option<Child>,

    agent: Option<SocketAgent>,

    fstack: FStack,

    exec_file_name: PathBuf,

    jailer_pid: Option<u32>,

    firecracker_pid: Option<u32>,
}

#[cfg(any(feature = "_rt-std", feature = "_rt-async"))]
impl Instance {
    pub(crate) fn new(
        socket_on_host: PathBuf,
        jailer_workspace_dir: Option<PathBuf>,
        chroot_strategy: Option<ChrootStrategy>,
        remove_jailer_workspace_dir: Option<bool>,
        command: Command,
        exec_file_name: PathBuf,
    ) -> Self {
        Self {
            socket_on_host,
            jailer_workspace_dir,
            chroot_strategy,
            remove_jailer_workspace_dir,
            command,
            child: None,
            agent: None,
            fstack: FStack::new(),
            exec_file_name,
            jailer_pid: None,
            firecracker_pid: None,
        }
    }

    /// Returns jailer workspace directory (i.e. <chroot_base>/exec_file_name/<id>/root/).
    ///
    /// Always returns [`None`] if the instance is not spawned with `jailer` (bare `firecracker`).
    pub fn jailer_workspace_dir(&self) -> Option<PathBuf> {
        self.jailer_workspace_dir.clone()
    }

    /// Check whether this instance would remove jailer workspace directory
    /// (i.e. <chroot_base>/exec_file_name/<id>/root/) when it's dropped.
    ///
    /// Always returns [`None`] if the instance is not spawned with `jailer` (bare `firecracker`).
    pub fn remove_jailer_workspace_dir(&self) -> Option<bool> {
        self.remove_jailer_workspace_dir
    }

    /// Returns `firecracker` PID of this instance.
    pub fn firecracker_pid(&self) -> Option<u32> {
        self.firecracker_pid
    }

    /// Returns `jailer` PID of this instance.
    /// Note that since `jailer` would exit as soon as it completes its job, the PID returned
    /// is usually without a corresponding running process.
    pub fn jailer_pid(&self) -> Option<u32> {
        self.jailer_pid
    }

    /// Returns the hard link inside the jailer corresponding to `path`.
    /// # Example
    /// ```rust,ignore,no_run
    /// // This is the file that would be seen by other processes
    /// let host_file = "/demo/foo/bar.txt";
    /// // Let's assume that the jailer workspace directory is
    /// let jailer_workspace_dir = "/srv/jailer/firecracker/test-instance/root";
    ///
    /// // Spawned with [`ChrootStrategy::NaiveLinkStrategy`]
    /// let instance_1: Instance;
    /// let jailed_link_1 = instance_1.jailed_link(host_file)?;
    ///
    /// // `host_file` and `jailed_link_1` are hard links pointing to the same INode.
    /// assert_eq!(
    ///     jailed_link_1,
    ///     "/srv/jailer/firecracker/test-instance/root/bar.txt".into();
    /// );
    ///
    /// // Spawned with [`ChrootStrategy::FullLinkStrategy`]
    /// let instance_2: Instance;
    /// let jailed_link_2 = instance_2.jailed_link(host_file)?;
    /// // `host_file` and `jailed_link_2` are hard links pointing to the same INode.
    /// assert_eq!(
    ///     jailed_link_2,
    ///     "/srv/jailer/firecracker/test-instance/root/demo/foo/bar.txt".into();
    /// );
    /// ```
    pub fn jailed_link<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                chroot_strategy.chroot_path(jailer_workspace_dir, path)
            }
            _ => Err(Error::Instance("Not using jailer".into())),
        }
    }
}

#[macro_export]
macro_rules! check_agent_exists {
    ($self:ident) => {{
        if $self.agent.is_none() {
            return Err(Error::Instance("No agent spawned".into()));
        }
        $self.agent.as_mut().unwrap()
    }};
}
