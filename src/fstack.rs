use std::{fs, path::PathBuf, process::Command};

use log::{error, info};

pub struct FStack {
    inner: Vec<FStackAction>,
}

pub enum FStackAction {
    RemoveDirectory(PathBuf),
    RemoveFile(PathBuf),
    TerminateProcess(u32),
}

impl Drop for FStack {
    fn drop(&mut self) {
        while let Some(action) = self.inner.pop() {
            match action {
                FStackAction::RemoveDirectory(dir) => {
                    info!("FStack: performing `RemoveDirectory({})`", dir.display());
                    let dir: PathBuf = dir.into();
                    if dir.exists() && dir.is_dir() {
                        let _ = fs::remove_dir_all(&dir);
                    } else {
                        error!("FStack: {} does not exist!", dir.display());
                    }
                }
                FStackAction::RemoveFile(path) => {
                    info!("FStack: performing `RemoveFile({})`", path.display());
                    if let Err(e) = fs::remove_file(&path) {
                        error!("FStack: fail to remove file {}: {e}", path.display());
                        /* We could do nothing on error though... */
                    }
                }
                FStackAction::TerminateProcess(pid) => {
                    info!("FStack: performing `TerminateProcess({})`", pid);
                    match Command::new("kill")
                        .arg("-15")
                        .arg(pid.to_string())
                        .output()
                    {
                        Ok(_output) => {
                            info!("FStack: killed process {pid}");
                        }
                        Err(e) => {
                            error!("FStack: fail to terminate process {pid}: {e}");
                        }
                    }
                }
            }
        }
    }
}

impl FStack {
    pub fn new() -> Self {
        FStack { inner: Vec::new() }
    }

    pub fn push_action(&mut self, action: FStackAction) {
        self.inner.push(action);
    }

    /// Drop this FStackStack without rollback.
    /// Called when we are sure that everything is running well and
    /// do not need rollback.
    pub fn cancel(mut self) {
        self.inner.clear();
        info!("FStack: stack cancelled, are we going well?");
    }
}
