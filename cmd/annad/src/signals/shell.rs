use super::SignalsCtx;
use super::UserHome;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

const BASH_FILE: &str = ".bash_history";
const ZSH_FILE: &str = ".zsh_history";

#[derive(Debug, Serialize, Deserialize, Default)]
struct OffsetRecord {
    pos: u64,
}

pub fn collect(ctx: &SignalsCtx, homes: &[UserHome]) -> Result<u32> {
    let mut total = 0u64;
    for home in homes {
        total += collect_history(ctx.shell_dir(), home, BASH_FILE, "bash")? as u64;
        total += collect_history(ctx.shell_dir(), home, ZSH_FILE, "zsh")? as u64;
    }
    Ok(total.min(u32::MAX as u64) as u32)
}

fn collect_history(dir: &Path, user: &UserHome, filename: &str, label: &str) -> Result<u32> {
    let path = user.home.join(filename);
    if !path.exists() {
        return Ok(0);
    }

    let offset_path = dir.join(format!("{}-{}.offset", user.uid, label));
    let mut record = read_offset(&offset_path)?;

    let mut file = File::open(&path).with_context(|| format!("open history {}", path.display()))?;
    let metadata = file.metadata()?;
    let len = metadata.len();
    if record.pos > len {
        record.pos = 0;
    }
    file.seek(SeekFrom::Start(record.pos))?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let mut count = 0u64;
    if !buffer.is_empty() {
        count += buffer.iter().filter(|b| **b == b'\n').count() as u64;
        if !buffer.ends_with(b"\n") {
            count += 1;
        }
    }

    record.pos = len;
    write_offset(&offset_path, &record)?;
    Ok(count.min(u32::MAX as u64) as u32)
}

fn read_offset(path: &Path) -> Result<OffsetRecord> {
    if !path.exists() {
        return Ok(OffsetRecord::default());
    }
    let data = fs::read(path).with_context(|| format!("read shell offset {}", path.display()))?;
    let record: OffsetRecord = serde_json::from_slice(&data).unwrap_or_default();
    Ok(record)
}

fn write_offset(path: &Path, record: &OffsetRecord) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_vec(record)?;
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("open shell offset {}", path.display()))?
        .write_all(&data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SignalsConfig;
    use crate::signals::{self, SignalsCtx};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn counts_appended_lines() {
        let tmp = temp_root("shell");
        let signals_root = tmp.join("signals");
        let ctx = SignalsCtx::for_tests(&signals_root).unwrap();

        let home_dir = tmp.join("home");
        fs::create_dir_all(&home_dir).unwrap();
        let bash_path = home_dir.join(BASH_FILE);
        fs::write(&bash_path, b"one\ntwo\n").unwrap();

        let homes = vec![signals::test_user_home(1234, home_dir.clone())];
        let cfg = SignalsConfig {
            allow_shell_history: true,
            allow_browser_history: false,
        };

        // First collection counts existing lines.
        let delta = signals::collect_with_homes(&ctx, &cfg, &homes).unwrap();
        assert_eq!(delta.shell_lines, 2);

        // Append new lines and ensure only new ones are counted.
        let mut file = OpenOptions::new().append(true).open(&bash_path).unwrap();
        writeln!(file, "three").unwrap();
        writeln!(file, "four").unwrap();

        let delta = signals::collect_with_homes(&ctx, &cfg, &homes).unwrap();
        assert_eq!(delta.shell_lines, 2);
    }

    fn temp_root(prefix: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let mut dir = std::env::temp_dir();
        let idx = COUNTER.fetch_add(1, Ordering::SeqCst);
        dir.push(format!(
            "anna_signals_{prefix}_{}_{}",
            std::process::id(),
            idx
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
