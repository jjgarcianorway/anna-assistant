use super::SignalsCtx;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use time::{Duration as TimeDuration, OffsetDateTime, PrimitiveDateTime};

const PACMAN_LOG: &str = "/var/log/pacman.log";
const DPKG_LOG: &str = "/var/log/dpkg.log";
const DNF_LOG: &str = "/var/log/dnf.rpm.log";

#[derive(Debug, Default, Serialize, Deserialize)]
struct LogOffset {
    pos: u64,
}

pub fn collect(ctx: &SignalsCtx) -> Result<u32> {
    let cutoff = OffsetDateTime::now_utc() - TimeDuration::hours(1);
    let mut total = 0u64;
    total += collect_pacman(ctx.pkgs_dir(), cutoff)? as u64;
    total += collect_dpkg(ctx.pkgs_dir(), cutoff)? as u64;
    total += collect_dnf(ctx.pkgs_dir(), cutoff)? as u64;
    Ok(total.min(u32::MAX as u64) as u32)
}

fn collect_pacman(offset_dir: &Path, cutoff: OffsetDateTime) -> Result<u32> {
    process_log(offset_dir, "pacman", PACMAN_LOG, cutoff, parse_pacman_line)
}

fn collect_dpkg(offset_dir: &Path, cutoff: OffsetDateTime) -> Result<u32> {
    process_log(offset_dir, "dpkg", DPKG_LOG, cutoff, parse_dpkg_line)
}

fn collect_dnf(offset_dir: &Path, cutoff: OffsetDateTime) -> Result<u32> {
    process_log(offset_dir, "dnf", DNF_LOG, cutoff, parse_dnf_line)
}

fn process_log<F>(
    offset_dir: &Path,
    name: &str,
    log_path: &str,
    cutoff: OffsetDateTime,
    parser: F,
) -> Result<u32>
where
    F: Fn(&str) -> Option<LogEvent>,
{
    let path = Path::new(log_path);
    if !path.exists() {
        return Ok(0);
    }
    let offset_path = offset_dir.join(format!("{}.offset", name));
    let mut record = read_offset(&offset_path)?;

    let mut file = File::open(path).with_context(|| format!("open log {}", log_path))?;
    let len = file.metadata()?.len();
    if record.pos > len {
        record.pos = 0;
    }
    file.seek(SeekFrom::Start(record.pos))?;

    let reader = BufReader::new(file);
    let mut count = 0u64;
    for line_res in reader.lines() {
        let line = line_res?;
        if let Some(event) = parser(&line) {
            if event.time >= cutoff {
                count += event.count;
            }
        }
    }

    record.pos = len;
    write_offset(&offset_path, &record)?;
    Ok(count.min(u32::MAX as u64) as u32)
}

#[derive(Debug)]
struct LogEvent {
    time: OffsetDateTime,
    count: u64,
}

fn parse_pacman_line(line: &str) -> Option<LogEvent> {
    let timestamp = line.split(']').next()?.trim_start_matches('[');
    let datetime = PrimitiveDateTime::parse(
        timestamp,
        &time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]").ok()?,
    )
    .ok()?;
    let local_offset = OffsetDateTime::now_local()
        .map(|dt| dt.offset())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().offset());
    let time = datetime
        .assume_offset(local_offset)
        .to_offset(OffsetDateTime::now_utc().offset());
    let count = if line.contains("[ALPM] installed")
        || line.contains("[ALPM] removed")
        || line.contains("[ALPM] upgraded")
    {
        1
    } else {
        0
    };
    if count == 0 {
        return None;
    }
    Some(LogEvent { time, count })
}

fn parse_dpkg_line(line: &str) -> Option<LogEvent> {
    let mut iter = line.split_whitespace();
    let date = iter.next()?;
    let time_part = iter.next()?;
    let datetime = PrimitiveDateTime::parse(
        &format!("{} {}", date, time_part),
        &time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").ok()?,
    )
    .ok()?;
    let local_offset = OffsetDateTime::now_local()
        .map(|dt| dt.offset())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().offset());
    let time = datetime
        .assume_offset(local_offset)
        .to_offset(OffsetDateTime::now_utc().offset());
    let rest = iter.collect::<Vec<&str>>().join(" ");
    let count = if rest.starts_with("install ")
        || rest.starts_with("upgrade ")
        || rest.starts_with("remove ")
    {
        1
    } else {
        0
    };
    if count == 0 {
        return None;
    }
    Some(LogEvent { time, count })
}

fn parse_dnf_line(line: &str) -> Option<LogEvent> {
    let mut parts = line.split_whitespace();
    let timestamp = parts.next()?;
    let datetime =
        OffsetDateTime::parse(timestamp, &time::format_description::well_known::Rfc3339).ok()?;
    let rest = parts.collect::<Vec<&str>>().join(" ");
    let count = if rest.contains("Install:") || rest.contains("Erase:") {
        1
    } else {
        0
    };
    if count == 0 {
        return None;
    }
    Some(LogEvent {
        time: datetime,
        count,
    })
}

fn read_offset(path: &Path) -> Result<LogOffset> {
    if !path.exists() {
        return Ok(LogOffset::default());
    }
    let data = fs::read(path).with_context(|| format!("read package offset {}", path.display()))?;
    let record: LogOffset = serde_json::from_slice(&data).unwrap_or_default();
    Ok(record)
}

fn write_offset(path: &Path, record: &LogOffset) -> Result<()> {
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
        .with_context(|| format!("open package offset {}", path.display()))?
        .write_all(&data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::SignalsCtx;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn counts_pacman_lines() {
        let tmp = temp_root("pacman");
        let ctx = SignalsCtx::for_tests(&tmp.join("signals")).unwrap();
        let log_path = tmp.join("pacman.log");
        let now_local = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let fmt = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]").unwrap();
        let ts1 = now_local.format(&fmt).unwrap();
        let ts2 = (now_local + TimeDuration::minutes(30))
            .format(&fmt)
            .unwrap();
        let content = format!("[{ts1}] [ALPM] installed foo\n[{ts2}] [ALPM] removed bar\n");
        std::fs::write(&log_path, content).unwrap();
        let cutoff = OffsetDateTime::now_utc() - TimeDuration::hours(2);
        let count = process_log(
            ctx.pkgs_dir(),
            "testpacman",
            log_path.to_str().unwrap(),
            cutoff,
            parse_pacman_line,
        )
        .unwrap();
        assert_eq!(count, 2);
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
