//! Telemetry Database v7.3.0 - SQLite-based Process Telemetry
//!
//! Stores per-process CPU/memory samples over time for:
//! - Real telemetry display in `annactl kdb <object>` [USAGE] section
//! - Real telemetry aggregates in `annactl status` [TELEMETRY] section
//!
//! Schema:
//! - process_samples: PID, name, CPU%, memory, timestamp
//! - telemetry_meta: key-value metadata (last sample time, etc)
//!
//! v7.2.0: Added time-windowed aggregations and global peak queries
//! v7.3.0: Added multi-window per-object stats for PHASE 9

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

/// Sample counts for different time windows
#[derive(Debug, Clone, Default)]
pub struct SampleCounts {
    /// Samples in last 1 hour
    pub last_1h: u64,
    /// Samples in last 24 hours
    pub last_24h: u64,
    /// Samples in last 7 days
    pub last_7d: u64,
    /// Samples in last 30 days
    pub last_30d: u64,
}

/// Usage statistics for a time window
#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    /// Average CPU percent
    pub avg_cpu_percent: f32,
    /// Peak CPU percent
    pub peak_cpu_percent: f32,
    /// Average RSS in bytes
    pub avg_mem_bytes: u64,
    /// Peak RSS in bytes
    pub peak_mem_bytes: u64,
    /// Number of samples in window
    pub sample_count: u64,
    /// Whether we have enough data (>= 10 minutes)
    pub has_enough_data: bool,
}

/// Global peak information
#[derive(Debug, Clone, Default)]
pub struct GlobalPeak {
    /// Process/command name
    pub name: String,
    /// Peak value (CPU% or bytes)
    pub value: f64,
    /// Timestamp of peak
    pub timestamp: u64,
    /// PID at peak (if available)
    pub pid: u32,
}

/// Data status for telemetry
#[derive(Debug, Clone, PartialEq)]
pub enum DataStatus {
    /// No data available
    NoData,
    /// Less than 10 minutes of samples
    NotEnoughData { minutes: f64 },
    /// Less than 24h of data
    PartialWindow { hours: f64 },
    /// 24h+ of samples
    Ok { hours: f64 },
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

    // ========================================================================
    // PHASE 3: Time-windowed aggregations
    // ========================================================================

    /// Get current unix timestamp
    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get data status based on coverage
    pub fn get_data_status(&self) -> DataStatus {
        let stats = match self.get_stats() {
            Ok(s) => s,
            Err(_) => return DataStatus::NoData,
        };

        if stats.total_samples == 0 {
            return DataStatus::NoData;
        }

        let minutes = stats.coverage_hours * 60.0;
        if minutes < 10.0 {
            return DataStatus::NotEnoughData { minutes };
        }

        if stats.coverage_hours < 24.0 {
            return DataStatus::PartialWindow { hours: stats.coverage_hours };
        }

        DataStatus::Ok { hours: stats.coverage_hours }
    }

    /// Get sample counts for a command in different time windows
    pub fn get_sample_counts(&self, name: &str) -> Result<SampleCounts> {
        let now = Self::now();
        let h1 = now.saturating_sub(3600);
        let h24 = now.saturating_sub(24 * 3600);
        let d7 = now.saturating_sub(7 * 24 * 3600);
        let d30 = now.saturating_sub(30 * 24 * 3600);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                SUM(CASE WHEN timestamp >= ?2 THEN 1 ELSE 0 END) as cnt_1h,
                SUM(CASE WHEN timestamp >= ?3 THEN 1 ELSE 0 END) as cnt_24h,
                SUM(CASE WHEN timestamp >= ?4 THEN 1 ELSE 0 END) as cnt_7d,
                SUM(CASE WHEN timestamp >= ?5 THEN 1 ELSE 0 END) as cnt_30d
            FROM process_samples
            WHERE name = ?1
            "#
        )?;

        stmt.query_row(params![name, h1 as i64, h24 as i64, d7 as i64, d30 as i64], |row| {
            Ok(SampleCounts {
                last_1h: row.get::<_, i64>(0).unwrap_or(0) as u64,
                last_24h: row.get::<_, i64>(1).unwrap_or(0) as u64,
                last_7d: row.get::<_, i64>(2).unwrap_or(0) as u64,
                last_30d: row.get::<_, i64>(3).unwrap_or(0) as u64,
            })
        }).map_err(|e| e.into())
    }

    /// Get usage stats for a command in the last 24h
    pub fn get_usage_stats_24h(&self, name: &str) -> Result<UsageStats> {
        let now = Self::now();
        let h24 = now.saturating_sub(24 * 3600);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                COUNT(*) as cnt,
                AVG(cpu_percent) as avg_cpu,
                MAX(cpu_percent) as peak_cpu,
                AVG(mem_bytes) as avg_mem,
                MAX(mem_bytes) as peak_mem,
                MIN(timestamp) as first_ts,
                MAX(timestamp) as last_ts
            FROM process_samples
            WHERE name = ?1 AND timestamp >= ?2
            "#
        )?;

        stmt.query_row(params![name, h24 as i64], |row| {
            let cnt: i64 = row.get(0).unwrap_or(0);
            let first_ts: i64 = row.get(5).unwrap_or(0);
            let last_ts: i64 = row.get(6).unwrap_or(0);

            // Check if we have at least 10 minutes of data
            let duration_minutes = if last_ts > first_ts {
                (last_ts - first_ts) as f64 / 60.0
            } else {
                0.0
            };

            Ok(UsageStats {
                sample_count: cnt as u64,
                avg_cpu_percent: row.get::<_, f64>(1).unwrap_or(0.0) as f32,
                peak_cpu_percent: row.get::<_, f64>(2).unwrap_or(0.0) as f32,
                avg_mem_bytes: row.get::<_, f64>(3).unwrap_or(0.0) as u64,
                peak_mem_bytes: row.get::<_, i64>(4).unwrap_or(0) as u64,
                has_enough_data: duration_minutes >= 10.0 || cnt >= 40, // ~10min at 15s intervals
            })
        }).map_err(|e| e.into())
    }

    /// Get global peak CPU in last 24h
    pub fn get_global_peak_cpu_24h(&self) -> Result<Option<GlobalPeak>> {
        let now = Self::now();
        let h24 = now.saturating_sub(24 * 3600);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT name, cpu_percent, timestamp, pid
            FROM process_samples
            WHERE timestamp >= ?1
            ORDER BY cpu_percent DESC
            LIMIT 1
            "#
        )?;

        let result = stmt.query_row(params![h24 as i64], |row| {
            Ok(GlobalPeak {
                name: row.get(0)?,
                value: row.get::<_, f64>(1)?,
                timestamp: row.get::<_, i64>(2)? as u64,
                pid: row.get::<_, i64>(3)? as u32,
            })
        });

        match result {
            Ok(peak) => Ok(Some(peak)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get global peak memory in last 24h
    pub fn get_global_peak_mem_24h(&self) -> Result<Option<GlobalPeak>> {
        let now = Self::now();
        let h24 = now.saturating_sub(24 * 3600);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT name, mem_bytes, timestamp, pid
            FROM process_samples
            WHERE timestamp >= ?1
            ORDER BY mem_bytes DESC
            LIMIT 1
            "#
        )?;

        let result = stmt.query_row(params![h24 as i64], |row| {
            Ok(GlobalPeak {
                name: row.get(0)?,
                value: row.get::<_, i64>(1)? as f64,
                timestamp: row.get::<_, i64>(2)? as u64,
                pid: row.get::<_, i64>(3)? as u32,
            })
        });

        match result {
            Ok(peak) => Ok(Some(peak)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // ========================================================================
    // PHASE 8: Top-N helpers for KDB overview
    // ========================================================================

    /// Get top N processes by sample count (launches) in 24h
    pub fn top_by_launches_24h(&self, limit: usize) -> Result<Vec<(String, u64)>> {
        let now = Self::now();
        let h24 = now.saturating_sub(24 * 3600);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT name, COUNT(*) as cnt
            FROM process_samples
            WHERE timestamp >= ?1
            GROUP BY name
            ORDER BY cnt DESC
            LIMIT ?2
            "#
        )?;

        let rows = stmt.query_map(params![h24 as i64, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get top N processes by average CPU in 24h
    pub fn top_by_avg_cpu_24h(&self, limit: usize) -> Result<Vec<(String, f64)>> {
        let now = Self::now();
        let h24 = now.saturating_sub(24 * 3600);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT name, AVG(cpu_percent) as avg_cpu
            FROM process_samples
            WHERE timestamp >= ?1
            GROUP BY name
            HAVING COUNT(*) >= 2
            ORDER BY avg_cpu DESC
            LIMIT ?2
            "#
        )?;

        let rows = stmt.query_map(params![h24 as i64, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get top N processes by average memory (RSS) in 24h
    pub fn top_by_avg_memory_24h(&self, limit: usize) -> Result<Vec<(String, u64)>> {
        let now = Self::now();
        let h24 = now.saturating_sub(24 * 3600);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT name, AVG(mem_bytes) as avg_mem
            FROM process_samples
            WHERE timestamp >= ?1
            GROUP BY name
            HAVING COUNT(*) >= 2
            ORDER BY avg_mem DESC
            LIMIT ?2
            "#
        )?;

        let rows = stmt.query_map(params![h24 as i64, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)? as u64))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get retention window (days of data kept)
    pub fn get_retention_days(&self) -> f64 {
        match self.get_stats() {
            Ok(stats) => stats.coverage_hours / 24.0,
            Err(_) => 0.0,
        }
    }

    /// Format timestamp as human-readable
    pub fn format_timestamp(ts: u64) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(ts as i64, 0)
            .unwrap_or_default();
        dt.format("%Y-%m-%d %H:%M").to_string()
    }

    // ========================================================================
    // PHASE 9: Time-windowed per-object telemetry (v7.3.0)
    // ========================================================================

    /// Get usage stats for a specific time window
    pub fn get_usage_stats_window(&self, name: &str, window_secs: u64) -> Result<UsageStats> {
        let now = Self::now();
        let since = now.saturating_sub(window_secs);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                COUNT(*) as cnt,
                AVG(cpu_percent) as avg_cpu,
                MAX(cpu_percent) as peak_cpu,
                AVG(mem_bytes) as avg_mem,
                MAX(mem_bytes) as peak_mem,
                MIN(timestamp) as first_ts,
                MAX(timestamp) as last_ts
            FROM process_samples
            WHERE name = ?1 AND timestamp >= ?2
            "#
        )?;

        stmt.query_row(params![name, since as i64], |row| {
            let cnt: i64 = row.get(0).unwrap_or(0);
            let first_ts: i64 = row.get(5).unwrap_or(0);
            let last_ts: i64 = row.get(6).unwrap_or(0);

            // Check if we have at least 10 minutes of data
            let duration_minutes = if last_ts > first_ts {
                (last_ts - first_ts) as f64 / 60.0
            } else {
                0.0
            };

            Ok(UsageStats {
                sample_count: cnt as u64,
                avg_cpu_percent: row.get::<_, f64>(1).unwrap_or(0.0) as f32,
                peak_cpu_percent: row.get::<_, f64>(2).unwrap_or(0.0) as f32,
                avg_mem_bytes: row.get::<_, f64>(3).unwrap_or(0.0) as u64,
                peak_mem_bytes: row.get::<_, i64>(4).unwrap_or(0) as u64,
                has_enough_data: duration_minutes >= 10.0 || cnt >= 40,
            })
        }).map_err(|e| e.into())
    }

    /// Get multi-window stats for an object (1h, 24h, 7d, 30d)
    pub fn get_windowed_stats(&self, name: &str) -> Result<WindowedStats> {
        Ok(WindowedStats {
            last_1h: self.get_usage_stats_window(name, WINDOW_1H)?,
            last_24h: self.get_usage_stats_window(name, WINDOW_24H)?,
            last_7d: self.get_usage_stats_window(name, WINDOW_7D)?,
            last_30d: self.get_usage_stats_window(name, WINDOW_30D)?,
        })
    }

    /// Get launch count (number of distinct sessions) in a window
    pub fn get_launch_count(&self, name: &str, window_secs: u64) -> Result<u64> {
        let now = Self::now();
        let since = now.saturating_sub(window_secs);

        // Count distinct PIDs as proxy for launches
        let mut stmt = self.conn.prepare(
            r#"
            SELECT COUNT(DISTINCT pid)
            FROM process_samples
            WHERE name = ?1 AND timestamp >= ?2
            "#
        )?;

        let count: i64 = stmt.query_row(params![name, since as i64], |row| row.get(0))?;
        Ok(count as u64)
    }

    /// Get windowed launch counts for an object
    pub fn get_windowed_launches(&self, name: &str) -> Result<WindowedLaunches> {
        Ok(WindowedLaunches {
            last_1h: self.get_launch_count(name, WINDOW_1H)?,
            last_24h: self.get_launch_count(name, WINDOW_24H)?,
            last_7d: self.get_launch_count(name, WINDOW_7D)?,
            last_30d: self.get_launch_count(name, WINDOW_30D)?,
        })
    }
}

// ============================================================================
// PHASE 9: Time window constants
// ============================================================================

/// 1 hour window in seconds
pub const WINDOW_1H: u64 = 3600;

/// 24 hours window in seconds
pub const WINDOW_24H: u64 = 24 * 3600;

/// 7 days window in seconds
pub const WINDOW_7D: u64 = 7 * 24 * 3600;

/// 30 days window in seconds
pub const WINDOW_30D: u64 = 30 * 24 * 3600;

/// Multi-window usage statistics
#[derive(Debug, Clone, Default)]
pub struct WindowedStats {
    /// Stats for last 1 hour
    pub last_1h: UsageStats,
    /// Stats for last 24 hours
    pub last_24h: UsageStats,
    /// Stats for last 7 days
    pub last_7d: UsageStats,
    /// Stats for last 30 days
    pub last_30d: UsageStats,
}

/// Multi-window launch counts
#[derive(Debug, Clone, Default)]
pub struct WindowedLaunches {
    /// Launches in last 1 hour
    pub last_1h: u64,
    /// Launches in last 24 hours
    pub last_24h: u64,
    /// Launches in last 7 days
    pub last_7d: u64,
    /// Launches in last 30 days
    pub last_30d: u64,
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
