use std::{fs, time::Duration};

use crate::{
    agent::SocketAgent,
    check_agent_exists,
    events::*,
    events::{EventTrait, ResponseTrait},
    fstack::FStackAction,
    models::*,
    Error, Result,
};

use super::Instance;

impl Instance {
    pub async fn start_vmm(&mut self) -> Result<()> {
        // spawn vmm process
        let child = self.command.spawn()?;
        let pid = child.id();
        self.child = Some(child);

        // if we should remove jailer workspace directory after using / error
        // and there is a jailer workspace directory configuration (spawn by jailer)
        match (self.remove_jailer_workspace_dir, &self.jailer_workspace_dir) {
            (Some(true), Some(path)) => self
                .fstack
                .push_action(FStackAction::RemoveDirectory(path.clone())),
            _ => (),
        }

        // connect socket
        println!("start_vmm connecting to {}", self.socket_on_host.display());
        let socket_agent = SocketAgent::new(&self.socket_on_host, Duration::from_secs(3)).await?;
        self.agent = Some(socket_agent);
        self.fstack
            .push_action(FStackAction::RemoveFile(self.socket_on_host.clone()));

        // get pids
        if let Some(ref root) = self.jailer_workspace_dir {
            // using jailer
            let pid_file = root.join(format!("{}.pid", self.exec_file_name.display()));
            // unwrap safe (1): if there's not pid file, there would not be socket too, then method should have returned because of connection failure.
            // unwrap safe (2): we should trust `jailer` that the pid file should be sound.
            let firecracker_pid = fs::read_to_string(pid_file)
                .unwrap()
                .parse::<u32>()
                .unwrap();
            self.jailer_pid = Some(pid);
            self.firecracker_pid = Some(firecracker_pid);
        } else {
            // bare firecracker
            self.jailer_pid = None;
            self.firecracker_pid = Some(pid);
        }
        // unwrap safe: should be `Some(...)`
        self.fstack.push_action(FStackAction::TerminateProcess(
            self.firecracker_pid.unwrap(),
        ));

        Ok(())
    }

    /// Utility method for starting the instance.
    /// Wrapper around [`Instance::create_sync_action`] with parameter [`ActionType::InstanceStart`].
    pub async fn start(&mut self) -> Result<()> {
        let _ = self.create_sync_action(ActionType::InstanceStart).await?;
        Ok(())
    }

    /// Utility method for pausing the instance.
    /// Wrapper around [`Instance::patch_vm`] with parameter [`VmState::Paused`].
    pub async fn pause(&mut self) -> Result<()> {
        let _ = self
            .patch_vm(&Vm {
                state: VmState::Paused,
            })
            .await?;
        Ok(())
    }

    /// Utility method for pausing the instance.
    /// Wrapper around [`Instance::patch_vm`] with parameter [`VmState::Resumed`].
    pub async fn resume(&mut self) -> Result<()> {
        let _ = self
            .patch_vm(&Vm {
                state: VmState::Resumed,
            })
            .await?;
        Ok(())
    }

    /// Utility method for stopping the instance.
    /// Wrapper around [`Instance::create_sync_action`] with parameter [`ActionType::SendCtrlAtlDel`].
    pub async fn stop(&mut self) -> Result<()> {
        let _ = self.create_sync_action(ActionType::SendCtrlAtlDel).await?;
        Ok(())
    }

    /// Wrapper around [`SocketAgent::event`].
    /// Usually you should not invoke this method manully because other methods
    /// have already covered whatever available manipulation of `firecracker` while
    /// handling messy details for you such as hard link devices, files into the
    /// jailer directory (if jailer is used)
    pub async fn event<E: EventTrait>(
        &mut self,
        event: E,
    ) -> Result<<E as ResponseTrait>::Payload> {
        let agent = check_agent_exists!(self);
        agent.event(event).await
    }

    pub async fn describe_instance(&mut self) -> Result<InstanceInfo> {
        let agent = check_agent_exists!(self);
        agent.event(DescribeInstance(&Empty)).await
    }

    pub async fn create_sync_action(&mut self, action_type: ActionType) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent
            .event(CreateSyncAction(&InstanceActionInfo { action_type }))
            .await
    }

    pub async fn describe_balloon_config(&mut self) -> Result<Balloon> {
        let agent = check_agent_exists!(self);
        agent.event(DescribeBalloonConfig(&Empty)).await
    }

    pub async fn put_balloon(&mut self, balloon: &Balloon) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent.event(PutBalloon(balloon)).await
    }

    pub async fn patch_balloon(&mut self, balloon_update: &BalloonUpdate) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent.event(PatchBalloon(balloon_update)).await
    }

    pub async fn describe_balloon_stats(&mut self) -> Result<BalloonStats> {
        let agent = check_agent_exists!(self);
        agent.event(DescribeBalloonStats(&Empty)).await
    }

    pub async fn patch_balloon_stats_interval(
        &mut self,
        balloon_stats_update: &BalloonStatsUpdate,
    ) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent
            .event(PatchBalloonStatsInterval(balloon_stats_update))
            .await
    }

    pub async fn put_guest_boot_source(&mut self, boot_source: &BootSource) -> Result<Empty> {
        let agent = check_agent_exists!(self);

        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                // link the file
                let chroot_initrd_path = if let Some(ref path) = boot_source.initrd_path {
                    Some(chroot_strategy.link_file(jailer_workspace_dir, path)?
                    .strip_prefix(jailer_workspace_dir)
                    .and_then(|x| Ok(x.to_path_buf()))
                    .map_err(|_| {
                        Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                    })?)
                } else {
                    None
                };

                let chroot_kernel_image_path = chroot_strategy
                    .link_file(jailer_workspace_dir, &boot_source.kernel_image_path)?
                    .strip_prefix(jailer_workspace_dir)
                    .and_then(|x| Ok(x.to_path_buf()))
                    .map_err(|_| {
                        Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                    })?;

                let boot_source = &BootSource {
                    boot_args: boot_source.boot_args.clone(),
                    initrd_path: chroot_initrd_path,
                    kernel_image_path: chroot_kernel_image_path,
                };

                agent.event(PutGuestBootSource(boot_source)).await
            }
            _ => agent.event(PutGuestBootSource(boot_source)).await,
        }
    }

    pub async fn put_cpu_configuration(&mut self, cpu_config: &CPUConfig) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent.event(PutCpuConfiguration(cpu_config)).await
    }

    pub async fn put_guest_drive_by_id(&mut self, drive: &Drive) -> Result<Empty> {
        let agent = check_agent_exists!(self);

        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                let chroot_drive_path = chroot_strategy
                    .link_file(jailer_workspace_dir, &drive.path_on_host)?
                    .strip_prefix(jailer_workspace_dir)
                    .and_then(|x| Ok(x.to_path_buf()))
                    .map_err(|_| {
                        Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                    })?;

                let drive = Drive {
                    path_on_host: chroot_drive_path,
                    ..drive.clone()
                };

                agent.event(PutGuestDriveByID(&drive)).await
            }
            _ => agent.event(PutGuestDriveByID(drive)).await,
        }
    }

    pub async fn patch_guest_drive_by_id(&mut self, partial_drive: &PartialDrive) -> Result<Empty> {
        let agent = check_agent_exists!(self);

        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                if let Some(ref path) = partial_drive.path_on_host {
                    let chroot_drive_path = chroot_strategy
                        .link_file(jailer_workspace_dir, path)?
                        .strip_prefix(jailer_workspace_dir)
                        .and_then(|x| Ok(x.to_path_buf()))
                        .map_err(|_| {
                            Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                        })?;
                    let partial_drive = PartialDrive {
                        path_on_host: Some(chroot_drive_path),
                        ..partial_drive.clone()
                    };
                    agent.event(PatchGuestDriveByID(&partial_drive)).await
                } else {
                    agent.event(PatchGuestDriveByID(partial_drive)).await
                }
            }
            _ => agent.event(PatchGuestDriveByID(partial_drive)).await,
        }
    }

    pub async fn put_logger(&mut self, logger: &Logger) -> Result<Empty> {
        let agent = check_agent_exists!(self);

        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                let chroot_log_path = chroot_strategy
                    .link_file(jailer_workspace_dir, &logger.log_path)?
                    .strip_prefix(jailer_workspace_dir)
                    .and_then(|x| Ok(x.to_path_buf()))
                    .map_err(|_| {
                        Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                    })?;

                let logger = Logger {
                    log_path: chroot_log_path,
                    ..logger.clone()
                };

                agent.event(PutLogger(&logger)).await
            }
            _ => agent.event(PutLogger(logger)).await,
        }
    }

    pub async fn get_machine_configuration(&mut self) -> Result<MachineConfiguration> {
        let agent = check_agent_exists!(self);
        agent.event(GetMachineConfiguration(&Empty)).await
    }

    pub async fn put_machine_configuration(
        &mut self,
        machine_configuration: &MachineConfiguration,
    ) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent
            .event(PutMachineConfiguration(&machine_configuration))
            .await
    }

    pub async fn patch_machine_configuration(
        &mut self,
        machine_configuration: &MachineConfiguration,
    ) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent
            .event(PatchMachineConfiguration(machine_configuration))
            .await
    }

    pub async fn put_metrics(&mut self, metrics: &Metrics) -> Result<Empty> {
        let agent = check_agent_exists!(self);

        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                let chroot_metrics_path = chroot_strategy
                    .link_file(jailer_workspace_dir, &metrics.metrics_path)?
                    .strip_prefix(jailer_workspace_dir)
                    .and_then(|x| Ok(x.to_path_buf()))
                    .map_err(|_| {
                        Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                    })?;

                let metrics = Metrics {
                    metrics_path: chroot_metrics_path,
                };

                agent.event(PutMetrics(&metrics)).await
            }
            _ => agent.event(PutMetrics(metrics)).await,
        }
    }

    pub async fn put_mmds(&mut self, content: &MmdsContentsObject) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent.event(PutMmds(content)).await
    }

    pub async fn patch_mmds(&mut self, content: &MmdsContentsObject) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent.event(PatchMmds(content)).await
    }

    pub async fn get_mmds(&mut self) -> Result<MmdsContentsObject> {
        let agent = check_agent_exists!(self);
        agent.event(GetMmds(&Empty)).await
    }

    pub async fn put_mmds_config(&mut self, mmds_config: &MmdsConfig) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent.event(PutMmdsConfig(mmds_config)).await
    }

    pub async fn put_entropy_device(&mut self, entropy_device: &EntropyDevice) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent.event(PutEntropyDevice(entropy_device)).await
    }

    pub async fn put_guest_network_interface_by_id(
        &mut self,
        network_interface: &NetworkInterface,
    ) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent
            .event(PutGuestNetworkInterfaceByID(network_interface))
            .await
    }

    pub async fn patch_guest_network_interface_by_id(
        &mut self,
        partial_network_interface: &PartialNetworkInterface,
    ) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent
            .event(PatchGuestNetworkInterfaceByID(partial_network_interface))
            .await
    }

    pub async fn create_snapshot(
        &mut self,
        snapshot_create_params: &SnapshotCreateParams,
    ) -> Result<Empty> {
        let agent = check_agent_exists!(self);

        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                let chroot_mem_file_path = chroot_strategy
                    .chroot_path(jailer_workspace_dir, &snapshot_create_params.mem_file_path)?;

                let chroot_snapshot_path = chroot_strategy
                    .chroot_path(jailer_workspace_dir, &snapshot_create_params.snapshot_path)?;

                let chroot_snapshot_create_params = SnapshotCreateParams {
                    mem_file_path: chroot_mem_file_path.clone(),
                    snapshot_path: chroot_snapshot_path.clone(),
                    ..snapshot_create_params.clone()
                };

                let res = agent
                    .event(CreateSnapshot(&chroot_snapshot_create_params))
                    .await;

                chroot_strategy
                    .perform_link(chroot_mem_file_path, &snapshot_create_params.mem_file_path)?;
                chroot_strategy
                    .perform_link(chroot_snapshot_path, &snapshot_create_params.snapshot_path)?;

                res
            }
            _ => agent.event(CreateSnapshot(snapshot_create_params)).await,
        }
    }

    pub async fn load_snapshot(
        &mut self,
        snapshot_load_params: &SnapshotLoadParams,
    ) -> Result<Empty> {
        let agent = check_agent_exists!(self);

        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                let chroot_mem_file_path = if let Some(ref path) =
                    snapshot_load_params.mem_file_path
                {
                    let x = chroot_strategy
                    .link_file(jailer_workspace_dir, path)?
                    .strip_prefix(jailer_workspace_dir)
                    .and_then(|x| Ok(x.to_path_buf()))
                    .map_err(|_| {
                        Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                    })?;
                    Some(x)
                } else {
                    None
                };

                let chroot_snapshot_path = chroot_strategy
                .link_file(jailer_workspace_dir, &snapshot_load_params.snapshot_path)?
                .strip_prefix(jailer_workspace_dir)
                .and_then(|x| Ok(x.to_path_buf()))
                .map_err(|_| {
                    Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                })?;

                let snapshot_load_params = SnapshotLoadParams {
                    mem_file_path: chroot_mem_file_path,
                    snapshot_path: chroot_snapshot_path,
                    ..snapshot_load_params.clone()
                };

                agent.event(LoadSnapshot(&snapshot_load_params)).await
            }
            _ => agent.event(LoadSnapshot(snapshot_load_params)).await,
        }
    }

    pub async fn get_firecracker_version(&mut self) -> Result<FirecrackerVersion> {
        let agent = check_agent_exists!(self);
        agent.event(GetFirecrackerVersion(&Empty)).await
    }

    pub async fn patch_vm(&mut self, vm: &Vm) -> Result<Empty> {
        let agent = check_agent_exists!(self);
        agent.event(PatchVm(vm)).await
    }

    pub async fn get_export_vm_config(&mut self) -> Result<FullVmConfiguration> {
        let agent = check_agent_exists!(self);
        agent.event(GetExportVmConfig(&Empty)).await
    }

    pub async fn put_guest_vsock(&mut self, vsock: &Vsock) -> Result<Empty> {
        let agent = check_agent_exists!(self);

        match (&self.chroot_strategy, &self.jailer_workspace_dir) {
            (Some(chroot_strategy), Some(jailer_workspace_dir)) => {
                let chroot_uds_path = chroot_strategy
                .link_file(jailer_workspace_dir, &vsock.uds_path)?
                .strip_prefix(jailer_workspace_dir)
                .and_then(|x| Ok(x.to_path_buf()))
                .map_err(|_| {
                    Error::Instance("Fail to strip prefix `jailer_workspace_dir`, the chroot strategy should always link the file under `jailer_workspace_dir`!".into())
                })?;

                let vsock = Vsock {
                    uds_path: chroot_uds_path,
                    ..vsock.clone()
                };

                agent.event(PutGuestVsock(&vsock)).await
            }
            _ => agent.event(PutGuestVsock(vsock)).await,
        }
    }
}
