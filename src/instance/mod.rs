use std::{
    path::PathBuf,
    process::{Child, Command},
    time::Duration,
};

use crate::{agent::SocketAgent, fstack::FStack, jailer::ChrootStrategy, Error, Result};

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
}

impl Instance {
    pub(crate) fn new(
        socket_on_host: PathBuf,
        jailer_workspace_dir: Option<PathBuf>,
        chroot_strategy: Option<ChrootStrategy>,
        remove_jailer_workspace_dir: Option<bool>,
        command: Command,
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
