//! Telemetry Database v7.1.0 - SQLite-based Process Telemetry
//!
//! Stores per-process CPU/memory samples over time for:
//! - Real telemetry display in `annactl kdb <object>` [USAGE] section
//! - Real telemetry aggregates in `annactl status` [TELEMETRY] section
//!
//! Schema:
//! - process_samples: PID, name, CPU%, memory, timestamp
//! - telemetry_meta: key-value metadata (last sample time, etc)

use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;

/// Default telemetry database path
pub const TELEMETRY_DB_PATH: &str = "/var/lib/anna/telemetry.db";

/// A single process telemetry sample
#[derive(Debug, Clone)]
pub struct ProcessTelemetrySample {
    pub timestamp: u64,
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub mem_bytes: u64,
}

/// Aggregated telemetry for a single object
#[derive(Debug, Clone, Default)]
pub struct ObjectTelemetry {
    /// Total samples collected for this object
    pub total_samples: u64,
    /// Total CPU time (sum of cpu% * interval)
    pub total_cpu_time_ms: u64,
    /// Peak CPU usage observed
    pub peak_cpu_percent: f32,
    /// Peak memory usage observed
    pub peak_mem_bytes: u64,
    /// Average memory usage
    pub avg_mem_bytes: u64,
    /// First sample timestamp
    pub first_seen: u64,
    /// Last sample timestamp
    pub last_seen: u64,
    /// How many hours of telemetry we have
    pub coverage_hours: f64,
}

/// Overall telemetry statistics
#[derive(Debug, Clone, Default)]
pub struct TelemetryStats {
    /// Total samples in database
    pub total_samples: u64,
    /// Unique processes tracked
    pub unique_processes: u64,
    /// First sample timestamp
    pub first_sample_at: u64,
    /// Last sample timestamp
    pub last_sample_at: u64,
    /// Database size in bytes
    pub db_size_bytes: u64,
    /// Coverage hours (last - first)
    pub coverage_hours: f64,
}

/// SQLite-backed telemetry database
pub struct TelemetryDb {
    conn: Connection,
}

impl TelemetryDb {
    /// Open or create the telemetry database (for daemon use)
    pub fn open() -> Result<Self> {
        Self::open_at(TELEMETRY_DB_PATH)
    }

    /// Open database read-only (for CLI use)
    /// Returns None if file doesn't exist or can't be opened
    pub fn open_readonly() -> Option<Self> {
        let path = Path::new(TELEMETRY_DB_PATH);
        if !path.exists() {
            return None;
        }
        // Open with read-only flag
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
        ).ok()?;
        Some(Self { conn })
    }

    /// Open at a specific path (for testing or daemon)
    pub fn open_at<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let is_new = !path_ref.exists();

        let conn = Connection::open(path_ref)?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Create tables if they don't exist
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS process_samples (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                pid INTEGER NOT NULL,
                name TEXT NOT NULL,
                cpu_percent REAL NOT NULL,
                mem_bytes INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_samples_name ON process_samples(name);
            CREATE INDEX IF NOT EXISTS idx_samples_timestamp ON process_samples(timestamp);
            CREATE INDEX IF NOT EXISTS idx_samples_name_time ON process_samples(name, timestamp);

            CREATE TABLE IF NOT EXISTS telemetry_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#
        )?;

        // Set world-readable permissions so annactl can read
        // (only on new database creation)
        if is_new {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(path_ref, std::fs::Permissions::from_mode(0o644));
            }
        }

        Ok(Self { conn })
    }

    /// Record a process sample
    pub fn record_sample(&self, sample: &ProcessTelemetrySample) -> Result<()> {
        self.conn.execute(
            "INSERT INTO process_samples (timestamp, pid, name, cpu_percent, mem_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                sample.timestamp,
                sample.pid,
                &sample.name,
                sample.cpu_percent,
                sample.mem_bytes
            ],
        )?;
        Ok(())
    }

    /// Record multiple samples in a transaction (more efficient)
    pub fn record_samples(&self, samples: &[ProcessTelemetrySample]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO process_samples (timestamp, pid, name, cpu_percent, mem_bytes)
                 VALUES (?1, ?2, ?3, ?4, ?5)"
            )?;

            for sample in samples {
                stmt.execute(params![
                    sample.timestamp,
                    sample.pid,
                    &sample.name,
                    sample.cpu_percent,
                    sample.mem_bytes
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Get telemetry aggregates for a specific object/process name
    pub fn get_object_telemetry(&self, name: &str) -> Result<ObjectTelemetry> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                COUNT(*) as total_samples,
                SUM(cpu_percent * 15) as total_cpu_time_ms,
                MAX(cpu_percent) as peak_cpu,
                MAX(mem_bytes) as peak_mem,
                AVG(mem_bytes) as avg_mem,
                MIN(timestamp) as first_seen,
                MAX(timestamp) as last_seen
            FROM process_samples
            WHERE name = ?1
            "#
        )?;

        let result = stmt.query_row(params![name], |row| {
            Ok(ObjectTelemetry {
                total_samples: row.get::<_, i64>(0)? as u64,
                total_cpu_time_ms: row.get::<_, f64>(1).unwrap_or(0.0) as u64,
                peak_cpu_percent: row.get::<_, f64>(2).unwrap_or(0.0) as f32,
                peak_mem_bytes: row.get::<_, i64>(3).unwrap_or(0) as u64,
                avg_mem_bytes: row.get::<_, f64>(4).unwrap_or(0.0) as u64,
                first_seen: row.get::<_, i64>(5).unwrap_or(0) as u64,
                last_seen: row.get::<_, i64>(6).unwrap_or(0) as u64,
                coverage_hours: 0.0, // Calculated below
            })
        })?;

        let coverage_hours = if result.first_seen > 0 && result.last_seen > result.first_seen {
            (result.last_seen - result.first_seen) as f64 / 3600.0
        } else {
            0.0
        };

        Ok(ObjectTelemetry {
            coverage_hours,
            ..result
        })
    }

    /// Get overall telemetry statistics
    pub fn get_stats(&self) -> Result<TelemetryStats> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                COUNT(*) as total_samples,
                COUNT(DISTINCT name) as unique_processes,
                MIN(timestamp) as first_sample,
                MAX(timestamp) as last_sample
            FROM process_samples
            "#
        )?;

        let result = stmt.query_row([], |row| {
            let first: i64 = row.get(2).unwrap_or(0);
            let last: i64 = row.get(3).unwrap_or(0);
            Ok(TelemetryStats {
                total_samples: row.get::<_, i64>(0)? as u64,
                unique_processes: row.get::<_, i64>(1)? as u64,
                first_sample_at: first as u64,
                last_sample_at: last as u64,
                db_size_bytes: 0, // Set below
                coverage_hours: if first > 0 && last > first {
                    (last - first) as f64 / 3600.0
                } else {
                    0.0
                },
            })
        })?;

        // Get database file size
        let db_size = std::fs::metadata(TELEMETRY_DB_PATH)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(TelemetryStats {
            db_size_bytes: db_size,
            ..result
        })
    }

    /// Get top N processes by CPU usage in a time window
    pub fn top_by_cpu(&self, since: u64, limit: usize) -> Result<Vec<(String, f64)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT name, SUM(cpu_percent) as total_cpu
            FROM process_samples
            WHERE timestamp >= ?1
            GROUP BY name
            ORDER BY total_cpu DESC
            LIMIT ?2
            "#
        )?;

        let rows = stmt.query_map(params![since as i64, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get top N processes by memory usage in a time window
    pub fn top_by_memory(&self, since: u64, limit: usize) -> Result<Vec<(String, u64)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT name, MAX(mem_bytes) as peak_mem
            FROM process_samples
            WHERE timestamp >= ?1
            GROUP BY name
            ORDER BY peak_mem DESC
            LIMIT ?2
            "#
        )?;

        let rows = stmt.query_map(params![since as i64, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Prune old samples (keep last N days)
    pub fn prune_old_samples(&self, days: u64) -> Result<u64> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(days * 24 * 3600);

        let deleted = self.conn.execute(
            "DELETE FROM process_samples WHERE timestamp < ?1",
            params![cutoff as i64],
        )?;

        // Vacuum to reclaim space
        self.conn.execute_batch("VACUUM;")?;

        Ok(deleted as u64)
    }

    /// Set a metadata value
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO telemetry_meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get a metadata value
    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let result: Result<String, _> = self.conn.query_row(
            "SELECT value FROM telemetry_meta WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Check if database has any data
    pub fn has_data(&self) -> bool {
        self.conn
            .query_row("SELECT COUNT(*) FROM process_samples", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|c| c > 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn test_db() -> TelemetryDb {
        let tmp = NamedTempFile::new().unwrap();
        TelemetryDb::open_at(tmp.path()).unwrap()
    }

    #[test]
    fn test_record_and_query_sample() {
        let db = test_db();

        let sample = ProcessTelemetrySample {
            timestamp: 1700000000,
            pid: 1234,
            name: "firefox".to_string(),
            cpu_percent: 25.5,
            mem_bytes: 500_000_000,
        };

        db.record_sample(&sample).unwrap();

        let telemetry = db.get_object_telemetry("firefox").unwrap();
        assert_eq!(telemetry.total_samples, 1);
        assert!((telemetry.peak_cpu_percent - 25.5).abs() < 0.1);
        assert_eq!(telemetry.peak_mem_bytes, 500_000_000);
    }

    #[test]
    fn test_batch_insert() {
        let db = test_db();

        let samples: Vec<_> = (0..100)
            .map(|i| ProcessTelemetrySample {
                timestamp: 1700000000 + i as u64,
                pid: 1234,
                name: "test".to_string(),
                cpu_percent: 10.0,
                mem_bytes: 100_000,
            })
            .collect();

        db.record_samples(&samples).unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_samples, 100);
        assert_eq!(stats.unique_processes, 1);
    }

    #[test]
    fn test_top_by_cpu() {
        let db = test_db();

        let samples = vec![
            ProcessTelemetrySample {
                timestamp: 1700000000,
                pid: 1,
                name: "high_cpu".to_string(),
                cpu_percent: 90.0,
                mem_bytes: 100,
            },
            ProcessTelemetrySample {
                timestamp: 1700000000,
                pid: 2,
                name: "low_cpu".to_string(),
                cpu_percent: 5.0,
                mem_bytes: 100,
            },
        ];

        db.record_samples(&samples).unwrap();

        let top = db.top_by_cpu(0, 10).unwrap();
        assert_eq!(top[0].0, "high_cpu");
    }

    #[test]
    fn test_metadata() {
        let db = test_db();

        db.set_meta("last_prune", "1700000000").unwrap();
        let val = db.get_meta("last_prune").unwrap();
        assert_eq!(val, Some("1700000000".to_string()));

        let missing = db.get_meta("nonexistent").unwrap();
        assert_eq!(missing, None);
    }
}
