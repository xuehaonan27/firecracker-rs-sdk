//! Option to launch jailer

use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

use crate::{
    firecracker::{FirecrackerOption, DEFAULT_API_SOCK, DEFAULT_ID},
    instance::Instance,
    Error, Result,
};

pub const DEFAULT_CGROUP_VERSION: usize = 1;
pub const DEFAULT_CHROOT_BASE_DIR: &'static str = "/srv/jailer";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JailerOption<'f> {
    jailer_bin: PathBuf,

    // File path to exec into.
    exec_file: Option<PathBuf>,

    // The group identifier the jailer switches to after exec.
    gid: Option<usize>,

    // Jail ID.
    id: Option<String>,

    // The user identifier the jailer switches to after exec.
    uid: Option<usize>,

    // Cgroup and value to be set by the jailer. It must follow this format: <cgroup_file>=<value> (e.g cpu.shares=10). This argument can be used multiple times to add multiple cgroups.
    cgroup: Vec<(String, String)>,

    // Select the cgroup version used by the jailer. [default: "1"]
    cgroup_version: Option<usize>,

    // The base folder where chroot jails are located. [default: "/srv/jailer"]
    chroot_base_dir: Option<PathBuf>,

    // Daemonize the jailer before exec, by invoking setsid(), and redirecting the standard I/O file descriptors to /dev/null.
    daemonize: Option<bool>,

    // Path to the network namespace this microVM should join.
    netns: Option<PathBuf>,

    // Exec into a new PID namespace.
    new_pid_ns: Option<bool>,

    // Parent cgroup in which the cgroup of this microvm will be placed.
    parent_cgroup: Option<String>,

    // Resource limit values to be set by the jailer. It must follow this format: <resource>=<value> (e.g no-file=1024).
    // This argument can be used multiple times to add multiple resource limits. Current available resource values are:
    //	 fsize: The maximum size in bytes for files created by the process.
    //	 no-file: Specifies a value one greater than the maximum file descriptor number that can be opened by this process.
    resource_limit: Vec<(String, usize)>,

    #[serde(skip)]
    firecracker_option: Option<&'f FirecrackerOption>,

    // Strategy for changing the jailer chroot.
    chroot_strategy: ChrootStrategy,

    // Whether to remove the jailer directory of the instance after using / error.
    remove_jailer_workspace_dir: Option<bool>,

    // Stdin of the jailer
    stdin: Option<PathBuf>,

    // Stdout of the jailer
    stdout: Option<PathBuf>,

    // Stderr of the jailer
    stderr: Option<PathBuf>,
}

impl<'f> JailerOption<'f> {
    pub fn new<P, Q, S>(jailer_bin: P, exec_file: Q, id: S, gid: usize, uid: usize) -> Self
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
        S: AsRef<str>,
    {
        Self {
            jailer_bin: jailer_bin.as_ref().into(),
            exec_file: Some(exec_file.as_ref().into()),
            id: Some(id.as_ref().into()),
            gid: Some(gid),
            uid: Some(uid),
            ..Default::default()
        }
    }

    fn exec_file_name(&self) -> Result<PathBuf> {
        let exec_file_name = self
            .exec_file
            .as_ref()
            .unwrap()
            .file_name()
            .ok_or_else(|| Error::Configuration("jailer `exec_file` ends with `..`".into()))?;
        Ok(exec_file_name.into())
    }

    fn jailer_workspace_dir(&self) -> Result<PathBuf> {
        let chroot_base_dir = match self.chroot_base_dir {
            Some(ref chroot_base_dir) => chroot_base_dir,
            None => &PathBuf::from(DEFAULT_CHROOT_BASE_DIR),
        };
        let exec_file_name = self.exec_file_name()?;
        let id = self
            .id
            .as_ref()
            .and_then(|s| Some(s.as_str()))
            .unwrap_or_else(|| DEFAULT_ID);
        const ROOT_FOLDER_NAME: &'static str = "root";
        let jailer_workspace_dir = chroot_base_dir
            .join(exec_file_name)
            .join(id)
            .join(ROOT_FOLDER_NAME);

        if jailer_workspace_dir.exists() {
            return Err(Error::Configuration(format!("conflict jailer ID {id}")));
        }

        Ok(jailer_workspace_dir)
    }

    pub fn spawn(&mut self) -> Result<Instance> {
        // spawn instance with jailer
        let mut command = self.build_cmd()?;

        // Redirect stdin, stdout and stderr
        if let Some(ref stdin) = self.stdin {
            command.stdin(Stdio::from(File::open(stdin)?));
        }

        if let Some(ref stdout) = self.stdout {
            command.stdout(Stdio::from(
                OpenOptions::new().create(true).write(true).open(stdout)?,
            ));
        }

        if let Some(ref stderr) = self.stderr {
            command.stderr(Stdio::from(
                OpenOptions::new().create(true).write(true).open(stderr)?,
            ));
        }

        let jailer_workspace_dir = self.jailer_workspace_dir()?;
        let firecracker_api_sock = match self
            .firecracker_option
            .and_then(|opt| opt.api_sock.as_ref())
        {
            Some(x) => x,
            None => &PathBuf::from(DEFAULT_API_SOCK),
        };
        // let socket_on_host = jailer_workspace_dir.join(firecracker_api_sock);
        // let socket_on_host = self
        //     .chroot_strategy
        //     .chroot_path(&jailer_workspace_dir, firecracker_api_sock)?;
        let socket_on_host = ChrootStrategy::FullLinkStrategy
            .chroot_path(&jailer_workspace_dir, firecracker_api_sock)?;

        Ok(Instance::new(
            socket_on_host,
            Some(jailer_workspace_dir),
            Some(self.chroot_strategy.clone()),
            self.remove_jailer_workspace_dir,
            command,
            self.exec_file_name()?,
        ))
    }

    pub fn build_cmd(&mut self) -> Result<Command> {
        let mut cmd = Command::new(&self.jailer_bin);

        let Some(ref exec_file) = self.exec_file else {
            return Err(Error::Configuration("`exec_file` not set".into()));
        };
        cmd.arg("--exec-file").arg(exec_file);

        let Some(ref gid) = self.gid else {
            return Err(Error::Configuration("`gid` not set".into()));
        };
        cmd.arg("--gid").arg(gid.to_string());

        let Some(ref id) = self.id else {
            return Err(Error::Configuration("`id` not set".into()));
        };
        cmd.arg("--id").arg(id.to_string());

        let Some(ref uid) = self.uid else {
            return Err(Error::Configuration("`uid` not set".into()));
        };
        cmd.arg("--uid").arg(uid.to_string());

        for (key, value) in self.cgroup.iter() {
            cmd.arg("--cgroup").arg(format!("{}={}", key, value));
        }

        if let Some(ref cgroup_version) = self.cgroup_version {
            cmd.arg("--cgroup-version").arg(cgroup_version.to_string());
        }

        if let Some(ref chroot_base_dir) = self.chroot_base_dir {
            cmd.arg("--chroot-base-dir").arg(chroot_base_dir);
        }

        if let Some(true) = self.daemonize {
            cmd.arg("--daemonize");
        }

        if let Some(ref netns) = self.netns {
            cmd.arg("--netns").arg(netns);
        }

        if let Some(true) = self.new_pid_ns {
            cmd.arg("--new-pid-ns");
        }

        if let Some(ref parent_cgroup) = self.parent_cgroup {
            cmd.arg("--parent-cgroup").arg(parent_cgroup);
        }

        for (key, value) in self.resource_limit.iter() {
            cmd.arg("--resource-limit")
                .arg(format!("{}={}", key, value));
        }

        if let Some(firecracker_option) = self.firecracker_option {
            let firecracker_cmd = firecracker_option.build_cmd();
            cmd.arg("--").args(firecracker_cmd.get_args());
        }

        Ok(cmd)
    }

    pub fn exec_file<P: AsRef<Path>>(&mut self, exec_file: Option<P>) -> &mut Self {
        self.exec_file = exec_file.and_then(|x| Some(x.as_ref().to_path_buf()));
        self
    }

    pub fn gid(&mut self, gid: Option<usize>) -> &mut Self {
        self.gid = gid;
        self
    }

    pub fn id(&mut self, id: Option<String>) -> &mut Self {
        self.id = id;
        self
    }

    pub fn uid(&mut self, uid: Option<usize>) -> &mut Self {
        self.uid = uid;
        self
    }

    pub fn cgroup(&mut self, cgroup: Vec<(String, String)>) -> &mut Self {
        self.cgroup = cgroup;
        self
    }

    pub fn cgroup_version(&mut self, cgroup_version: Option<usize>) -> &mut Self {
        self.cgroup_version = cgroup_version;
        self
    }

    pub fn chroot_base_dir<P: AsRef<Path>>(&mut self, chroot_base_dir: Option<P>) -> &mut Self {
        self.chroot_base_dir = chroot_base_dir.and_then(|x| Some(x.as_ref().to_path_buf()));
        self
    }

    pub fn daemonize(&mut self) -> &mut Self {
        self.daemonize = Some(true);
        self
    }

    pub fn netns<P: AsRef<Path>>(&mut self, netns: Option<P>) -> &mut Self {
        self.netns = netns.and_then(|x| Some(x.as_ref().to_path_buf()));
        self
    }

    pub fn new_pid_ns(&mut self, new_pid_ns: Option<bool>) -> &mut Self {
        self.new_pid_ns = new_pid_ns;
        self
    }

    pub fn parent_cgroup(&mut self, parent_cgroup: Option<String>) -> &mut Self {
        self.parent_cgroup = parent_cgroup;
        self
    }

    pub fn resource_limit(&mut self, resource_limit: Vec<(String, usize)>) -> &mut Self {
        self.resource_limit = resource_limit;
        self
    }

    pub fn firecracker_option(
        &mut self,
        firecracker_option: Option<&'f FirecrackerOption>,
    ) -> &mut Self {
        self.firecracker_option = firecracker_option;
        self
    }

    pub fn chroot_strategy(&mut self, chroot_strategy: ChrootStrategy) -> &mut Self {
        self.chroot_strategy = chroot_strategy;
        self
    }

    pub fn remove_jailer_workspace_dir(&mut self) -> &mut Self {
        self.remove_jailer_workspace_dir = Some(true);
        self
    }

    pub fn stdin<P: AsRef<Path>>(&mut self, stdin: P) -> &mut Self {
        self.stdin = Some(stdin.as_ref().into());
        self
    }

    pub fn stdout<P: AsRef<Path>>(&mut self, stdout: P) -> &mut Self {
        self.stdout = Some(stdout.as_ref().into());
        self
    }

    pub fn stderr<P: AsRef<Path>>(&mut self, stderr: P) -> &mut Self {
        self.stderr = Some(stderr.as_ref().into());
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ChrootStrategy {
    #[default]
    NaiveLinkStrategy,
    FullLinkStrategy,
}

impl ChrootStrategy {
    /// Return the `chroot`ed path seen by host
    pub fn chroot_path<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        rootfs: P,
        path_on_host: Q,
    ) -> Result<PathBuf> {
        match self {
            Self::NaiveLinkStrategy => {
                let link = rootfs.as_ref().join(
                    path_on_host
                        .as_ref()
                        .file_name()
                        .ok_or_else(|| Error::Configuration("file name ended with `..`".into()))?,
                );
                Ok(link)
            }
            Self::FullLinkStrategy => {
                let path: &Path = path_on_host.as_ref();
                let path = if path.is_absolute() {
                    path.strip_prefix("/").map_err(|e| {
                        Error::Configuration(format!("Fail to strip prefix `/`: {e}"))
                    })?
                } else {
                    path
                };

                let link = rootfs.as_ref().join(path);
                Ok(link)
            }
        }
    }

    /// Perform actual link behavior
    pub fn perform_link<P: AsRef<Path>, Q: AsRef<Path>>(&self, origin: P, link: Q) -> Result<()> {
        match self {
            Self::NaiveLinkStrategy => fs::hard_link(origin.as_ref(), &link)?,
            Self::FullLinkStrategy => fs::hard_link(origin.as_ref(), &link)?,
        }
        Ok(())
    }

    pub fn link_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        rootfs: P,
        path_on_host: Q,
    ) -> Result<PathBuf> {
        let link = self.chroot_path(&rootfs, &path_on_host)?;
        self.perform_link(&path_on_host, &link)?;
        Ok(link)
    }
}
