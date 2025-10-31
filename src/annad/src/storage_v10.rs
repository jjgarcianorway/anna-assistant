// Anna v0.10 Storage Module
// SQLite persistence + in-memory ring buffer for telemetry

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::telemetry_v10::*;

const RING_BUFFER_SIZE: usize = 60; // Last 60 samples (~30 minutes at 30s interval)

/// Storage manager with SQLite backend and ring buffer
pub struct StorageManager {
    conn: Arc<Mutex<Connection>>,
    ring_buffer: Arc<Mutex<VecDeque<TelemetrySnapshot>>>,
}

impl StorageManager {
    /// Create or open database at the given path
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path.as_ref())
            .context("Failed to open telemetry database")?;

        // Initialize schema
        Self::init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            ring_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(RING_BUFFER_SIZE))),
        })
    }

    /// Initialize SQLite schema
    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            -- Core snapshot metadata
            CREATE TABLE IF NOT EXISTS snapshot (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts INTEGER NOT NULL UNIQUE,
                host_id TEXT NOT NULL,
                kernel TEXT,
                distro TEXT,
                uptime_s INTEGER
            );

            -- CPU metrics (per-core)
            CREATE TABLE IF NOT EXISTS cpu (
                ts INTEGER NOT NULL,
                core INTEGER NOT NULL,
                util_pct REAL,
                temp_c REAL,
                PRIMARY KEY (ts, core)
            );

            -- Memory metrics
            CREATE TABLE IF NOT EXISTS mem (
                ts INTEGER PRIMARY KEY,
                total_mb INTEGER,
                used_mb INTEGER,
                free_mb INTEGER,
                cached_mb INTEGER,
                swap_total_mb INTEGER,
                swap_used_mb INTEGER
            );

            -- Disk metrics (per mount)
            CREATE TABLE IF NOT EXISTS disk (
                ts INTEGER NOT NULL,
                mount TEXT NOT NULL,
                device TEXT,
                fstype TEXT,
                total_gb REAL,
                used_gb REAL,
                pct REAL,
                inodes_pct REAL,
                read_iops INTEGER,
                write_iops INTEGER,
                PRIMARY KEY (ts, mount)
            );

            -- Network metrics (per interface)
            CREATE TABLE IF NOT EXISTS net (
                ts INTEGER NOT NULL,
                iface TEXT NOT NULL,
                rx_kbps REAL,
                tx_kbps REAL,
                link_state TEXT,
                ipv4_redacted TEXT,
                ipv6_prefix TEXT,
                mac_hash TEXT,
                rssi_dbm INTEGER,
                ssid_hash TEXT,
                vpn_flag INTEGER,
                PRIMARY KEY (ts, iface)
            );

            -- Power/Battery metrics
            CREATE TABLE IF NOT EXISTS power (
                ts INTEGER PRIMARY KEY,
                percent INTEGER,
                status TEXT,
                on_ac_bool INTEGER,
                time_to_empty_min INTEGER,
                time_to_full_min INTEGER,
                power_now_w REAL
            );

            -- GPU metrics
            CREATE TABLE IF NOT EXISTS gpu (
                ts INTEGER NOT NULL,
                device_id TEXT NOT NULL,
                util_pct REAL,
                temp_c REAL,
                mem_used_mb INTEGER,
                mem_total_mb INTEGER,
                PRIMARY KEY (ts, device_id)
            );

            -- Process snapshots (top N)
            CREATE TABLE IF NOT EXISTS process (
                ts INTEGER NOT NULL,
                pid INTEGER NOT NULL,
                name TEXT,
                cpu_pct REAL,
                mem_mb REAL,
                state TEXT,
                PRIMARY KEY (ts, pid)
            );

            -- Systemd unit states
            CREATE TABLE IF NOT EXISTS systemd_unit (
                ts INTEGER NOT NULL,
                unit TEXT NOT NULL,
                load TEXT,
                active TEXT,
                sub TEXT,
                PRIMARY KEY (ts, unit)
            );

            -- Alert/anomaly log
            CREATE TABLE IF NOT EXISTS alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts INTEGER NOT NULL,
                level TEXT,
                component TEXT,
                message TEXT
            );

            -- Persona scores
            CREATE TABLE IF NOT EXISTS persona_scores (
                ts INTEGER NOT NULL,
                name TEXT NOT NULL,
                score REAL,
                evidence_json TEXT,
                PRIMARY KEY (ts, name)
            );

            -- Inventory (updated on change only)
            CREATE TABLE IF NOT EXISTS inventory (
                ts INTEGER PRIMARY KEY,
                cpu_model TEXT,
                cpu_cores INTEGER,
                ram_gb INTEGER,
                distro TEXT,
                kernel TEXT,
                devices_json TEXT
            );

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_snapshot_ts ON snapshot(ts);
            CREATE INDEX IF NOT EXISTS idx_cpu_ts ON cpu(ts);
            CREATE INDEX IF NOT EXISTS idx_mem_ts ON mem(ts);
            CREATE INDEX IF NOT EXISTS idx_disk_ts ON disk(ts);
            CREATE INDEX IF NOT EXISTS idx_net_ts ON net(ts);
            CREATE INDEX IF NOT EXISTS idx_power_ts ON power(ts);
            CREATE INDEX IF NOT EXISTS idx_gpu_ts ON gpu(ts);
            CREATE INDEX IF NOT EXISTS idx_process_ts ON process(ts);
            CREATE INDEX IF NOT EXISTS idx_alerts_ts ON alerts(ts);
            CREATE INDEX IF NOT EXISTS idx_persona_ts ON persona_scores(ts);
            "#,
        )?;

        Ok(())
    }

    /// Store a telemetry snapshot (both to SQLite and ring buffer)
    pub fn store_snapshot(&self, snapshot: &TelemetrySnapshot) -> Result<()> {
        // Add to ring buffer
        {
            let mut ring = self.ring_buffer.lock().unwrap();
            ring.push_back(snapshot.clone());
            if ring.len() > RING_BUFFER_SIZE {
                ring.pop_front();
            }
        }

        // Persist to SQLite
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        // Insert snapshot metadata
        tx.execute(
            "INSERT OR REPLACE INTO snapshot (ts, host_id, kernel, distro, uptime_s) VALUES (?, ?, ?, ?, ?)",
            params![
                snapshot.ts,
                snapshot.host_id,
                snapshot.kernel,
                snapshot.distro,
                snapshot.uptime_s
            ],
        )?;

        // Insert CPU metrics
        for core in &snapshot.cpu.cores {
            tx.execute(
                "INSERT OR REPLACE INTO cpu (ts, core, util_pct, temp_c) VALUES (?, ?, ?, ?)",
                params![snapshot.ts, core.core, core.util_pct, core.temp_c],
            )?;
        }

        // Insert memory metrics
        tx.execute(
            "INSERT OR REPLACE INTO mem (ts, total_mb, used_mb, free_mb, cached_mb, swap_total_mb, swap_used_mb) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                snapshot.ts,
                snapshot.mem.total_mb,
                snapshot.mem.used_mb,
                snapshot.mem.free_mb,
                snapshot.mem.cached_mb,
                snapshot.mem.swap_total_mb,
                snapshot.mem.swap_used_mb
            ],
        )?;

        // Insert disk metrics
        for disk in &snapshot.disk {
            tx.execute(
                "INSERT OR REPLACE INTO disk (ts, mount, device, fstype, total_gb, used_gb, pct, inodes_pct, read_iops, write_iops) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    snapshot.ts,
                    disk.mount,
                    disk.device,
                    disk.fstype,
                    disk.total_gb,
                    disk.used_gb,
                    disk.pct,
                    disk.inodes_pct,
                    disk.read_iops,
                    disk.write_iops
                ],
            )?;
        }

        // Insert network metrics
        for net in &snapshot.net {
            tx.execute(
                "INSERT OR REPLACE INTO net (ts, iface, rx_kbps, tx_kbps, link_state, ipv4_redacted, ipv6_prefix, mac_hash, rssi_dbm, ssid_hash, vpn_flag) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    snapshot.ts,
                    net.iface,
                    net.rx_kbps,
                    net.tx_kbps,
                    net.link_state,
                    net.ipv4_redacted,
                    net.ipv6_prefix,
                    net.mac_hash,
                    net.rssi_dbm,
                    net.ssid_hash,
                    net.vpn_flag as i32
                ],
            )?;
        }

        // Insert power metrics (if available)
        if let Some(power) = &snapshot.power {
            tx.execute(
                "INSERT OR REPLACE INTO power (ts, percent, status, on_ac_bool, time_to_empty_min, time_to_full_min, power_now_w) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    snapshot.ts,
                    power.percent,
                    power.status,
                    power.on_ac_bool as i32,
                    power.time_to_empty_min,
                    power.time_to_full_min,
                    power.power_now_w
                ],
            )?;
        }

        // Insert GPU metrics
        for gpu in &snapshot.gpu {
            tx.execute(
                "INSERT OR REPLACE INTO gpu (ts, device_id, util_pct, temp_c, mem_used_mb, mem_total_mb) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    snapshot.ts,
                    gpu.device_id,
                    gpu.util_pct,
                    gpu.temp_c,
                    gpu.mem_used_mb,
                    gpu.mem_total_mb
                ],
            )?;
        }

        // Insert process metrics
        for proc in &snapshot.processes {
            tx.execute(
                "INSERT OR REPLACE INTO process (ts, pid, name, cpu_pct, mem_mb, state) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    snapshot.ts,
                    proc.pid,
                    proc.name,
                    proc.cpu_pct,
                    proc.mem_mb,
                    proc.state
                ],
            )?;
        }

        // Insert systemd unit metrics
        for unit in &snapshot.systemd_units {
            tx.execute(
                "INSERT OR REPLACE INTO systemd_unit (ts, unit, load, active, sub) VALUES (?, ?, ?, ?, ?)",
                params![
                    snapshot.ts,
                    unit.unit,
                    unit.load,
                    unit.active,
                    unit.sub
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get the latest snapshot from ring buffer
    pub fn get_latest_snapshot(&self) -> Option<TelemetrySnapshot> {
        let ring = self.ring_buffer.lock().unwrap();
        ring.back().cloned()
    }

    /// Get N most recent snapshots from ring buffer
    pub fn get_recent_snapshots(&self, n: usize) -> Vec<TelemetrySnapshot> {
        let ring = self.ring_buffer.lock().unwrap();
        ring.iter().rev().take(n).cloned().collect()
    }

    /// Query historical snapshots from SQLite (last N minutes)
    pub fn query_history(&self, window_min: u32) -> Result<Vec<TelemetrySnapshot>> {
        let conn = self.conn.lock().unwrap();
        let cutoff_ts = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() - (window_min as u64 * 60)) as i64;

        let mut stmt = conn.prepare(
            "SELECT ts, host_id, kernel, distro, uptime_s FROM snapshot WHERE ts >= ? ORDER BY ts ASC"
        )?;

        let mut rows = stmt.query(params![cutoff_ts])?;
        let mut snapshots = Vec::new();

        while let Some(row) = rows.next()? {
            let ts: u64 = row.get(0)?;

            // Build partial snapshot (metadata only for now)
            let snapshot = TelemetrySnapshot {
                ts,
                host_id: row.get(1)?,
                kernel: row.get(2)?,
                distro: row.get(3)?,
                uptime_s: row.get(4)?,
                cpu: self.query_cpu_metrics(&conn, ts)?,
                mem: self.query_mem_metrics(&conn, ts)?,
                disk: self.query_disk_metrics(&conn, ts)?,
                net: self.query_net_metrics(&conn, ts)?,
                power: self.query_power_metrics(&conn, ts).ok(),
                gpu: self.query_gpu_metrics(&conn, ts)?,
                processes: self.query_process_metrics(&conn, ts)?,
                systemd_units: self.query_systemd_units(&conn, ts)?,
            };

            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }

    fn query_cpu_metrics(&self, conn: &Connection, ts: u64) -> Result<CpuMetrics> {
        let mut stmt = conn.prepare("SELECT core, util_pct, temp_c FROM cpu WHERE ts = ? ORDER BY core")?;
        let cores: Vec<CpuCore> = stmt
            .query_map(params![ts], |row| {
                Ok(CpuCore {
                    core: row.get(0)?,
                    util_pct: row.get(1)?,
                    temp_c: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(CpuMetrics {
            cores,
            load_avg: [0.0, 0.0, 0.0], // Not stored in DB yet
            throttle_flags: Vec::new(),
        })
    }

    fn query_mem_metrics(&self, conn: &Connection, ts: u64) -> Result<MemMetrics> {
        let row = conn.query_row(
            "SELECT total_mb, used_mb, free_mb, cached_mb, swap_total_mb, swap_used_mb FROM mem WHERE ts = ?",
            params![ts],
            |row| {
                Ok(MemMetrics {
                    total_mb: row.get(0)?,
                    used_mb: row.get(1)?,
                    free_mb: row.get(2)?,
                    cached_mb: row.get(3)?,
                    swap_total_mb: row.get(4)?,
                    swap_used_mb: row.get(5)?,
                })
            },
        )?;

        Ok(row)
    }

    fn query_disk_metrics(&self, conn: &Connection, ts: u64) -> Result<Vec<DiskMetrics>> {
        let mut stmt = conn.prepare(
            "SELECT mount, device, fstype, total_gb, used_gb, pct, inodes_pct, read_iops, write_iops FROM disk WHERE ts = ?"
        )?;

        let disks: Vec<DiskMetrics> = stmt
            .query_map(params![ts], |row| {
                Ok(DiskMetrics {
                    mount: row.get(0)?,
                    device: row.get(1)?,
                    fstype: row.get(2)?,
                    total_gb: row.get(3)?,
                    used_gb: row.get(4)?,
                    pct: row.get(5)?,
                    inodes_pct: row.get(6)?,
                    read_iops: row.get(7)?,
                    write_iops: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(disks)
    }

    fn query_net_metrics(&self, conn: &Connection, ts: u64) -> Result<Vec<NetMetrics>> {
        let mut stmt = conn.prepare(
            "SELECT iface, rx_kbps, tx_kbps, link_state, ipv4_redacted, ipv6_prefix, mac_hash, rssi_dbm, ssid_hash, vpn_flag FROM net WHERE ts = ?"
        )?;

        let nets: Vec<NetMetrics> = stmt
            .query_map(params![ts], |row| {
                Ok(NetMetrics {
                    iface: row.get(0)?,
                    rx_kbps: row.get(1)?,
                    tx_kbps: row.get(2)?,
                    link_state: row.get(3)?,
                    ipv4_redacted: row.get(4)?,
                    ipv6_prefix: row.get(5)?,
                    mac_hash: row.get(6)?,
                    rssi_dbm: row.get(7)?,
                    ssid_hash: row.get(8)?,
                    vpn_flag: row.get::<_, i32>(9)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(nets)
    }

    fn query_power_metrics(&self, conn: &Connection, ts: u64) -> Result<PowerMetrics> {
        let row = conn.query_row(
            "SELECT percent, status, on_ac_bool, time_to_empty_min, time_to_full_min, power_now_w FROM power WHERE ts = ?",
            params![ts],
            |row| {
                Ok(PowerMetrics {
                    percent: row.get(0)?,
                    status: row.get(1)?,
                    on_ac_bool: row.get::<_, i32>(2)? != 0,
                    time_to_empty_min: row.get(3)?,
                    time_to_full_min: row.get(4)?,
                    power_now_w: row.get(5)?,
                })
            },
        ).optional()?.context("Power metrics not found")?;

        Ok(row)
    }

    fn query_gpu_metrics(&self, conn: &Connection, ts: u64) -> Result<Vec<GpuMetrics>> {
        let mut stmt = conn.prepare(
            "SELECT device_id, util_pct, temp_c, mem_used_mb, mem_total_mb FROM gpu WHERE ts = ?"
        )?;

        let gpus: Vec<GpuMetrics> = stmt
            .query_map(params![ts], |row| {
                Ok(GpuMetrics {
                    device_id: row.get(0)?,
                    util_pct: row.get(1)?,
                    temp_c: row.get(2)?,
                    mem_used_mb: row.get(3)?,
                    mem_total_mb: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(gpus)
    }

    fn query_process_metrics(&self, conn: &Connection, ts: u64) -> Result<Vec<ProcessMetrics>> {
        let mut stmt = conn.prepare(
            "SELECT pid, name, cpu_pct, mem_mb, state FROM process WHERE ts = ? ORDER BY cpu_pct DESC"
        )?;

        let procs: Vec<ProcessMetrics> = stmt
            .query_map(params![ts], |row| {
                Ok(ProcessMetrics {
                    pid: row.get(0)?,
                    name: row.get(1)?,
                    cpu_pct: row.get(2)?,
                    mem_mb: row.get(3)?,
                    state: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(procs)
    }

    fn query_systemd_units(&self, conn: &Connection, ts: u64) -> Result<Vec<SystemdUnit>> {
        let mut stmt = conn.prepare(
            "SELECT unit, load, active, sub FROM systemd_unit WHERE ts = ?"
        )?;

        let units: Vec<SystemdUnit> = stmt
            .query_map(params![ts], |row| {
                Ok(SystemdUnit {
                    unit: row.get(0)?,
                    load: row.get(1)?,
                    active: row.get(2)?,
                    sub: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(units)
    }

    /// Store persona scores
    pub fn store_persona_scores(&self, ts: u64, scores: &[(String, f32, Vec<String>)]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        for (name, score, evidence) in scores {
            let evidence_json = serde_json::to_string(evidence)?;
            tx.execute(
                "INSERT OR REPLACE INTO persona_scores (ts, name, score, evidence_json) VALUES (?, ?, ?, ?)",
                params![ts, name, score, evidence_json],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Query latest persona scores
    pub fn query_latest_persona_scores(&self) -> Result<Vec<(String, f32, Vec<String>)>> {
        let conn = self.conn.lock().unwrap();

        // Get latest timestamp
        let latest_ts: Option<u64> = conn
            .query_row("SELECT MAX(ts) FROM persona_scores", [], |row| row.get(0))
            .optional()?
            .flatten();

        if let Some(ts) = latest_ts {
            let mut stmt = conn.prepare(
                "SELECT name, score, evidence_json FROM persona_scores WHERE ts = ?"
            )?;

            let scores: Vec<(String, f32, Vec<String>)> = stmt
                .query_map(params![ts], |row| {
                    let name: String = row.get(0)?;
                    let score: f32 = row.get(1)?;
                    let evidence_json: String = row.get(2)?;
                    let evidence: Vec<String> = serde_json::from_str(&evidence_json)
                        .unwrap_or_else(|_| Vec::new());

                    Ok((name, score, evidence))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(scores)
        } else {
            Ok(Vec::new())
        }
    }

    /// Log an alert
    pub fn log_alert(&self, level: &str, component: &str, message: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        conn.execute(
            "INSERT INTO alerts (ts, level, component, message) VALUES (?, ?, ?, ?)",
            params![ts, level, component, message],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_storage_init() -> Result<()> {
        let temp_db = NamedTempFile::new()?;
        let storage = StorageManager::new(temp_db.path())?;

        // Verify tables exist
        let conn = storage.conn.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        )?;

        assert!(count >= 10); // Should have at least 10 tables

        Ok(())
    }

    #[test]
    fn test_ring_buffer() -> Result<()> {
        let temp_db = NamedTempFile::new()?;
        let storage = StorageManager::new(temp_db.path())?;

        // Create test snapshot
        let snapshot = TelemetrySnapshot {
            ts: 1730390400,
            host_id: "test".to_string(),
            kernel: "6.0.0".to_string(),
            distro: "Test OS".to_string(),
            uptime_s: 3600,
            cpu: CpuMetrics {
                cores: vec![CpuCore {
                    core: 0,
                    util_pct: 50.0,
                    temp_c: Some(42.0),
                }],
                load_avg: [1.0, 1.0, 1.0],
                throttle_flags: Vec::new(),
            },
            mem: MemMetrics {
                total_mb: 16384,
                used_mb: 8192,
                free_mb: 8192,
                cached_mb: 2048,
                swap_total_mb: 4096,
                swap_used_mb: 512,
            },
            disk: Vec::new(),
            net: Vec::new(),
            power: None,
            gpu: Vec::new(),
            processes: Vec::new(),
            systemd_units: Vec::new(),
        };

        storage.store_snapshot(&snapshot)?;

        let latest = storage.get_latest_snapshot();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().ts, 1730390400);

        Ok(())
    }
}
