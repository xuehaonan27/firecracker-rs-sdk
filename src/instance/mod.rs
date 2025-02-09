use std::{
    path::PathBuf,
    process::{Child, Command},
};

use crate::{agent::SocketAgent, fstack::FStack, jailer::ChrootStrategy};

#[cfg(feature = "_rt_async")]
mod rt_async;
#[cfg(feature = "_rt_std")]
mod rt_std;

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

    pub fn jailer_workspace_dir(&self) -> Option<PathBuf> {
        self.jailer_workspace_dir.clone()
    }

    pub fn remove_jailer_workspace_dir(&self) -> Option<bool> {
        self.remove_jailer_workspace_dir
    }

    pub fn firecracker_pid(&self) -> Option<u32> {
        self.firecracker_pid
    }

    pub fn jailer_pid(&self) -> Option<u32> {
        self.jailer_pid
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
