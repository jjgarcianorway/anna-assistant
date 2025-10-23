use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub mod browser;
pub mod pkgs;
pub mod shell;

use crate::config::SignalsConfig;

const SIGNALS_ROOT: &str = "/var/lib/anna/signals";
const SHELL_DIR: &str = "shell";
const BROWSER_DIR: &str = "browser";
const PKGS_DIR: &str = "pkgs";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SignalDelta {
    pub shell_lines: u32,
    pub browser_navs: u32,
    pub pkg_churn: u32,
}

impl SignalDelta {
    pub fn is_empty(&self) -> bool {
        self.shell_lines == 0 && self.browser_navs == 0 && self.pkg_churn == 0
    }
}

#[derive(Clone)]
pub struct SignalsCtx {
    shell_dir: PathBuf,
    browser_dir: PathBuf,
    pkgs_dir: PathBuf,
}

impl SignalsCtx {
    pub fn new() -> Result<Self> {
        Self::with_root(Path::new(SIGNALS_ROOT))
    }

    fn with_root(root_path: &Path) -> Result<Self> {
        ensure_dir(root_path, 0o700)?;
        let shell_dir = root_path.join(SHELL_DIR);
        let browser_dir = root_path.join(BROWSER_DIR);
        let pkgs_dir = root_path.join(PKGS_DIR);
        ensure_dir(&shell_dir, 0o700)?;
        ensure_dir(&browser_dir, 0o700)?;
        ensure_dir(&pkgs_dir, 0o700)?;
        Ok(Self {
            shell_dir,
            browser_dir,
            pkgs_dir,
        })
    }

    pub fn shell_dir(&self) -> &Path {
        &self.shell_dir
    }

    pub fn browser_dir(&self) -> &Path {
        &self.browser_dir
    }

    pub fn pkgs_dir(&self) -> &Path {
        &self.pkgs_dir
    }

    #[cfg(test)]
    pub fn for_tests(root: &Path) -> Result<Self> {
        Self::with_root(root)
    }
}

#[derive(Debug, Clone)]
pub struct UserHome {
    pub uid: u32,
    pub home: PathBuf,
}

pub fn collect(ctx: &SignalsCtx, cfg: &SignalsConfig) -> Result<SignalDelta> {
    let homes = discover_user_homes()?;
    collect_with_homes(ctx, cfg, &homes)
}

pub(crate) fn collect_with_homes(
    ctx: &SignalsCtx,
    cfg: &SignalsConfig,
    homes: &[UserHome],
) -> Result<SignalDelta> {
    let mut delta = SignalDelta::default();
    if cfg.allow_shell_history {
        delta.shell_lines = shell::collect(ctx, homes)?;
    }
    if cfg.allow_browser_history {
        delta.browser_navs = browser::collect(ctx, homes)?;
    }
    delta.pkg_churn = pkgs::collect(ctx)?;
    Ok(delta)
}

fn discover_user_homes() -> Result<Vec<UserHome>> {
    let mut uids = HashSet::new();
    for entry in fs::read_dir("/proc")? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_name = entry.file_name();
        let pid_str = match file_name.to_str() {
            Some(s) => s,
            None => continue,
        };
        if !pid_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let status_path = entry.path().join("status");
        if let Ok(contents) = fs::read_to_string(status_path) {
            for line in contents.lines() {
                if let Some(rest) = line.strip_prefix("Uid:") {
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    if let Some(first) = parts.first() {
                        if let Ok(uid) = first.parse::<u32>() {
                            if uid != 0 {
                                uids.insert(uid);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    if uids.is_empty() {
        return Ok(Vec::new());
    }

    let passwd_map = read_passwd_map()?;
    let mut homes = Vec::new();
    for uid in uids {
        if let Some(home) = passwd_map.get(&uid) {
            homes.push(UserHome {
                uid,
                home: home.clone(),
            });
        }
    }
    Ok(homes)
}

fn read_passwd_map() -> Result<HashMap<u32, PathBuf>> {
    let contents = fs::read_to_string("/etc/passwd").context("read /etc/passwd")?;
    let mut map = HashMap::new();
    for line in contents.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 7 {
            continue;
        }
        if let Ok(uid) = parts[2].parse::<u32>() {
            map.insert(uid, PathBuf::from(parts[5]));
        }
    }
    Ok(map)
}

fn ensure_dir(path: &Path, mode: u32) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create dir {}", path.display()))?;
    let perms = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perms)
        .with_context(|| format!("set permissions {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn test_user_home(uid: u32, home: PathBuf) -> UserHome {
    UserHome { uid, home }
}
