use super::{SignalsCtx, UserHome};
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use time::{OffsetDateTime, PrimitiveDateTime, Time};

#[derive(Debug, Default, Serialize, Deserialize)]
struct RowOffset {
    last_row: i64,
}

pub fn collect(ctx: &SignalsCtx, homes: &[UserHome]) -> Result<u32> {
    let mut total = 0u64;
    for user in homes {
        total += collect_firefox(ctx, user)? as u64;
        total += collect_chromium(ctx, user)? as u64;
    }
    Ok(total.min(u32::MAX as u64) as u32)
}

fn collect_firefox(ctx: &SignalsCtx, user: &UserHome) -> Result<u32> {
    let profiles_dir = user.home.join(".mozilla/firefox");
    if !profiles_dir.exists() {
        return Ok(0);
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let profile_dir = entry.path();
            let db_path = profile_dir.join("places.sqlite");
            if !db_path.exists() {
                continue;
            }
            let ident = sanitize_identifier(user.uid, profile_dir.file_name().unwrap_or_default());
            total += collect_firefox_profile(ctx.browser_dir(), &db_path, &ident)? as u64;
        }
    }
    Ok(total.min(u32::MAX as u64) as u32)
}

fn collect_firefox_profile(dir: &Path, db_path: &Path, ident: &str) -> Result<u32> {
    let offset_path = dir.join(format!("firefox_{}.offset", ident));
    let mut offset = read_row_offset(&offset_path)?;

    let conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(conn) => conn,
        Err(err) => {
            if matches!(err, rusqlite::Error::SqliteFailure(_, _)) {
                return Ok(0);
            }
            return Err(err.into());
        }
    };
    conn.busy_timeout(std::time::Duration::from_millis(100))
        .ok();

    let start_threshold = start_of_day_unix_micros();
    let mut stmt = conn
        .prepare(
            "SELECT rowid, visit_date FROM moz_historyvisits WHERE rowid > ? ORDER BY rowid ASC",
        )
        .context("prepare firefox query")?;
    let mut rows = stmt.query([offset.last_row])?;
    let mut count = 0u64;
    let mut max_row = offset.last_row;
    while let Some(row) = rows.next()? {
        let rowid: i64 = row.get(0)?;
        let visit_date: Option<i64> = row.get(1).ok();
        if let Some(ts) = visit_date {
            if ts >= start_threshold {
                count += 1;
            }
        }
        if rowid > max_row {
            max_row = rowid;
        }
    }
    offset.last_row = max_row;
    write_row_offset(&offset_path, &offset)?;
    Ok(count.min(u32::MAX as u64) as u32)
}

fn collect_chromium(ctx: &SignalsCtx, user: &UserHome) -> Result<u32> {
    let mut total = 0u64;
    let config_dir = user.home.join(".config");
    if config_dir.exists() {
        total += collect_chromium_root(ctx, user, &config_dir)? as u64;
    }
    let local_app = user.home.join(".local/share");
    if local_app.exists() {
        total += collect_chromium_root(ctx, user, &local_app)? as u64;
    }
    Ok(total.min(u32::MAX as u64) as u32)
}

fn collect_chromium_root(ctx: &SignalsCtx, user: &UserHome, root: &Path) -> Result<u32> {
    let mut total = 0u64;
    for product in [
        "chromium",
        "google-chrome",
        "google-chrome-beta",
        "google-chrome-unstable",
    ] {
        let product_dir = root.join(product);
        if !product_dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&product_dir) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let profile_dir = entry.path();
                let db_path = profile_dir.join("History");
                if !db_path.exists() {
                    continue;
                }
                let ident =
                    sanitize_identifier(user.uid, profile_dir.file_name().unwrap_or_default());
                total += collect_chromium_profile(ctx.browser_dir(), &db_path, &ident)? as u64;
            }
        }
    }
    Ok(total.min(u32::MAX as u64) as u32)
}

fn collect_chromium_profile(dir: &Path, db_path: &Path, ident: &str) -> Result<u32> {
    let offset_path = dir.join(format!("chromium_{}.offset", ident));
    let mut offset = read_row_offset(&offset_path)?;

    let conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(conn) => conn,
        Err(err) => {
            if matches!(err, rusqlite::Error::SqliteFailure(_, _)) {
                return Ok(0);
            }
            return Err(err.into());
        }
    };
    conn.busy_timeout(std::time::Duration::from_millis(100))
        .ok();

    let start_threshold = start_of_day_webkit_ticks();
    let mut stmt = conn
        .prepare("SELECT id, visit_time FROM visits WHERE id > ? ORDER BY id ASC")
        .context("prepare chromium query")?;
    let mut rows = stmt.query([offset.last_row])?;
    let mut count = 0u64;
    let mut max_row = offset.last_row;
    while let Some(row) = rows.next()? {
        let rowid: i64 = row.get(0)?;
        let visit_time: Option<i64> = row.get(1).ok();
        if let Some(ts) = visit_time {
            if ts >= start_threshold {
                count += 1;
            }
        }
        if rowid > max_row {
            max_row = rowid;
        }
    }
    offset.last_row = max_row;
    write_row_offset(&offset_path, &offset)?;
    Ok(count.min(u32::MAX as u64) as u32)
}

fn sanitize_identifier(uid: u32, name: &std::ffi::OsStr) -> String {
    let mut ident = format!("{}_", uid);
    ident.push_str(
        &name
            .to_string_lossy()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
    );
    ident
}

fn read_row_offset(path: &Path) -> Result<RowOffset> {
    if !path.exists() {
        return Ok(RowOffset::default());
    }
    let data = fs::read(path).with_context(|| format!("read browser offset {}", path.display()))?;
    let record: RowOffset = serde_json::from_slice(&data).unwrap_or_default();
    Ok(record)
}

fn write_row_offset(path: &Path, record: &RowOffset) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_vec(record)?;
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("open browser offset {}", path.display()))?
        .write_all(&data)?;
    Ok(())
}

fn start_of_day_unix_micros() -> i64 {
    let local = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let midnight = PrimitiveDateTime::new(local.date(), Time::from_hms(0, 0, 0).unwrap())
        .assume_offset(local.offset());
    midnight.unix_timestamp() * 1_000_000
}

fn start_of_day_webkit_ticks() -> i64 {
    let utc_start = start_of_day_unix_micros();
    unix_micros_to_webkit(utc_start)
}

fn unix_micros_to_webkit(us: i64) -> i64 {
    // WebKit epoch starts 1601-01-01T00:00:00Z, which is 11644473600 seconds before Unix epoch.
    us + 11_644_473_600i64 * 1_000_000
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::{self, SignalsCtx};
    use rusqlite::params;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use time::{Duration as TimeDuration, OffsetDateTime};

    #[test]
    fn counts_firefox_visits() {
        let tmp = temp_root("firefox");
        let ctx = SignalsCtx::for_tests(&tmp.join("signals")).unwrap();

        let home = tmp.join("home");
        fs::create_dir_all(&home.join(".mozilla/firefox/test.default")).unwrap();
        let db_path = home.join(".mozilla/firefox/test.default/places.sqlite");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE moz_historyvisits (id INTEGER PRIMARY KEY, from_visit INTEGER, place_id INTEGER, visit_date INTEGER)",
            [],
        )
        .unwrap();
        let now_us = OffsetDateTime::now_utc().unix_timestamp() as i64 * 1_000_000;
        conn.execute(
            "INSERT INTO moz_historyvisits (visit_date) VALUES (?)",
            params![now_us],
        )
        .unwrap();

        let homes = vec![signals::test_user_home(1111, home.clone())];
        let count = collect(&ctx, &homes).unwrap();
        assert_eq!(count, 1);

        // Append another visit and expect only the new one to be counted.
        let later = (OffsetDateTime::now_utc() + TimeDuration::seconds(60)).unix_timestamp() as i64
            * 1_000_000;
        conn.execute(
            "INSERT INTO moz_historyvisits (visit_date) VALUES (?)",
            params![later],
        )
        .unwrap();
        let count = collect(&ctx, &homes).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn counts_chromium_visits() {
        let tmp = temp_root("chromium");
        let ctx = SignalsCtx::for_tests(&tmp.join("signals")).unwrap();

        let home = tmp.join("home");
        let profile_dir = home.join(".config/chromium/Default");
        fs::create_dir_all(&profile_dir).unwrap();
        let db_path = profile_dir.join("History");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE visits (id INTEGER PRIMARY KEY, visit_time INTEGER)",
            [],
        )
        .unwrap();
        let now_webkit =
            unix_micros_to_webkit(OffsetDateTime::now_utc().unix_timestamp() as i64 * 1_000_000);
        conn.execute(
            "INSERT INTO visits (visit_time) VALUES (?)",
            params![now_webkit],
        )
        .unwrap();

        let homes = vec![signals::test_user_home(2222, home.clone())];
        let count = collect(&ctx, &homes).unwrap();
        assert_eq!(count, 1);

        let later = unix_micros_to_webkit(
            (OffsetDateTime::now_utc() + TimeDuration::seconds(120)).unix_timestamp() as i64
                * 1_000_000,
        );
        conn.execute("INSERT INTO visits (visit_time) VALUES (?)", params![later])
            .unwrap();
        let count = collect(&ctx, &homes).unwrap();
        assert_eq!(count, 1);
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
