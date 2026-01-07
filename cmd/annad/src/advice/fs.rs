use crate::advice::types::Advice;
use anyhow::{anyhow, Context, Result};
use serde_json;
#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

static ADVICE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_advice_dir(path: PathBuf) {
    ADVICE_DIR.set(path).ok();
}

fn advice_dir() -> &'static PathBuf {
    ADVICE_DIR.get().expect("advice dir not initialized")
}

pub fn ensure_dirs() -> Result<()> {
    let path = advice_dir();
    ensure_dir(path, 0o700)?;
    configure_advice_dir(path)?;
    if path.exists() {
        for entry in
            fs::read_dir(path).with_context(|| format!("scan advice dir {}", path.display()))?
        {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            if let Err(err) = grant_group_read(&file_path) {
                tracing::warn!(
                    target: "annad",
                    "failed to refresh advice ACL {}: {err}",
                    file_path.display()
                );
            }
        }
    }
    Ok(())
}

pub fn advice_path(id: &str) -> PathBuf {
    advice_dir().join(format!("{id}.json"))
}

pub fn write_advice(advice: &Advice) -> Result<PathBuf> {
    ensure_dirs()?;
    let path = advice_path(&advice.id);
    let payload = serde_json::to_vec_pretty(advice)?;
    write_atomic(&path, &payload)?;
    Ok(path)
}

/// Write advice to custom directory (for per-user paths)
pub fn write_advice_to(advice: &Advice, advice_dir: &Path) -> Result<PathBuf> {
    ensure_dir(advice_dir, 0o2770)?;
    configure_advice_dir(advice_dir)?;
    let path = advice_dir.join(format!("{}.json", advice.id));
    let payload = serde_json::to_vec_pretty(advice)?;
    write_atomic(&path, &payload)?;
    Ok(path)
}

pub fn read_all() -> Result<Vec<Advice>> {
    let mut out = Vec::new();
    let root = advice_dir();
    if !root.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(root).with_context(|| format!("read dir {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let data = fs::read(&path).with_context(|| format!("read advice {}", path.display()))?;
        match serde_json::from_slice::<Advice>(&data) {
            Ok(advice) => out.push(advice),
            Err(err) => {
                tracing::warn!(
                    target: "annad",
                    "skip malformed advice {}: {err}",
                    path.display()
                );
            }
        }
    }
    Ok(out)
}

fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent, 0o700)?;
        configure_advice_dir(parent)?;
    }
    let tmp = path.with_extension("tmp");
    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .with_context(|| format!("open temp {}", tmp.display()))?;
        file.write_all(data)?;
        file.sync_all()?;
    }
    if let Some(parent) = path.parent() {
        let dir = File::open(parent).with_context(|| format!("open dir {}", parent.display()))?;
        dir.sync_all()?;
    }
    fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    if let Err(err) = grant_group_read(path) {
        tracing::warn!(
            target: "annad",
            "failed to extend advice ACL {}: {err}",
            path.display()
        );
    }
    Ok(())
}

fn ensure_dir(path: &Path, mode: u32) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create dir {}", path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .with_context(|| format!("set permissions {}", path.display()))?;
    Ok(())
}

fn configure_advice_dir(path: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if !path.exists() {
            return Ok(());
        }
        if let Some(gid) = resolve_anna_gid()? {
            set_group_owner(path, gid)?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o2770))
                .with_context(|| format!("set permissions {}", path.display()))?;
            if let Err(err) = apply_acl_spec(path, "g:anna:rx") {
                tracing::warn!(
                    target: "annad",
                    "setfacl failed for {}: {err}",
                    path.display()
                );
            }
            if let Err(err) = apply_acl_spec(path, "d:g:anna:rx") {
                tracing::warn!(
                    target: "annad",
                    "setfacl default failed for {}: {err}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn grant_group_read(path: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if let Some(gid) = resolve_anna_gid()? {
            set_group_owner(path, gid)?;
            if let Err(err) = apply_acl_spec(path, "g:anna:r") {
                tracing::warn!(
                    target: "annad",
                    "setfacl failed for {}: {err}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn resolve_anna_gid() -> Result<Option<libc::gid_t>> {
    let group = CString::new("anna").context("group name CString")?;
    let grp = unsafe { libc::getgrnam(group.as_ptr()) };
    if grp.is_null() {
        return Ok(None);
    }
    // Safety: pointer remains valid for duration of call per getgrnam contract.
    let gid = unsafe { (*grp).gr_gid };
    Ok(Some(gid))
}

#[cfg(target_os = "linux")]
fn set_group_owner(path: &Path, gid: libc::gid_t) -> Result<()> {
    let c_path = CString::new(path.as_os_str().as_bytes()).context("path CString")?;
    let rc = unsafe { libc::chown(c_path.as_ptr(), (!0) as libc::uid_t, gid) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("chown {}", path.display()));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn apply_acl_spec(path: &Path, spec: &str) -> Result<()> {
    let status = Command::new("setfacl")
        .arg("-m")
        .arg(spec)
        .arg(path)
        .status();

    match status {
        Ok(exit) if exit.success() => Ok(()),
        Ok(exit) => Err(anyhow!("setfacl exited with status {}", exit)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("invoke setfacl {}", path.display())),
    }
}
