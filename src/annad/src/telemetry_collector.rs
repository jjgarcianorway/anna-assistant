//! Telemetry Collection System
//!
//! Collects system metrics (CPU, RAM, disk, uptime, network) every 60 seconds
//! and stores them persistently in SQLite for analysis and policy evaluation.

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use sysinfo::{System, Networks, Disks};
use tokio::time::{interval, Duration};

/// Telemetry sample containing system metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub timestamp: String,
    pub cpu_usage: f32,
    pub mem_usage: f32,
    pub disk_free: f32,
    pub uptime_sec: u64,
    pub net_in_kb: u64,
    pub net_out_kb: u64,
}

/// Telemetry collector with persistent storage
pub struct TelemetryCollector {
    db_path: String,
    conn: Arc<Mutex<Connection>>,
}

impl TelemetryCollector {
    /// Create new telemetry collector with SQLite storage
    pub fn new(db_path: &str) -> Result<Self> {
        // Create database directory if it doesn't exist
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)
            .context("Failed to open telemetry database")?;

        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS telemetry (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                cpu REAL,
                mem REAL,
                disk REAL,
                uptime INTEGER,
                net_in INTEGER,
                net_out INTEGER
            )",
            [],
        )?;

        // Create index on timestamp for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON telemetry(timestamp DESC)",
            [],
        )?;

        Ok(Self {
            db_path: db_path.to_string(),
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Collect current system metrics
    pub fn collect_sample() -> Result<TelemetrySample> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // CPU usage (global average)
        let cpu_usage = sys.global_cpu_info().cpu_usage();

        // Memory usage percentage
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let mem_usage = if total_mem > 0 {
            (used_mem as f64 / total_mem as f64 * 100.0) as f32
        } else {
            0.0
        };

        // Disk usage (first disk or 0 if none)
        let disks = Disks::new_with_refreshed_list();
        let disk_free = if let Some(disk) = disks.list().first() {
            let total = disk.total_space();
            let available = disk.available_space();
            if total > 0 {
                (available as f64 / total as f64 * 100.0) as f32
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Uptime
        let uptime_sec = System::uptime();

        // Network throughput (sum of all interfaces)
        let networks = Networks::new_with_refreshed_list();
        let (mut net_in, mut net_out) = (0u64, 0u64);
        for (_name, network) in networks.list() {
            net_in += network.total_received();
            net_out += network.total_transmitted();
        }

        // Convert bytes to KB
        let net_in_kb = net_in / 1024;
        let net_out_kb = net_out / 1024;

        let timestamp = chrono::Local::now().to_rfc3339();

        Ok(TelemetrySample {
            timestamp,
            cpu_usage,
            mem_usage,
            disk_free,
            uptime_sec,
            net_in_kb,
            net_out_kb,
        })
    }

    /// Store telemetry sample in database
    pub fn store_sample(&self, sample: &TelemetrySample) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO telemetry (timestamp, cpu, mem, disk, uptime, net_in, net_out)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &sample.timestamp,
                sample.cpu_usage,
                sample.mem_usage,
                sample.disk_free,
                sample.uptime_sec as i64,
                sample.net_in_kb as i64,
                sample.net_out_kb as i64,
            ],
        )?;

        Ok(())
    }

    /// Get most recent telemetry sample
    pub fn get_snapshot(&self) -> Result<Option<TelemetrySample>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT timestamp, cpu, mem, disk, uptime, net_in, net_out
             FROM telemetry
             ORDER BY id DESC
             LIMIT 1"
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(TelemetrySample {
                timestamp: row.get(0)?,
                cpu_usage: row.get(1)?,
                mem_usage: row.get(2)?,
                disk_free: row.get(3)?,
                uptime_sec: row.get::<_, i64>(4)? as u64,
                net_in_kb: row.get::<_, i64>(5)? as u64,
                net_out_kb: row.get::<_, i64>(6)? as u64,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get telemetry history (last N samples)
    pub fn get_history(&self, limit: usize) -> Result<Vec<TelemetrySample>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT timestamp, cpu, mem, disk, uptime, net_in, net_out
             FROM telemetry
             ORDER BY id DESC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map([limit], |row| {
            Ok(TelemetrySample {
                timestamp: row.get(0)?,
                cpu_usage: row.get(1)?,
                mem_usage: row.get(2)?,
                disk_free: row.get(3)?,
                uptime_sec: row.get::<_, i64>(4)? as u64,
                net_in_kb: row.get::<_, i64>(5)? as u64,
                net_out_kb: row.get::<_, i64>(6)? as u64,
            })
        })?;

        let mut samples = Vec::new();
        for row in rows {
            samples.push(row?);
        }

        Ok(samples)
    }

    /// Get telemetry trends for a specific metric
    pub fn get_trends(&self, metric: &str, hours: usize) -> Result<TelemetryTrends> {
        let conn = self.conn.lock().unwrap();

        // Calculate time cutoff
        let cutoff = chrono::Local::now() - chrono::Duration::hours(hours as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let column = match metric {
            "cpu" => "cpu",
            "mem" | "memory" => "mem",
            "disk" => "disk",
            _ => return Err(anyhow::anyhow!("Unknown metric: {}", metric)),
        };

        let query = format!(
            "SELECT AVG({0}), MIN({0}), MAX({0}), COUNT(*)
             FROM telemetry
             WHERE timestamp >= ?1",
            column
        );

        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([&cutoff_str])?;

        if let Some(row) = rows.next()? {
            let count: i64 = row.get(3)?;
            if count > 0 {
                Ok(TelemetryTrends {
                    metric: metric.to_string(),
                    hours,
                    avg: row.get(0)?,
                    min: row.get(1)?,
                    max: row.get(2)?,
                    samples: count as usize,
                })
            } else {
                Ok(TelemetryTrends {
                    metric: metric.to_string(),
                    hours,
                    avg: 0.0,
                    min: 0.0,
                    max: 0.0,
                    samples: 0,
                })
            }
        } else {
            Ok(TelemetryTrends {
                metric: metric.to_string(),
                hours,
                avg: 0.0,
                min: 0.0,
                max: 0.0,
                samples: 0,
            })
        }
    }

    /// Start background collection loop (runs every 60 seconds)
    pub fn start_collection_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                match Self::collect_sample() {
                    Ok(sample) => {
                        tracing::debug!(
                            "Telemetry collected: CPU {:.1}%, MEM {:.1}%, DISK {:.1}%",
                            sample.cpu_usage,
                            sample.mem_usage,
                            sample.disk_free
                        );

                        if let Err(e) = self.store_sample(&sample) {
                            tracing::error!("Failed to store telemetry sample: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to collect telemetry sample: {}", e);
                    }
                }
            }
        });
    }
}

/// Telemetry trends analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryTrends {
    pub metric: String,
    pub hours: usize,
    pub avg: f32,
    pub min: f32,
    pub max: f32,
    pub samples: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_sample() {
        let sample = TelemetryCollector::collect_sample().unwrap();
        assert!(sample.cpu_usage >= 0.0);
        assert!(sample.mem_usage >= 0.0);
        assert!(!sample.timestamp.is_empty());
    }

    #[test]
    fn test_database_storage() {
        let temp_db = "/tmp/anna_telemetry_test.db";
        let _ = std::fs::remove_file(temp_db);

        let collector = TelemetryCollector::new(temp_db).unwrap();
        let sample = TelemetryCollector::collect_sample().unwrap();

        collector.store_sample(&sample).unwrap();

        let snapshot = collector.get_snapshot().unwrap();
        assert!(snapshot.is_some());

        let history = collector.get_history(10).unwrap();
        assert_eq!(history.len(), 1);

        std::fs::remove_file(temp_db).unwrap();
    }
}
