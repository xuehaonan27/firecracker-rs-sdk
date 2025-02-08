use std::time::Duration;

use crate::{agent::SocketAgent, fstack::FStackAction, Result};

use super::Instance;

impl Instance {
    pub async fn start_vmm(&mut self) -> Result<()> {
        // spawn vmm process
        let child = self.command.spawn()?;
        let pid = child.id();
        self.child = Some(child);
        self.fstack.push_action(FStackAction::TerminateProcess(pid));

        // if we should remove jailer workspace directory after using / error
        // and there is a jailer workspace directory configuration (spawn by jailer)
        match (self.remove_jailer_workspace_dir, &self.jailer_workspace_dir) {
            (Some(true), Some(path)) => self
                .fstack
                .push_action(FStackAction::RemoveDirectory(path.clone())),
            _ => (),
        }

        // connect socket
        let socket_agent = SocketAgent::new(&self.socket_on_host, Duration::from_secs(3)).await?;
        self.agent = Some(socket_agent);

        Ok(())
    }
}
