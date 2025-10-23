use crate::advice::types::Advice;
use anyhow::{Context, Result};
use serde_json;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub const ADVICE_ROOT: &str = "/var/lib/anna/advice";

pub fn ensure_dirs() -> Result<()> {
    ensure_dir(Path::new(ADVICE_ROOT), 0o700)
}

pub fn advice_path(id: &str) -> PathBuf {
    Path::new(ADVICE_ROOT).join(format!("{id}.json"))
}

pub fn write_advice(advice: &Advice) -> Result<PathBuf> {
    ensure_dirs()?;
    let path = advice_path(&advice.id);
    let payload = serde_json::to_vec_pretty(advice)?;
    write_atomic(&path, &payload)?;
    Ok(path)
}

pub fn read_all() -> Result<Vec<Advice>> {
    let mut out = Vec::new();
    let root = Path::new(ADVICE_ROOT);
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
