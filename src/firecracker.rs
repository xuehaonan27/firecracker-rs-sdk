//! Option to launch firecracker

use std::{path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};

use crate::instance::Instance;

pub const DEFAULT_API_SOCK: &'static str = "/run/firecracker.socket";
pub const DEFAULT_HTTP_API_MAX_PAYLOAD_SIZE: usize = 51200;
pub const DEFAULT_ID: &'static str = "anonymous-instance";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FirecrackerOption {
    firecracker_bin: PathBuf,

    // Path to unix domain socket used by the API. [default: "/run/firecracker.socket"]
    pub(crate) api_sock: Option<PathBuf>,

    // Whether or not to load boot timer device for logging elapsed time since InstanceStart command.
    boot_timer: Option<bool>,

    // Path to a file that contains the microVM configuration in JSON format.
    config_file: Option<PathBuf>,

    // Print the data format version of the provided snapshot state file.
    describe_snapshot: Option<bool>,

    // Http API request payload max size, in bytes. [default: "51200"]
    http_api_max_payload_size: Option<usize>,

    // MicroVM unique identifier. [default: "anonymous-instance"]
    id: Option<String>,

    // Set the logger level.
    level: Option<String>,

    // Path to a fifo or a file used for configuring the logger on startup.
    log_path: Option<PathBuf>,

    // Path to a file that contains metadata in JSON format to add to the mmds.
    metadata: Option<PathBuf>,

    // Path to a fifo or a file used for configuring the metrics on startup.
    metrics_path: Option<PathBuf>,

    // Mmds data store limit, in bytes.
    mmds_size_limit: Option<PathBuf>,

    // Set the logger module filter.
    module: Option<String>,

    // Optional parameter which allows starting and using a microVM without an active API socket.
    no_api: Option<bool>,

    // Optional parameter which allows starting and using a microVM without seccomp filtering. Not recommended.
    no_seccomp: Option<bool>,

    // Parent process CPU time (wall clock, microseconds). This parameter is optional.
    parent_cpu_time_us: Option<usize>,

    // Optional parameter which allows specifying the path to a custom seccomp filter. For advanced users.
    seccomp_filter: Option<String>,

    // Whether or not to output the level in the logs.
    show_level: Option<bool>,

    // Whether or not to include the file path and line number of the log's origin.
    show_log_origin: Option<bool>,

    // Process start CPU time (wall clock, microseconds). This parameter is optional.
    start_time_cpu_us: Option<usize>,

    // Process start time (wall clock, microseconds). This parameter is optional.
    start_time_us: Option<usize>,
}

impl FirecrackerOption {
    pub fn new(firecracker_bin: PathBuf) -> Self {
        Self {
            firecracker_bin,
            ..Default::default()
        }
    }

    pub fn spawn(&mut self) -> Instance {
        // spawn instance directly with firecracker
        let command = self.build_cmd(None);

        let socket_on_host = self
            .api_sock
            .clone()
            .unwrap_or_else(|| DEFAULT_API_SOCK.into());

        Instance::new(socket_on_host, None, None, None, command)
    }

    pub(crate) fn build_cmd(&self, jailer_workspace_dir: Option<PathBuf>) -> Command {
        let mut cmd = Command::new(&self.firecracker_bin);

        let api_sock = match self.api_sock {
            Some(ref api_sock) => api_sock,
            None => &DEFAULT_API_SOCK.into(),
        };

        let api_sock = if let Some(ref jailer_workspace_dir) = jailer_workspace_dir {
            &jailer_workspace_dir.join(api_sock)
        } else {
            api_sock
        };
        cmd.arg("--api-sock").arg(api_sock);

        if let Some(true) = self.boot_timer {
            cmd.arg("--boot-timer");
        }

        if let Some(ref config_file) = self.config_file {
            cmd.arg("--config-file").arg(config_file);
        }

        if let Some(ref http_api_max_payload_size) = self.http_api_max_payload_size {
            cmd.arg("--http-api-max-payload-size")
                .arg(http_api_max_payload_size.to_string());
        }

        if let Some(ref id) = self.id {
            cmd.arg("--id").arg(id);
        }

        if let Some(ref level) = self.level {
            cmd.arg("--level").arg(level);
        }

        if let Some(ref log_path) = self.log_path {
            cmd.arg("--log-path").arg(log_path);
        }

        if let Some(ref metadata) = self.metadata {
            cmd.arg("--metadata").arg(metadata);
        }

        if let Some(ref metrics_path) = self.metrics_path {
            cmd.arg("--metrics-path").arg(metrics_path);
        }

        if let Some(ref mmds_size_limit) = self.mmds_size_limit {
            cmd.arg("--mmds-size-limit").arg(mmds_size_limit);
        }

        if let Some(ref module) = self.module {
            cmd.arg("--module").arg(module);
        }

        if let Some(true) = self.no_api {
            cmd.arg("--no-api");
        }

        if let Some(true) = self.no_seccomp {
            cmd.arg("--no-seccomp");
        }

        if let Some(ref parent_cpu_time_us) = self.parent_cpu_time_us {
            cmd.arg("--parent-cpu-time-us")
                .arg(parent_cpu_time_us.to_string());
        }

        if let Some(ref seccomp_filter) = self.seccomp_filter {
            cmd.arg("--seccomp-filter").arg(seccomp_filter);
        }

        if let Some(true) = self.show_level {
            cmd.arg("--show-level");
        }

        if let Some(true) = self.show_log_origin {
            cmd.arg("--show-log-origin");
        }

        if let Some(ref start_time_cpu_us) = self.start_time_cpu_us {
            cmd.arg("--start-time-cpu-us")
                .arg(start_time_cpu_us.to_string());
        }

        if let Some(ref start_time_us) = self.start_time_us {
            cmd.arg("--start-time-us").arg(start_time_us.to_string());
        }

        cmd
    }

    pub fn api_sock(&mut self, api_sock: Option<PathBuf>) -> &mut Self {
        self.api_sock = api_sock;
        self
    }

    pub fn boot_timer(&mut self, boot_timer: Option<bool>) -> &mut Self {
        self.boot_timer = boot_timer;
        self
    }

    pub fn config_file(&mut self, config_file: Option<PathBuf>) -> &mut Self {
        self.config_file = config_file;
        self
    }

    pub fn describe_snapshot(&mut self, describe_snapshot: Option<bool>) -> &mut Self {
        self.describe_snapshot = describe_snapshot;
        self
    }

    pub fn http_api_max_payload_size(
        &mut self,
        http_api_max_payload_size: Option<usize>,
    ) -> &mut Self {
        self.http_api_max_payload_size = http_api_max_payload_size;
        self
    }

    pub fn id(&mut self, id: Option<String>) -> &mut Self {
        self.id = id;
        self
    }

    pub fn level(&mut self, level: Option<String>) -> &mut Self {
        self.level = level;
        self
    }

    pub fn log_path(&mut self, log_path: Option<PathBuf>) -> &mut Self {
        self.log_path = log_path;
        self
    }

    pub fn metadata(&mut self, metadata: Option<PathBuf>) -> &mut Self {
        self.metadata = metadata;
        self
    }

    pub fn metrics_path(&mut self, metrics_path: Option<PathBuf>) -> &mut Self {
        self.metrics_path = metrics_path;
        self
    }

    pub fn mmds_size_limit(&mut self, mmds_size_limit: Option<PathBuf>) -> &mut Self {
        self.mmds_size_limit = mmds_size_limit;
        self
    }

    pub fn module(&mut self, module: Option<String>) -> &mut Self {
        self.module = module;
        self
    }

    pub fn no_api(&mut self) -> &mut Self {
        self.no_api = Some(true);
        self
    }

    pub fn no_seccomp(&mut self) -> &mut Self {
        self.no_seccomp = Some(true);
        self
    }

    pub fn parent_cpu_time_us(&mut self, parent_cpu_time_us: Option<usize>) -> &mut Self {
        self.parent_cpu_time_us = parent_cpu_time_us;
        self
    }

    pub fn seccomp_filter(&mut self, seccomp_filter: Option<String>) -> &mut Self {
        self.seccomp_filter = seccomp_filter;
        self
    }

    pub fn show_level(&mut self) -> &mut Self {
        self.show_level = Some(true);
        self
    }

    pub fn show_log_origin(&mut self) -> &mut Self {
        self.show_log_origin = Some(true);
        self
    }

    pub fn start_time_cpu_us(&mut self, start_time_cpu_us: Option<usize>) -> &mut Self {
        self.start_time_cpu_us = start_time_cpu_us;
        self
    }

    pub fn start_time_us(&mut self, start_time_us: Option<usize>) -> &mut Self {
        self.start_time_us = start_time_us;
        self
    }
}
