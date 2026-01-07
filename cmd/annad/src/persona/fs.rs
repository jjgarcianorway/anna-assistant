use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static PERSONA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_persona_dir(path: PathBuf) {
    PERSONA_DIR.set(path).ok();
}

fn persona_dir() -> &'static PathBuf {
    PERSONA_DIR.get().expect("persona dir not initialized")
}

fn samples_dir() -> PathBuf {
    persona_dir().join("samples")
}

pub fn rollups_dir() -> PathBuf {
    persona_dir().join("rollups")
}

pub fn ensure_dirs() -> Result<()> {
    ensure_dir(persona_dir(), 0o700)?;
    ensure_dir(&samples_dir(), 0o700)?;
    ensure_dir(&rollups_dir(), 0o700)?;
    Ok(())
}

pub fn samples_path(date: &str) -> PathBuf {
    samples_dir().join(format!("{date}.ndjson"))
}

pub fn rollup_path(date: &str) -> PathBuf {
    rollups_dir().join(format!("{date}.json"))
}

pub fn append_lines(path: &Path, lines: &[Vec<u8>]) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent, 0o700)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("append to {}", path.display()))?;
    for line in lines {
        file.write_all(line)?;
    }
    file.sync_all()?;
    Ok(())
}

pub fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent, 0o700)?;
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
    Ok(())
}

fn ensure_dir(path: &Path, mode: u32) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create dir {}", path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .with_context(|| format!("set permissions {}", path.display()))?;
    Ok(())
}
