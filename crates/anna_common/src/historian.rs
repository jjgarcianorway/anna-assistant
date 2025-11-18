use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Historian - Anna's long-term memory and trend analysis system
///
/// This module tracks system metrics over time, enabling Anna to:
/// - Identify trends, degradations, and improvements
/// - Answer "when did this start" and "what changed before it broke"
/// - Measure effectiveness of repairs and optimizations
/// - Maintain performance baselines and detect deviations
///
/// Database location: /var/lib/anna/historian.db

// ============================================================================
// Database Schema Constants
// ============================================================================

pub const SCHEMA_VERSION: i32 = 1;

pub const SCHEMA_SQL: &str = r#"
-- Historian database schema v1
-- Stores time-series data, events, baselines, and aggregates

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

-- ============================================================================
-- 1. Global Timeline & Events
-- ============================================================================

-- System installation and version history
CREATE TABLE IF NOT EXISTS system_timeline (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL, -- 'install', 'upgrade', 'rollback', 'kernel_change', 'config_migration'
    timestamp TEXT NOT NULL,
    version_from TEXT,
    version_to TEXT,
    kernel_from TEXT,
    kernel_to TEXT,
    metadata TEXT, -- JSON blob for additional context
    notes TEXT
);

-- Self-repair and suggestion tracking
CREATE TABLE IF NOT EXISTS repair_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    trigger_type TEXT NOT NULL, -- 'health_check', 'user_request', 'startup_check'
    action_type TEXT NOT NULL, -- 'package_cleanup', 'config_fix', 'service_restart', etc.
    actions_taken TEXT NOT NULL, -- JSON array of specific actions
    metrics_before TEXT, -- JSON snapshot of relevant metrics
    metrics_after TEXT, -- JSON snapshot after repair
    success BOOLEAN NOT NULL,
    regressed BOOLEAN DEFAULT 0,
    user_feedback TEXT, -- 'helpful', 'no_change', 'made_worse', null
    notes TEXT
);

-- ============================================================================
-- 2. Boot/Shutdown Analysis
-- ============================================================================

-- Per-boot event log
CREATE TABLE IF NOT EXISTS boot_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    boot_id TEXT NOT NULL UNIQUE, -- systemd boot ID
    boot_timestamp TEXT NOT NULL,
    shutdown_timestamp TEXT,
    boot_duration_ms INTEGER,
    shutdown_duration_ms INTEGER,
    target_reached TEXT, -- 'graphical.target', 'multi-user.target'
    time_to_target_ms INTEGER,
    slowest_units TEXT, -- JSON array of {unit, duration_ms}
    failed_units TEXT, -- JSON array of failed unit names
    degraded_units TEXT, -- JSON array of degraded unit names
    fsck_triggered BOOLEAN DEFAULT 0,
    fsck_duration_ms INTEGER,
    kernel_errors TEXT, -- JSON array of early boot kernel errors
    boot_health_score INTEGER -- 0-100
);

-- Boot time aggregates (daily summaries)
CREATE TABLE IF NOT EXISTS boot_aggregates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE, -- YYYY-MM-DD
    boot_count INTEGER NOT NULL,
    avg_boot_duration_ms INTEGER,
    min_boot_duration_ms INTEGER,
    max_boot_duration_ms INTEGER,
    failed_boot_count INTEGER DEFAULT 0,
    avg_health_score INTEGER
);

-- ============================================================================
-- 3. CPU Usage Trends
-- ============================================================================

-- Hourly CPU samples
CREATE TABLE IF NOT EXISTS cpu_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    avg_utilization_percent REAL NOT NULL,
    peak_utilization_percent REAL NOT NULL,
    idle_background_percent REAL,
    throttle_event_count INTEGER DEFAULT 0,
    spike_count INTEGER DEFAULT 0, -- 100% spikes lasting >N seconds
    top_processes TEXT -- JSON array of {name, cpu_percent, cumulative_time_ms}
);

-- Daily CPU aggregates
CREATE TABLE IF NOT EXISTS cpu_aggregates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,
    avg_utilization_percent REAL NOT NULL,
    peak_utilization_percent REAL NOT NULL,
    total_throttle_events INTEGER DEFAULT 0,
    avg_idle_percent REAL
);

-- ============================================================================
-- 4. Memory & Swap Trends
-- ============================================================================

-- Hourly memory samples
CREATE TABLE IF NOT EXISTS memory_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    avg_ram_used_mb INTEGER NOT NULL,
    peak_ram_used_mb INTEGER NOT NULL,
    avg_swap_used_mb INTEGER DEFAULT 0,
    peak_swap_used_mb INTEGER DEFAULT 0,
    oom_kill_count INTEGER DEFAULT 0,
    oom_victims TEXT, -- JSON array of {process, timestamp}
    top_memory_hogs TEXT -- JSON array of {name, rss_mb}
);

-- Daily memory aggregates
CREATE TABLE IF NOT EXISTS memory_aggregates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,
    avg_ram_used_mb INTEGER NOT NULL,
    peak_ram_used_mb INTEGER NOT NULL,
    total_oom_kills INTEGER DEFAULT 0,
    swap_dependency_score INTEGER -- 0-100, how much system relies on swap
);

-- ============================================================================
-- 5. Disk Space & I/O Trends
-- ============================================================================

-- Daily disk space snapshots (per filesystem)
CREATE TABLE IF NOT EXISTS disk_space_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    filesystem TEXT NOT NULL, -- mount point
    total_gb REAL NOT NULL,
    used_gb REAL NOT NULL,
    free_gb REAL NOT NULL,
    used_percent REAL NOT NULL,
    inode_used_percent REAL
);

-- Directory growth tracking
CREATE TABLE IF NOT EXISTS directory_growth (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    filesystem TEXT NOT NULL,
    directory TEXT NOT NULL, -- e.g., '/home', '/var/log', '/var/cache'
    size_mb INTEGER NOT NULL,
    growth_rate_mb_per_day REAL
);

-- Hourly I/O statistics (per device)
CREATE TABLE IF NOT EXISTS io_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    device TEXT NOT NULL,
    avg_read_mb_per_sec REAL NOT NULL,
    avg_write_mb_per_sec REAL NOT NULL,
    avg_queue_depth REAL,
    avg_latency_ms REAL,
    high_latency_events INTEGER DEFAULT 0 -- count of >100ms latency events
);

-- ============================================================================
-- 6. Network Quality & Stability
-- ============================================================================

-- Hourly network quality samples
CREATE TABLE IF NOT EXISTS network_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    interface TEXT NOT NULL,
    target TEXT NOT NULL, -- IP or hostname being monitored
    avg_latency_ms REAL,
    packet_loss_percent REAL DEFAULT 0,
    disconnect_count INTEGER DEFAULT 0,
    dhcp_renew_failures INTEGER DEFAULT 0,
    dns_failures INTEGER DEFAULT 0
);

-- VPN connection tracking
CREATE TABLE IF NOT EXISTS vpn_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL, -- 'connect', 'disconnect', 'failure'
    vpn_name TEXT NOT NULL,
    duration_seconds INTEGER,
    disconnect_reason TEXT
);

-- ============================================================================
-- 7. Service & Daemon Reliability
-- ============================================================================

-- Per-service reliability tracking
CREATE TABLE IF NOT EXISTS service_reliability (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    service_name TEXT NOT NULL,
    event_type TEXT NOT NULL, -- 'start', 'stop', 'crash', 'restart', 'config_change'
    was_intentional BOOLEAN DEFAULT 1,
    start_duration_ms INTEGER,
    failed_state_duration_ms INTEGER,
    exit_code INTEGER,
    signal INTEGER,
    metadata TEXT -- JSON for additional context
);

-- Daily service aggregates
CREATE TABLE IF NOT EXISTS service_aggregates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    service_name TEXT NOT NULL,
    restart_count INTEGER DEFAULT 0,
    crash_count INTEGER DEFAULT 0,
    total_failed_time_ms INTEGER DEFAULT 0,
    avg_start_time_ms INTEGER,
    stability_score INTEGER, -- 0-100
    UNIQUE(date, service_name)
);

-- ============================================================================
-- 8. Error & Warning Statistics
-- ============================================================================

-- Unique error signatures
CREATE TABLE IF NOT EXISTS error_signatures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    signature_hash TEXT NOT NULL UNIQUE, -- hash of normalized error message
    source TEXT NOT NULL, -- 'kernel', 'systemd', 'application', service name
    severity TEXT NOT NULL, -- 'critical', 'error', 'warning'
    message_template TEXT NOT NULL, -- error message with variables normalized
    first_occurrence TEXT NOT NULL,
    last_occurrence TEXT NOT NULL,
    total_count INTEGER DEFAULT 1,
    disappeared_after_change TEXT -- ID of repair/upgrade that fixed it
);

-- Hourly error rate samples
CREATE TABLE IF NOT EXISTS error_rate_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    error_count INTEGER DEFAULT 0,
    warning_count INTEGER DEFAULT 0,
    critical_count INTEGER DEFAULT 0,
    new_signature_count INTEGER DEFAULT 0,
    top_sources TEXT -- JSON array of {source, count}
);

-- ============================================================================
-- 9. Performance Baselines
-- ============================================================================

-- Baseline snapshots (taken when system is "healthy")
CREATE TABLE IF NOT EXISTS baselines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    baseline_type TEXT NOT NULL, -- 'boot_time', 'idle_resources', 'workflow_X'
    timestamp TEXT NOT NULL,
    metrics TEXT NOT NULL, -- JSON snapshot of all relevant metrics
    hardware_config TEXT, -- JSON snapshot of CPU/RAM/GPU config
    notes TEXT,
    is_active BOOLEAN DEFAULT 1 -- only one active baseline per type
);

-- Performance delta tracking (before/after significant changes)
CREATE TABLE IF NOT EXISTS performance_deltas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    change_type TEXT NOT NULL, -- 'upgrade', 'repair', 'config_change', 'driver_update'
    change_id TEXT, -- reference to repair_history or system_timeline
    metric_name TEXT NOT NULL,
    value_before REAL,
    value_after REAL,
    delta_percent REAL,
    impact_description TEXT
);

-- ============================================================================
-- 10. User Behavior Patterns
-- ============================================================================

-- Daily usage patterns
CREATE TABLE IF NOT EXISTS usage_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,
    active_hours TEXT, -- JSON array of hours when user was active
    heavy_load_hours TEXT, -- JSON array of hours with heavy CPU/RAM usage
    common_applications TEXT, -- JSON array of {name, usage_count, total_time_ms}
    package_update_count INTEGER DEFAULT 0,
    anna_invocation_count INTEGER DEFAULT 0
);

-- Anomaly detection
CREATE TABLE IF NOT EXISTS usage_anomalies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    anomaly_type TEXT NOT NULL, -- 'unexpected_heavy_load', 'unusual_time', 'unknown_process'
    description TEXT NOT NULL,
    severity TEXT NOT NULL -- 'info', 'warning', 'critical'
);

-- ============================================================================
-- 11. LLM Statistics
-- ============================================================================

-- LLM performance tracking
CREATE TABLE IF NOT EXISTS llm_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    model_name TEXT NOT NULL,
    avg_latency_ms INTEGER,
    total_requests INTEGER DEFAULT 0,
    failed_requests INTEGER DEFAULT 0,
    avg_memory_mb INTEGER,
    avg_gpu_utilization_percent REAL,
    avg_cpu_utilization_percent REAL,
    avg_temperature_c REAL,
    max_temperature_c REAL
);

-- Model change history
CREATE TABLE IF NOT EXISTS llm_model_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL, -- 'install', 'upgrade', 'downgrade', 'switch'
    model_from TEXT,
    model_to TEXT NOT NULL,
    reason TEXT,
    hardware_requirements TEXT, -- JSON
    performance_impact TEXT -- JSON with before/after metrics
);

-- ============================================================================
-- 12. Synthesized Indicators (High-Level Scores)
-- ============================================================================

-- Daily system health scores
CREATE TABLE IF NOT EXISTS health_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,
    stability_score INTEGER NOT NULL, -- 0-100
    performance_score INTEGER NOT NULL, -- 0-100
    noise_score INTEGER NOT NULL, -- 0-100, how noisy logs are
    stability_trend TEXT NOT NULL, -- 'up', 'down', 'flat'
    performance_trend TEXT NOT NULL,
    noise_trend TEXT NOT NULL,
    last_major_regression TEXT, -- timestamp
    last_major_improvement TEXT, -- timestamp
    regression_cause TEXT,
    improvement_cause TEXT
);

-- Indices for common queries
CREATE INDEX IF NOT EXISTS idx_boot_timestamp ON boot_events(boot_timestamp);
CREATE INDEX IF NOT EXISTS idx_boot_date ON boot_aggregates(date);
CREATE INDEX IF NOT EXISTS idx_cpu_window ON cpu_samples(window_start, window_end);
CREATE INDEX IF NOT EXISTS idx_memory_window ON memory_samples(window_start, window_end);
CREATE INDEX IF NOT EXISTS idx_disk_timestamp ON disk_space_samples(timestamp, filesystem);
CREATE INDEX IF NOT EXISTS idx_network_window ON network_samples(window_start, window_end, interface);
CREATE INDEX IF NOT EXISTS idx_service_timestamp ON service_reliability(timestamp, service_name);
CREATE INDEX IF NOT EXISTS idx_error_signature ON error_signatures(signature_hash);
CREATE INDEX IF NOT EXISTS idx_health_date ON health_scores(date);
"#;

// ============================================================================
// Rust Data Structures
// ============================================================================

/// Global timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event_type: TimelineEventType,
    pub timestamp: DateTime<Utc>,
    pub version_from: Option<String>,
    pub version_to: Option<String>,
    pub kernel_from: Option<String>,
    pub kernel_to: Option<String>,
    pub metadata: serde_json::Value,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineEventType {
    Install,
    Upgrade,
    Rollback,
    KernelChange,
    ConfigMigration,
}

/// Repair/action tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairRecord {
    pub timestamp: DateTime<Utc>,
    pub trigger_type: RepairTrigger,
    pub action_type: String,
    pub actions_taken: Vec<String>,
    pub metrics_before: Option<serde_json::Value>,
    pub metrics_after: Option<serde_json::Value>,
    pub success: bool,
    pub regressed: bool,
    pub user_feedback: Option<UserFeedback>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairTrigger {
    HealthCheck,
    UserRequest,
    StartupCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserFeedback {
    Helpful,
    NoChange,
    MadeWorse,
}

/// Boot event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootEvent {
    pub boot_id: String,
    pub boot_timestamp: DateTime<Utc>,
    pub shutdown_timestamp: Option<DateTime<Utc>>,
    pub boot_duration_ms: Option<i64>,
    pub shutdown_duration_ms: Option<i64>,
    pub target_reached: String,
    pub time_to_target_ms: Option<i64>,
    pub slowest_units: Vec<SlowUnit>,
    pub failed_units: Vec<String>,
    pub degraded_units: Vec<String>,
    pub fsck_triggered: bool,
    pub fsck_duration_ms: Option<i64>,
    pub kernel_errors: Vec<String>,
    pub boot_health_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowUnit {
    pub unit: String,
    pub duration_ms: i64,
}

/// CPU usage sample (hourly window)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSample {
    pub timestamp: DateTime<Utc>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub avg_utilization_percent: f64,
    pub peak_utilization_percent: f64,
    pub idle_background_percent: Option<f64>,
    pub throttle_event_count: i32,
    pub spike_count: i32,
    pub top_processes: Vec<ProcessCpuInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCpuInfo {
    pub name: String,
    pub cpu_percent: f64,
    pub cumulative_time_ms: i64,
}

/// Memory usage sample (hourly window)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    pub timestamp: DateTime<Utc>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub avg_ram_used_mb: i64,
    pub peak_ram_used_mb: i64,
    pub avg_swap_used_mb: i64,
    pub peak_swap_used_mb: i64,
    pub oom_kill_count: i32,
    pub oom_victims: Vec<OomVictim>,
    pub top_memory_hogs: Vec<ProcessMemoryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OomVictim {
    pub process: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMemoryInfo {
    pub name: String,
    pub rss_mb: i64,
}

/// Performance baseline snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub baseline_type: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: serde_json::Value,
    pub hardware_config: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub is_active: bool,
}

/// Synthesized health scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub date: String,          // YYYY-MM-DD
    pub stability_score: u8,   // 0-100
    pub performance_score: u8, // 0-100
    pub noise_score: u8,       // 0-100
    pub stability_trend: Trend,
    pub performance_trend: Trend,
    pub noise_trend: Trend,
    pub last_major_regression: Option<DateTime<Utc>>,
    pub last_major_improvement: Option<DateTime<Utc>>,
    pub regression_cause: Option<String>,
    pub improvement_cause: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Up,
    Down,
    Flat,
}

impl std::fmt::Display for Trend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trend::Up => write!(f, "↑"),
            Trend::Down => write!(f, "↓"),
            Trend::Flat => write!(f, "→"),
        }
    }
}

// ============================================================================
// Implementation
// ============================================================================

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;

/// Main Historian database manager
pub struct Historian {
    conn: Connection,
}

impl Historian {
    /// Create a new Historian instance and initialize the database
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref()).context("Failed to open historian database")?;

        let mut historian = Self { conn };
        historian.init_schema()?;
        Ok(historian)
    }

    /// Initialize database schema
    fn init_schema(&mut self) -> Result<()> {
        // Execute the schema SQL
        self.conn
            .execute_batch(SCHEMA_SQL)
            .context("Failed to create database schema")?;

        // Check if we need to insert the schema version
        let version_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM schema_version WHERE version = ?1",
                [SCHEMA_VERSION],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !version_exists {
            self.conn.execute(
                "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
                rusqlite::params![SCHEMA_VERSION, Utc::now().to_rfc3339()],
            )?;
        }

        Ok(())
    }

    // ========================================================================
    // Timeline & Event Tracking
    // ========================================================================

    /// Record a system timeline event
    pub fn record_timeline_event(&self, event: &TimelineEvent) -> Result<i64> {
        let event_type = serde_json::to_string(&event.event_type)?;
        let metadata = serde_json::to_string(&event.metadata)?;

        let id = self.conn.execute(
            "INSERT INTO system_timeline (event_type, timestamp, version_from, version_to, kernel_from, kernel_to, metadata, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                event_type,
                event.timestamp.to_rfc3339(),
                event.version_from,
                event.version_to,
                event.kernel_from,
                event.kernel_to,
                metadata,
                event.notes,
            ],
        )?;

        Ok(id as i64)
    }

    /// Record a repair action
    pub fn record_repair(&self, repair: &RepairRecord) -> Result<i64> {
        let trigger_type = serde_json::to_string(&repair.trigger_type)?;
        let actions_taken = serde_json::to_string(&repair.actions_taken)?;
        let metrics_before = repair
            .metrics_before
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let metrics_after = repair
            .metrics_after
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let user_feedback = repair
            .user_feedback
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let id = self.conn.execute(
            "INSERT INTO repair_history (timestamp, trigger_type, action_type, actions_taken, metrics_before, metrics_after, success, regressed, user_feedback, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                repair.timestamp.to_rfc3339(),
                trigger_type,
                repair.action_type,
                actions_taken,
                metrics_before,
                metrics_after,
                repair.success,
                repair.regressed,
                user_feedback,
                repair.notes,
            ],
        )?;

        Ok(id as i64)
    }

    /// Get timeline events since a given date
    pub fn get_timeline_since(&self, since: DateTime<Utc>) -> Result<Vec<TimelineEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT event_type, timestamp, version_from, version_to, kernel_from, kernel_to, metadata, notes
             FROM system_timeline
             WHERE timestamp >= ?1
             ORDER BY timestamp DESC"
        )?;

        let events = stmt.query_map([since.to_rfc3339()], |row| {
            let event_type_str: String = row.get(0)?;
            let timestamp_str: String = row.get(1)?;
            let metadata_str: String = row.get(6)?;

            Ok(TimelineEvent {
                event_type: serde_json::from_str(&event_type_str).unwrap(),
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                    .unwrap()
                    .with_timezone(&Utc),
                version_from: row.get(2)?,
                version_to: row.get(3)?,
                kernel_from: row.get(4)?,
                kernel_to: row.get(5)?,
                metadata: serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null),
                notes: row.get(7)?,
            })
        })?;

        events.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get repair effectiveness statistics
    pub fn get_repair_effectiveness(&self) -> Result<RepairEffectiveness> {
        let total: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM repair_history", [], |row| row.get(0))?;

        let successful: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM repair_history WHERE success = 1",
            [],
            |row| row.get(0),
        )?;

        let regressed: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM repair_history WHERE regressed = 1",
            [],
            |row| row.get(0),
        )?;

        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let regression_rate = if total > 0 {
            (regressed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(RepairEffectiveness {
            total_repairs: total as usize,
            successful_repairs: successful as usize,
            regressed_repairs: regressed as usize,
            success_rate,
            regression_rate,
        })
    }

    // ========================================================================
    // Boot Analysis
    // ========================================================================

    /// Record a boot event
    pub fn record_boot_event(&self, boot: &BootEvent) -> Result<i64> {
        let slowest_units = serde_json::to_string(&boot.slowest_units)?;
        let failed_units = serde_json::to_string(&boot.failed_units)?;
        let degraded_units = serde_json::to_string(&boot.degraded_units)?;
        let kernel_errors = serde_json::to_string(&boot.kernel_errors)?;

        let id = self.conn.execute(
            "INSERT OR REPLACE INTO boot_events (
                boot_id, boot_timestamp, shutdown_timestamp, boot_duration_ms, shutdown_duration_ms,
                target_reached, time_to_target_ms, slowest_units, failed_units, degraded_units,
                fsck_triggered, fsck_duration_ms, kernel_errors, boot_health_score
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                boot.boot_id,
                boot.boot_timestamp.to_rfc3339(),
                boot.shutdown_timestamp.map(|t| t.to_rfc3339()),
                boot.boot_duration_ms,
                boot.shutdown_duration_ms,
                boot.target_reached,
                boot.time_to_target_ms,
                slowest_units,
                failed_units,
                degraded_units,
                boot.fsck_triggered,
                boot.fsck_duration_ms,
                kernel_errors,
                boot.boot_health_score,
            ],
        )?;

        Ok(id as i64)
    }

    /// Compute boot aggregates for a given date
    pub fn compute_boot_aggregates(&self, date: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO boot_aggregates (date, boot_count, avg_boot_duration_ms, min_boot_duration_ms, max_boot_duration_ms, failed_boot_count, avg_health_score)
             SELECT
                DATE(?1) as date,
                COUNT(*) as boot_count,
                AVG(boot_duration_ms) as avg_boot_duration_ms,
                MIN(boot_duration_ms) as min_boot_duration_ms,
                MAX(boot_duration_ms) as max_boot_duration_ms,
                SUM(CASE WHEN LENGTH(failed_units) > 2 THEN 1 ELSE 0 END) as failed_boot_count,
                AVG(boot_health_score) as avg_health_score
             FROM boot_events
             WHERE DATE(boot_timestamp) = DATE(?1)",
            [date],
        )?;

        Ok(())
    }

    /// Get boot time trends over the last N days
    pub fn get_boot_trends(&self, days: u32) -> Result<BootTrends> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            "SELECT date, avg_boot_duration_ms, avg_health_score
             FROM boot_aggregates
             WHERE date >= DATE(?1)
             ORDER BY date ASC",
        )?;

        let mut boot_times = Vec::new();
        let mut health_scores = Vec::new();

        let rows = stmt.query_map([cutoff.format("%Y-%m-%d").to_string()], |row| {
            let avg_boot: Option<f64> = row.get(1)?;
            let avg_health: Option<f64> = row.get(2)?;
            Ok((avg_boot, avg_health))
        })?;

        for row in rows {
            let (avg_boot, avg_health) = row?;
            if let Some(bt) = avg_boot {
                boot_times.push(bt);
            }
            if let Some(hs) = avg_health {
                health_scores.push(hs);
            }
        }

        let trend = calculate_trend(&boot_times);
        let avg_boot_time = if !boot_times.is_empty() {
            boot_times.iter().sum::<f64>() / boot_times.len() as f64
        } else {
            0.0
        };

        Ok(BootTrends {
            avg_boot_time_ms: avg_boot_time as i64,
            trend,
            days_analyzed: days,
        })
    }

    /// Get slowest boot units (recurring offenders)
    pub fn get_slowest_units(&self) -> Result<Vec<SlowUnitStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT slowest_units FROM boot_events ORDER BY boot_timestamp DESC LIMIT 30",
        )?;

        let mut unit_stats: HashMap<String, (i64, i64)> = HashMap::new(); // (total_time, count)

        let rows = stmt.query_map([], |row| {
            let slowest_units_str: String = row.get(0)?;
            Ok(slowest_units_str)
        })?;

        for row in rows {
            let slowest_units_str = row?;
            if let Ok(units) = serde_json::from_str::<Vec<SlowUnit>>(&slowest_units_str) {
                for unit in units {
                    let entry = unit_stats.entry(unit.unit.clone()).or_insert((0, 0));
                    entry.0 += unit.duration_ms;
                    entry.1 += 1;
                }
            }
        }

        let mut results: Vec<SlowUnitStats> = unit_stats
            .into_iter()
            .map(|(unit, (total_time, count))| SlowUnitStats {
                unit,
                avg_duration_ms: (total_time / count),
                occurrences: count as usize,
            })
            .collect();

        results.sort_by(|a, b| b.avg_duration_ms.cmp(&a.avg_duration_ms));
        results.truncate(10);

        Ok(results)
    }

    // ========================================================================
    // Resource Tracking (CPU/Memory)
    // ========================================================================

    /// Record a CPU sample
    pub fn record_cpu_sample(&self, sample: &CpuSample) -> Result<i64> {
        let top_processes = serde_json::to_string(&sample.top_processes)?;

        let id = self.conn.execute(
            "INSERT INTO cpu_samples (
                timestamp, window_start, window_end, avg_utilization_percent, peak_utilization_percent,
                idle_background_percent, throttle_event_count, spike_count, top_processes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                sample.timestamp.to_rfc3339(),
                sample.window_start.to_rfc3339(),
                sample.window_end.to_rfc3339(),
                sample.avg_utilization_percent,
                sample.peak_utilization_percent,
                sample.idle_background_percent,
                sample.throttle_event_count,
                sample.spike_count,
                top_processes,
            ],
        )?;

        Ok(id as i64)
    }

    /// Record a memory sample
    pub fn record_memory_sample(&self, sample: &MemorySample) -> Result<i64> {
        let oom_victims = serde_json::to_string(&sample.oom_victims)?;
        let top_memory_hogs = serde_json::to_string(&sample.top_memory_hogs)?;

        let id = self.conn.execute(
            "INSERT INTO memory_samples (
                timestamp, window_start, window_end, avg_ram_used_mb, peak_ram_used_mb,
                avg_swap_used_mb, peak_swap_used_mb, oom_kill_count, oom_victims, top_memory_hogs
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                sample.timestamp.to_rfc3339(),
                sample.window_start.to_rfc3339(),
                sample.window_end.to_rfc3339(),
                sample.avg_ram_used_mb,
                sample.peak_ram_used_mb,
                sample.avg_swap_used_mb,
                sample.peak_swap_used_mb,
                sample.oom_kill_count,
                oom_victims,
                top_memory_hogs,
            ],
        )?;

        Ok(id as i64)
    }

    /// Compute resource aggregates for a given date
    pub fn compute_resource_aggregates(&self, date: &str) -> Result<()> {
        // CPU aggregates
        self.conn.execute(
            "INSERT OR REPLACE INTO cpu_aggregates (date, avg_utilization_percent, peak_utilization_percent, total_throttle_events, avg_idle_percent)
             SELECT
                DATE(?1) as date,
                AVG(avg_utilization_percent) as avg_utilization_percent,
                MAX(peak_utilization_percent) as peak_utilization_percent,
                SUM(throttle_event_count) as total_throttle_events,
                AVG(idle_background_percent) as avg_idle_percent
             FROM cpu_samples
             WHERE DATE(timestamp) = DATE(?1)",
            [date],
        )?;

        // Memory aggregates
        self.conn.execute(
            "INSERT OR REPLACE INTO memory_aggregates (date, avg_ram_used_mb, peak_ram_used_mb, total_oom_kills, swap_dependency_score)
             SELECT
                DATE(?1) as date,
                AVG(avg_ram_used_mb) as avg_ram_used_mb,
                MAX(peak_ram_used_mb) as peak_ram_used_mb,
                SUM(oom_kill_count) as total_oom_kills,
                CAST(AVG(CASE WHEN avg_swap_used_mb > 100 THEN 80 ELSE 20 END) AS INTEGER) as swap_dependency_score
             FROM memory_samples
             WHERE DATE(timestamp) = DATE(?1)",
            [date],
        )?;

        Ok(())
    }

    /// Get CPU usage trends over the last N days
    pub fn get_cpu_trends(&self, days: u32) -> Result<CpuTrends> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            "SELECT avg_utilization_percent FROM cpu_aggregates
             WHERE date >= DATE(?1)
             ORDER BY date ASC",
        )?;

        let mut utilizations = Vec::new();
        let rows = stmt.query_map([cutoff.format("%Y-%m-%d").to_string()], |row| {
            let util: Option<f64> = row.get(0)?;
            Ok(util)
        })?;

        for row in rows {
            if let Some(util) = row? {
                utilizations.push(util);
            }
        }

        let trend = calculate_trend(&utilizations);
        let avg_utilization = if !utilizations.is_empty() {
            utilizations.iter().sum::<f64>() / utilizations.len() as f64
        } else {
            0.0
        };

        Ok(CpuTrends {
            avg_utilization_percent: avg_utilization,
            trend,
            days_analyzed: days,
        })
    }

    /// Get memory usage trends over the last N days
    pub fn get_memory_trends(&self, days: u32) -> Result<MemoryTrends> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            "SELECT avg_ram_used_mb, peak_ram_used_mb FROM memory_aggregates
             WHERE date >= DATE(?1)
             ORDER BY date ASC",
        )?;

        let mut ram_usage = Vec::new();
        let rows = stmt.query_map([cutoff.format("%Y-%m-%d").to_string()], |row| {
            let avg_ram: Option<i64> = row.get(0)?;
            let peak_ram: Option<i64> = row.get(1)?;
            Ok((avg_ram, peak_ram))
        })?;

        for row in rows {
            if let (Some(avg), _) = row? {
                ram_usage.push(avg as f64);
            }
        }

        let trend = calculate_trend(&ram_usage);
        let avg_ram = if !ram_usage.is_empty() {
            ram_usage.iter().sum::<f64>() / ram_usage.len() as f64
        } else {
            0.0
        };

        Ok(MemoryTrends {
            avg_ram_used_mb: avg_ram as i64,
            trend,
            days_analyzed: days,
        })
    }

    /// Identify resource hogs (processes that consistently use too much)
    pub fn identify_resource_hogs(&self) -> Result<Vec<ResourceHog>> {
        let mut stmt = self
            .conn
            .prepare("SELECT top_processes FROM cpu_samples ORDER BY timestamp DESC LIMIT 50")?;

        let mut process_stats: HashMap<String, (f64, usize)> = HashMap::new(); // (total_cpu, count)

        let rows = stmt.query_map([], |row| {
            let top_processes_str: String = row.get(0)?;
            Ok(top_processes_str)
        })?;

        for row in rows {
            let top_processes_str = row?;
            if let Ok(processes) = serde_json::from_str::<Vec<ProcessCpuInfo>>(&top_processes_str) {
                for process in processes {
                    let entry = process_stats
                        .entry(process.name.clone())
                        .or_insert((0.0, 0));
                    entry.0 += process.cpu_percent;
                    entry.1 += 1;
                }
            }
        }

        let mut results: Vec<ResourceHog> = process_stats
            .into_iter()
            .filter(|(_, (_, count))| *count >= 5) // At least 5 occurrences
            .map(|(name, (total_cpu, count))| ResourceHog {
                process_name: name,
                avg_cpu_percent: total_cpu / count as f64,
                occurrences: count,
            })
            .collect();

        results.sort_by(|a, b| b.avg_cpu_percent.partial_cmp(&a.avg_cpu_percent).unwrap());
        results.truncate(10);

        Ok(results)
    }

    // ========================================================================
    // Disk & Network Tracking
    // ========================================================================

    /// Record a disk space snapshot
    pub fn record_disk_snapshot(
        &self,
        filesystem: &str,
        total_gb: f64,
        used_gb: f64,
        inode_used_percent: Option<f64>,
    ) -> Result<i64> {
        let free_gb = total_gb - used_gb;
        let used_percent = (used_gb / total_gb) * 100.0;

        let id = self.conn.execute(
            "INSERT INTO disk_space_samples (timestamp, filesystem, total_gb, used_gb, free_gb, used_percent, inode_used_percent)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                Utc::now().to_rfc3339(),
                filesystem,
                total_gb,
                used_gb,
                free_gb,
                used_percent,
                inode_used_percent,
            ],
        )?;

        Ok(id as i64)
    }

    /// Record an I/O sample
    pub fn record_io_sample(&self, device: &str, io_stats: &IoStats) -> Result<i64> {
        let id = self.conn.execute(
            "INSERT INTO io_samples (timestamp, window_start, window_end, device, avg_read_mb_per_sec, avg_write_mb_per_sec, avg_queue_depth, avg_latency_ms, high_latency_events)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                io_stats.timestamp.to_rfc3339(),
                io_stats.window_start.to_rfc3339(),
                io_stats.window_end.to_rfc3339(),
                device,
                io_stats.avg_read_mb_per_sec,
                io_stats.avg_write_mb_per_sec,
                io_stats.avg_queue_depth,
                io_stats.avg_latency_ms,
                io_stats.high_latency_events,
            ],
        )?;

        Ok(id as i64)
    }

    /// Record a network quality sample
    pub fn record_network_sample(
        &self,
        interface: &str,
        quality_data: &NetworkQuality,
    ) -> Result<i64> {
        let id = self.conn.execute(
            "INSERT INTO network_samples (timestamp, window_start, window_end, interface, target, avg_latency_ms, packet_loss_percent, disconnect_count, dhcp_renew_failures, dns_failures)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                quality_data.timestamp.to_rfc3339(),
                quality_data.window_start.to_rfc3339(),
                quality_data.window_end.to_rfc3339(),
                interface,
                quality_data.target,
                quality_data.avg_latency_ms,
                quality_data.packet_loss_percent,
                quality_data.disconnect_count,
                quality_data.dhcp_renew_failures,
                quality_data.dns_failures,
            ],
        )?;

        Ok(id as i64)
    }

    /// Analyze disk growth rate over the last N days
    pub fn analyze_disk_growth(&self, filesystem: &str, days: u32) -> Result<DiskGrowthAnalysis> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            "SELECT timestamp, used_gb FROM disk_space_samples
             WHERE filesystem = ?1 AND timestamp >= ?2
             ORDER BY timestamp ASC",
        )?;

        let mut samples: Vec<(DateTime<Utc>, f64)> = Vec::new();
        let rows = stmt.query_map(rusqlite::params![filesystem, cutoff.to_rfc3339()], |row| {
            let timestamp_str: String = row.get(0)?;
            let used_gb: f64 = row.get(1)?;
            Ok((timestamp_str, used_gb))
        })?;

        for row in rows {
            let (timestamp_str, used_gb) = row?;
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(&timestamp_str) {
                samples.push((timestamp.with_timezone(&Utc), used_gb));
            }
        }

        if samples.len() < 2 {
            return Ok(DiskGrowthAnalysis {
                filesystem: filesystem.to_string(),
                growth_rate_gb_per_day: 0.0,
                days_until_full: None,
                current_used_gb: samples.first().map(|(_, gb)| *gb).unwrap_or(0.0),
            });
        }

        let first = samples.first().unwrap();
        let last = samples.last().unwrap();
        let days_elapsed = (last.0 - first.0).num_days().max(1) as f64;
        let growth = last.1 - first.1;
        let growth_rate = growth / days_elapsed;

        // Get total size to estimate days until full
        let total_gb: Option<f64> = self.conn.query_row(
            "SELECT total_gb FROM disk_space_samples WHERE filesystem = ?1 ORDER BY timestamp DESC LIMIT 1",
            [filesystem],
            |row| row.get(0),
        ).ok();

        let days_until_full = if let Some(total) = total_gb {
            if growth_rate > 0.01 {
                let remaining = total - last.1;
                Some(((remaining / growth_rate) as i64).max(0))
            } else {
                None
            }
        } else {
            None
        };

        Ok(DiskGrowthAnalysis {
            filesystem: filesystem.to_string(),
            growth_rate_gb_per_day: growth_rate,
            days_until_full,
            current_used_gb: last.1,
        })
    }

    /// Get network quality trends for an interface over the last N days
    pub fn get_network_quality_trends(
        &self,
        interface: &str,
        days: u32,
    ) -> Result<NetworkQualityTrends> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            "SELECT avg_latency_ms, packet_loss_percent FROM network_samples
             WHERE interface = ?1 AND timestamp >= ?2
             ORDER BY timestamp ASC",
        )?;

        let mut latencies = Vec::new();
        let mut packet_losses = Vec::new();

        let rows = stmt.query_map(rusqlite::params![interface, cutoff.to_rfc3339()], |row| {
            let latency: Option<f64> = row.get(0)?;
            let loss: Option<f64> = row.get(1)?;
            Ok((latency, loss))
        })?;

        for row in rows {
            let (latency, loss) = row?;
            if let Some(l) = latency {
                latencies.push(l);
            }
            if let Some(pl) = loss {
                packet_losses.push(pl);
            }
        }

        let avg_latency = if !latencies.is_empty() {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        } else {
            0.0
        };

        let avg_packet_loss = if !packet_losses.is_empty() {
            packet_losses.iter().sum::<f64>() / packet_losses.len() as f64
        } else {
            0.0
        };

        Ok(NetworkQualityTrends {
            interface: interface.to_string(),
            avg_latency_ms: avg_latency,
            avg_packet_loss_percent: avg_packet_loss,
            days_analyzed: days,
        })
    }

    // ========================================================================
    // Service Reliability
    // ========================================================================

    /// Record a service event
    pub fn record_service_event(
        &self,
        service_name: &str,
        event_type: &str,
        metadata: Option<&serde_json::Value>,
    ) -> Result<i64> {
        let metadata_str = metadata.map(serde_json::to_string).transpose()?;

        let id = self.conn.execute(
            "INSERT INTO service_reliability (timestamp, service_name, event_type, was_intentional, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                Utc::now().to_rfc3339(),
                service_name,
                event_type,
                event_type != "crash", // crashes are unintentional
                metadata_str,
            ],
        )?;

        Ok(id as i64)
    }

    /// Compute service aggregates for a given date
    pub fn compute_service_aggregates(&self, date: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO service_aggregates (date, service_name, restart_count, crash_count, stability_score)
             SELECT
                DATE(?1) as date,
                service_name,
                SUM(CASE WHEN event_type = 'restart' THEN 1 ELSE 0 END) as restart_count,
                SUM(CASE WHEN event_type = 'crash' THEN 1 ELSE 0 END) as crash_count,
                CASE
                    WHEN SUM(CASE WHEN event_type = 'crash' THEN 1 ELSE 0 END) = 0 THEN 100
                    WHEN SUM(CASE WHEN event_type = 'crash' THEN 1 ELSE 0 END) <= 2 THEN 80
                    WHEN SUM(CASE WHEN event_type = 'crash' THEN 1 ELSE 0 END) <= 5 THEN 60
                    ELSE 40
                END as stability_score
             FROM service_reliability
             WHERE DATE(timestamp) = DATE(?1)
             GROUP BY service_name",
            [date],
        )?;

        Ok(())
    }

    /// Get service stability scores (identify flaky services)
    pub fn get_service_stability_scores(&self) -> Result<Vec<ServiceStability>> {
        let mut stmt = self.conn.prepare(
            "SELECT service_name, AVG(stability_score) as avg_score, SUM(crash_count) as total_crashes
             FROM service_aggregates
             WHERE date >= DATE('now', '-30 days')
             GROUP BY service_name
             HAVING total_crashes > 0
             ORDER BY avg_score ASC, total_crashes DESC
             LIMIT 20"
        )?;

        let services = stmt.query_map([], |row| {
            Ok(ServiceStability {
                service_name: row.get(0)?,
                stability_score: row.get::<_, f64>(1)? as u8,
                total_crashes: row.get::<_, i64>(2)? as usize,
            })
        })?;

        services.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get crash patterns for a service over the last N days
    pub fn get_crash_patterns(&self, service: &str, days: u32) -> Result<Vec<ServiceCrashPattern>> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            "SELECT timestamp, exit_code, signal FROM service_reliability
             WHERE service_name = ?1 AND event_type = 'crash' AND timestamp >= ?2
             ORDER BY timestamp DESC",
        )?;

        let crashes = stmt.query_map(rusqlite::params![service, cutoff.to_rfc3339()], |row| {
            let timestamp_str: String = row.get(0)?;
            Ok(ServiceCrashPattern {
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                    .unwrap()
                    .with_timezone(&Utc),
                exit_code: row.get(1)?,
                signal: row.get(2)?,
            })
        })?;

        crashes.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ========================================================================
    // Error Statistics
    // ========================================================================

    /// Record an error with signature deduplication
    pub fn record_error(&self, source: &str, severity: &str, message: &str) -> Result<i64> {
        // Create a signature hash from the normalized message
        let signature_hash = create_signature_hash(message);
        let now = Utc::now().to_rfc3339();

        // Try to update existing signature
        let updated = self.conn.execute(
            "UPDATE error_signatures SET last_occurrence = ?1, total_count = total_count + 1
             WHERE signature_hash = ?2",
            rusqlite::params![now, signature_hash],
        )?;

        if updated == 0 {
            // Insert new signature
            self.conn.execute(
                "INSERT INTO error_signatures (signature_hash, source, severity, message_template, first_occurrence, last_occurrence, total_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
                rusqlite::params![signature_hash, source, severity, message, now, now],
            )?;
        }

        Ok(1)
    }

    /// Record error rate sample
    pub fn record_error_rate_sample(
        &self,
        error_count: i32,
        warning_count: i32,
        critical_count: i32,
        new_signature_count: i32,
    ) -> Result<i64> {
        let now = Utc::now();
        let window_start = now - chrono::Duration::hours(1);

        let id = self.conn.execute(
            "INSERT INTO error_rate_samples (timestamp, window_start, window_end, error_count, warning_count, critical_count, new_signature_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                now.to_rfc3339(),
                window_start.to_rfc3339(),
                now.to_rfc3339(),
                error_count,
                warning_count,
                critical_count,
                new_signature_count,
            ],
        )?;

        Ok(id as i64)
    }

    /// Get error trends over the last N days
    pub fn get_error_trends(&self, days: u32) -> Result<ErrorTrends> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            "SELECT SUM(error_count), SUM(warning_count), SUM(critical_count) FROM error_rate_samples
             WHERE timestamp >= ?1"
        )?;

        let (total_errors, total_warnings, total_criticals) =
            stmt.query_row([cutoff.to_rfc3339()], |row| {
                Ok((
                    row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                    row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                ))
            })?;

        Ok(ErrorTrends {
            total_errors: total_errors as usize,
            total_warnings: total_warnings as usize,
            total_criticals: total_criticals as usize,
            days_analyzed: days,
        })
    }

    /// Identify new errors that never appeared before the given date
    pub fn identify_new_errors(&self, since: DateTime<Utc>) -> Result<Vec<ErrorSignature>> {
        let mut stmt = self.conn.prepare(
            "SELECT signature_hash, source, severity, message_template, first_occurrence, last_occurrence, total_count
             FROM error_signatures
             WHERE first_occurrence >= ?1
             ORDER BY first_occurrence DESC"
        )?;

        let errors = stmt.query_map([since.to_rfc3339()], |row| {
            let first_str: String = row.get(4)?;
            let last_str: String = row.get(5)?;

            Ok(ErrorSignature {
                signature_hash: row.get(0)?,
                source: row.get(1)?,
                severity: row.get(2)?,
                message_template: row.get(3)?,
                first_occurrence: DateTime::parse_from_rfc3339(&first_str)
                    .unwrap()
                    .with_timezone(&Utc),
                last_occurrence: DateTime::parse_from_rfc3339(&last_str)
                    .unwrap()
                    .with_timezone(&Utc),
                total_count: row.get::<_, i64>(6)? as usize,
            })
        })?;

        errors.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Check if an error disappeared after a change
    pub fn check_error_disappeared(
        &self,
        signature_hash: &str,
        after_change_id: i64,
    ) -> Result<bool> {
        // Get the change timestamp
        let change_timestamp: String = self.conn.query_row(
            "SELECT timestamp FROM repair_history WHERE id = ?1",
            [after_change_id],
            |row| row.get(0),
        )?;

        // Check if error's last occurrence is before the change
        let last_occurrence: String = self.conn.query_row(
            "SELECT last_occurrence FROM error_signatures WHERE signature_hash = ?1",
            [signature_hash],
            |row| row.get(0),
        )?;

        let change_dt = DateTime::parse_from_rfc3339(&change_timestamp)?.with_timezone(&Utc);
        let last_dt = DateTime::parse_from_rfc3339(&last_occurrence)?.with_timezone(&Utc);

        // Mark as disappeared if it hasn't been seen after the change
        if last_dt < change_dt {
            self.conn.execute(
                "UPDATE error_signatures SET disappeared_after_change = ?1 WHERE signature_hash = ?2",
                rusqlite::params![after_change_id.to_string(), signature_hash],
            )?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // ========================================================================
    // Baseline & Performance Tracking
    // ========================================================================

    /// Save a performance baseline
    pub fn save_baseline(&self, baseline: &Baseline) -> Result<i64> {
        // Deactivate previous baselines of the same type
        self.conn.execute(
            "UPDATE baselines SET is_active = 0 WHERE baseline_type = ?1 AND is_active = 1",
            [&baseline.baseline_type],
        )?;

        let metrics = serde_json::to_string(&baseline.metrics)?;
        let hardware_config = baseline
            .hardware_config
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let id = self.conn.execute(
            "INSERT INTO baselines (baseline_type, timestamp, metrics, hardware_config, notes, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                baseline.baseline_type,
                baseline.timestamp.to_rfc3339(),
                metrics,
                hardware_config,
                baseline.notes,
                baseline.is_active,
            ],
        )?;

        Ok(id as i64)
    }

    /// Get the active baseline for a given type
    pub fn get_active_baseline(&self, baseline_type: &str) -> Result<Option<Baseline>> {
        let mut stmt = self.conn.prepare(
            "SELECT baseline_type, timestamp, metrics, hardware_config, notes, is_active
             FROM baselines
             WHERE baseline_type = ?1 AND is_active = 1
             ORDER BY timestamp DESC
             LIMIT 1",
        )?;

        let result = stmt.query_row([baseline_type], |row| {
            let timestamp_str: String = row.get(1)?;
            let metrics_str: String = row.get(2)?;
            let hardware_str: Option<String> = row.get(3)?;

            Ok(Baseline {
                baseline_type: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                    .unwrap()
                    .with_timezone(&Utc),
                metrics: serde_json::from_str(&metrics_str).unwrap(),
                hardware_config: hardware_str.and_then(|s| serde_json::from_str(&s).ok()),
                notes: row.get(4)?,
                is_active: row.get(5)?,
            })
        });

        match result {
            Ok(baseline) => Ok(Some(baseline)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Record a performance delta (before/after a change)
    pub fn record_performance_delta(
        &self,
        change_type: &str,
        change_id: Option<&str>,
        metric_name: &str,
        value_before: f64,
        value_after: f64,
    ) -> Result<i64> {
        let delta_percent = ((value_after - value_before) / value_before) * 100.0;
        let impact = if delta_percent.abs() < 5.0 {
            "minimal"
        } else if delta_percent < -10.0 {
            "significant_improvement"
        } else if delta_percent > 10.0 {
            "significant_regression"
        } else if delta_percent < 0.0 {
            "minor_improvement"
        } else {
            "minor_regression"
        };

        let id = self.conn.execute(
            "INSERT INTO performance_deltas (timestamp, change_type, change_id, metric_name, value_before, value_after, delta_percent, impact_description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                Utc::now().to_rfc3339(),
                change_type,
                change_id,
                metric_name,
                value_before,
                value_after,
                delta_percent,
                impact,
            ],
        )?;

        Ok(id as i64)
    }

    /// Compare current metrics to baseline
    pub fn compare_to_baseline(
        &self,
        baseline_type: &str,
        current_metrics: &serde_json::Value,
    ) -> Result<Option<BaselineComparison>> {
        let baseline = match self.get_active_baseline(baseline_type)? {
            Some(b) => b,
            None => return Ok(None),
        };

        // Extract comparable metrics (this is simplified - real implementation would be more sophisticated)
        let deviations = calculate_metric_deviations(&baseline.metrics, current_metrics);

        Ok(Some(BaselineComparison {
            baseline_type: baseline_type.to_string(),
            baseline_timestamp: baseline.timestamp,
            deviations,
        }))
    }

    // ========================================================================
    // User Behavior & LLM Stats
    // ========================================================================

    /// Record daily usage pattern
    pub fn record_usage_pattern(&self, date: &str, pattern_data: &UsagePatternData) -> Result<i64> {
        let active_hours = serde_json::to_string(&pattern_data.active_hours)?;
        let heavy_load_hours = serde_json::to_string(&pattern_data.heavy_load_hours)?;
        let common_applications = serde_json::to_string(&pattern_data.common_applications)?;

        let id = self.conn.execute(
            "INSERT OR REPLACE INTO usage_patterns (date, active_hours, heavy_load_hours, common_applications, package_update_count, anna_invocation_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                date,
                active_hours,
                heavy_load_hours,
                common_applications,
                pattern_data.package_update_count,
                pattern_data.anna_invocation_count,
            ],
        )?;

        Ok(id as i64)
    }

    /// Record LLM performance sample
    pub fn record_llm_sample(
        &self,
        model: &str,
        performance_data: &LlmPerformanceData,
    ) -> Result<i64> {
        let id = self.conn.execute(
            "INSERT INTO llm_samples (timestamp, window_start, window_end, model_name, avg_latency_ms, total_requests, failed_requests, avg_memory_mb, avg_gpu_utilization_percent, avg_cpu_utilization_percent, avg_temperature_c, max_temperature_c)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                performance_data.timestamp.to_rfc3339(),
                performance_data.window_start.to_rfc3339(),
                performance_data.window_end.to_rfc3339(),
                model,
                performance_data.avg_latency_ms,
                performance_data.total_requests,
                performance_data.failed_requests,
                performance_data.avg_memory_mb,
                performance_data.avg_gpu_utilization_percent,
                performance_data.avg_cpu_utilization_percent,
                performance_data.avg_temperature_c,
                performance_data.max_temperature_c,
            ],
        )?;

        Ok(id as i64)
    }

    /// Record model change
    pub fn record_model_change(
        &self,
        from: Option<&str>,
        to: &str,
        reason: Option<&str>,
    ) -> Result<i64> {
        let id = self.conn.execute(
            "INSERT INTO llm_model_history (timestamp, event_type, model_from, model_to, reason)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                Utc::now().to_rfc3339(),
                if from.is_some() { "switch" } else { "install" },
                from,
                to,
                reason,
            ],
        )?;

        Ok(id as i64)
    }

    /// Detect usage anomalies
    pub fn detect_usage_anomalies(&self) -> Result<Vec<UsageAnomaly>> {
        // Simple anomaly detection: compare last 7 days to previous 30 days
        let mut anomalies = Vec::new();

        // Check for unusual heavy load hours
        let recent_heavy_hours: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM usage_patterns
             WHERE date >= DATE('now', '-7 days')
             AND LENGTH(heavy_load_hours) > 10",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if recent_heavy_hours > 5 {
            anomalies.push(UsageAnomaly {
                timestamp: Utc::now(),
                anomaly_type: "unexpected_heavy_load".to_string(),
                description: format!(
                    "Heavy system load detected on {} of the last 7 days",
                    recent_heavy_hours
                ),
                severity: "warning".to_string(),
            });
        }

        Ok(anomalies)
    }

    // ========================================================================
    // Synthesized Health Scores
    // ========================================================================

    /// Compute daily health scores
    pub fn compute_daily_health_scores(&self, date: &str) -> Result<()> {
        // Get various metrics for the day
        let boot_health: Option<f64> = self
            .conn
            .query_row(
                "SELECT avg_health_score FROM boot_aggregates WHERE date = ?1",
                [date],
                |row| row.get(0),
            )
            .ok()
            .flatten();

        let error_count: i64 = self
            .conn
            .query_row(
                "SELECT SUM(error_count) FROM error_rate_samples WHERE DATE(timestamp) = ?1",
                [date],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let service_crashes: i64 = self
            .conn
            .query_row(
                "SELECT SUM(crash_count) FROM service_aggregates WHERE date = ?1",
                [date],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Calculate scores (0-100)
        let stability_score = calculate_stability_score(boot_health, service_crashes);
        let performance_score = calculate_performance_score(boot_health);
        let noise_score = calculate_noise_score(error_count);

        // Determine trends (simplified - needs historical comparison)
        let stability_trend = "flat";
        let performance_trend = "flat";
        let noise_trend = "flat";

        self.conn.execute(
            "INSERT OR REPLACE INTO health_scores (date, stability_score, performance_score, noise_score, stability_trend, performance_trend, noise_trend)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                date,
                stability_score,
                performance_score,
                noise_score,
                stability_trend,
                performance_trend,
                noise_trend,
            ],
        )?;

        Ok(())
    }

    /// Compute trends across all health scores
    pub fn compute_trends(&self) -> Result<HealthTrendSummary> {
        let mut stmt = self.conn.prepare(
            "SELECT date, stability_score, performance_score, noise_score FROM health_scores
             ORDER BY date DESC LIMIT 30",
        )?;

        let mut stability_scores = Vec::new();
        let mut performance_scores = Vec::new();
        let mut noise_scores = Vec::new();

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;

        for row in rows {
            let (stability, performance, noise) = row?;
            stability_scores.push(stability as f64);
            performance_scores.push(performance as f64);
            noise_scores.push(noise as f64);
        }

        Ok(HealthTrendSummary {
            stability_trend: calculate_trend(&stability_scores),
            performance_trend: calculate_trend(&performance_scores),
            noise_trend: calculate_trend(&noise_scores),
        })
    }

    /// Identify regressions and improvements
    pub fn identify_regressions_improvements(&self) -> Result<Vec<HealthChange>> {
        let mut changes = Vec::new();

        // Get the last 30 days of health scores
        let mut stmt = self.conn.prepare(
            "SELECT date, stability_score, performance_score FROM health_scores
             ORDER BY date DESC LIMIT 30",
        )?;

        let mut scores: Vec<(String, i64, i64)> = Vec::new();
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;

        for row in rows {
            scores.push(row?);
        }

        // Look for significant changes (>20 point drop or increase)
        for window in scores.windows(2) {
            let (date1, stab1, perf1) = &window[0];
            let (date2, stab2, perf2) = &window[1];

            if stab1 - stab2 > 20 {
                changes.push(HealthChange {
                    date: date1.clone(),
                    change_type: "regression".to_string(),
                    metric: "stability".to_string(),
                    delta: (*stab1 - *stab2) as i32,
                });
            } else if stab2 - stab1 > 20 {
                changes.push(HealthChange {
                    date: date2.clone(),
                    change_type: "improvement".to_string(),
                    metric: "stability".to_string(),
                    delta: (*stab2 - *stab1) as i32,
                });
            }

            if perf1 - perf2 > 20 {
                changes.push(HealthChange {
                    date: date1.clone(),
                    change_type: "regression".to_string(),
                    metric: "performance".to_string(),
                    delta: (*perf1 - *perf2) as i32,
                });
            } else if perf2 - perf1 > 20 {
                changes.push(HealthChange {
                    date: date2.clone(),
                    change_type: "improvement".to_string(),
                    metric: "performance".to_string(),
                    delta: (*perf2 - *perf1) as i32,
                });
            }
        }

        Ok(changes)
    }

    /// Get health summary for the last N days
    pub fn get_health_summary(&self, days: u32) -> Result<HealthSummary> {
        let cutoff_date = (Utc::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string();

        let mut stmt = self.conn.prepare(
            "SELECT AVG(stability_score), AVG(performance_score), AVG(noise_score) FROM health_scores
             WHERE date >= ?1"
        )?;

        let (avg_stability, avg_performance, avg_noise) = stmt.query_row([cutoff_date], |row| {
            Ok((
                row.get::<_, Option<f64>>(0)?.unwrap_or(0.0),
                row.get::<_, Option<f64>>(1)?.unwrap_or(0.0),
                row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
            ))
        })?;

        Ok(HealthSummary {
            avg_stability_score: avg_stability as u8,
            avg_performance_score: avg_performance as u8,
            avg_noise_score: avg_noise as u8,
            days_analyzed: days,
        })
    }

    // ========================================================================
    // Query APIs for LLM
    // ========================================================================

    /// Get comprehensive system summary
    pub fn get_system_summary(&self) -> Result<SystemSummary> {
        let health = self.get_health_summary(30)?;
        let boot_trends = self.get_boot_trends(30)?;
        let cpu_trends = self.get_cpu_trends(30)?;
        let error_trends = self.get_error_trends(30)?;

        Ok(SystemSummary {
            health_summary: health,
            boot_trends,
            cpu_trends,
            error_trends,
            recent_repairs: self.get_recent_repairs(10)?,
        })
    }

    /// Answer "when did this problem start?"
    pub fn answer_when_did_this_start(
        &self,
        problem_description: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        // This is a simplified implementation - real version would use NLP to match problem description
        // For now, check for new errors in the last 90 days
        let cutoff = Utc::now() - chrono::Duration::days(90);
        let new_errors = self.identify_new_errors(cutoff)?;

        if let Some(earliest) = new_errors.first() {
            Ok(Some(earliest.first_occurrence))
        } else {
            Ok(None)
        }
    }

    /// Get timeline of what changed before a timestamp
    pub fn what_changed_before(&self, timestamp: DateTime<Utc>) -> Result<Vec<ChangeEvent>> {
        let before = timestamp - chrono::Duration::hours(24);

        let mut changes = Vec::new();

        // Get repairs in that window
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, action_type, actions_taken FROM repair_history
             WHERE timestamp BETWEEN ?1 AND ?2
             ORDER BY timestamp DESC",
        )?;

        let rows = stmt.query_map(
            rusqlite::params![before.to_rfc3339(), timestamp.to_rfc3339()],
            |row| {
                let ts_str: String = row.get(0)?;
                let actions_str: String = row.get(2)?;
                Ok((ts_str, row.get::<_, String>(1)?, actions_str))
            },
        )?;

        for row in rows {
            let (ts_str, action_type, actions_str) = row?;
            let ts = DateTime::parse_from_rfc3339(&ts_str)?.with_timezone(&Utc);
            let actions: Vec<String> = serde_json::from_str(&actions_str).unwrap_or_default();

            changes.push(ChangeEvent {
                timestamp: ts,
                change_type: action_type,
                description: actions.join(", "),
            });
        }

        // Get timeline events
        let mut stmt2 = self.conn.prepare(
            "SELECT timestamp, event_type, notes FROM system_timeline
             WHERE timestamp BETWEEN ?1 AND ?2
             ORDER BY timestamp DESC",
        )?;

        let rows2 = stmt2.query_map(
            rusqlite::params![before.to_rfc3339(), timestamp.to_rfc3339()],
            |row| {
                let ts_str: String = row.get(0)?;
                let event_str: String = row.get(1)?;
                let notes: Option<String> = row.get(2)?;
                Ok((ts_str, event_str, notes))
            },
        )?;

        for row in rows2 {
            let (ts_str, event_type, notes) = row?;
            let ts = DateTime::parse_from_rfc3339(&ts_str)?.with_timezone(&Utc);

            changes.push(ChangeEvent {
                timestamp: ts,
                change_type: event_type,
                description: notes.unwrap_or_default(),
            });
        }

        changes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(changes)
    }

    /// Get repair impact (before/after metrics)
    pub fn get_repair_impact(&self, repair_id: i64) -> Result<Option<RepairImpact>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, action_type, metrics_before, metrics_after, success FROM repair_history
             WHERE id = ?1"
        )?;

        let result = stmt.query_row([repair_id], |row| {
            let ts_str: String = row.get(0)?;
            let before_str: Option<String> = row.get(2)?;
            let after_str: Option<String> = row.get(3)?;

            Ok(RepairImpact {
                repair_id,
                timestamp: DateTime::parse_from_rfc3339(&ts_str)
                    .unwrap()
                    .with_timezone(&Utc),
                action_type: row.get(1)?,
                metrics_before: before_str.and_then(|s| serde_json::from_str(&s).ok()),
                metrics_after: after_str.and_then(|s| serde_json::from_str(&s).ok()),
                success: row.get(4)?,
            })
        });

        match result {
            Ok(impact) => Ok(Some(impact)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Recommend whether to update baseline
    pub fn recommend_baseline_update(&self) -> Result<bool> {
        // Check if system has been stable and performing well for 7+ days
        let health = self.get_health_summary(7)?;

        Ok(health.avg_stability_score >= 85
            && health.avg_performance_score >= 85
            && health.avg_noise_score >= 75)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    fn get_recent_repairs(&self, limit: usize) -> Result<Vec<RepairSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, action_type, success FROM repair_history
             ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let repairs = stmt.query_map([limit], |row| {
            let ts_str: String = row.get(1)?;
            Ok(RepairSummary {
                id: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&ts_str)
                    .unwrap()
                    .with_timezone(&Utc),
                action_type: row.get(2)?,
                success: row.get(3)?,
            })
        })?;

        repairs.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

// ============================================================================
// Supporting Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairEffectiveness {
    pub total_repairs: usize,
    pub successful_repairs: usize,
    pub regressed_repairs: usize,
    pub success_rate: f64,
    pub regression_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootTrends {
    pub avg_boot_time_ms: i64,
    pub trend: Trend,
    pub days_analyzed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowUnitStats {
    pub unit: String,
    pub avg_duration_ms: i64,
    pub occurrences: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTrends {
    pub avg_utilization_percent: f64,
    pub trend: Trend,
    pub days_analyzed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrends {
    pub avg_ram_used_mb: i64,
    pub trend: Trend,
    pub days_analyzed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHog {
    pub process_name: String,
    pub avg_cpu_percent: f64,
    pub occurrences: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoStats {
    pub timestamp: DateTime<Utc>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub avg_read_mb_per_sec: f64,
    pub avg_write_mb_per_sec: f64,
    pub avg_queue_depth: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub high_latency_events: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkQuality {
    pub timestamp: DateTime<Utc>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub target: String,
    pub avg_latency_ms: Option<f64>,
    pub packet_loss_percent: f64,
    pub disconnect_count: i32,
    pub dhcp_renew_failures: i32,
    pub dns_failures: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskGrowthAnalysis {
    pub filesystem: String,
    pub growth_rate_gb_per_day: f64,
    pub days_until_full: Option<i64>,
    pub current_used_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkQualityTrends {
    pub interface: String,
    pub avg_latency_ms: f64,
    pub avg_packet_loss_percent: f64,
    pub days_analyzed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStability {
    pub service_name: String,
    pub stability_score: u8,
    pub total_crashes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCrashPattern {
    pub timestamp: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrends {
    pub total_errors: usize,
    pub total_warnings: usize,
    pub total_criticals: usize,
    pub days_analyzed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSignature {
    pub signature_hash: String,
    pub source: String,
    pub severity: String,
    pub message_template: String,
    pub first_occurrence: DateTime<Utc>,
    pub last_occurrence: DateTime<Utc>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub baseline_type: String,
    pub baseline_timestamp: DateTime<Utc>,
    pub deviations: Vec<MetricDeviation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDeviation {
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub deviation_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatternData {
    pub active_hours: Vec<u8>,
    pub heavy_load_hours: Vec<u8>,
    pub common_applications: Vec<ApplicationUsage>,
    pub package_update_count: i32,
    pub anna_invocation_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationUsage {
    pub name: String,
    pub usage_count: usize,
    pub total_time_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmPerformanceData {
    pub timestamp: DateTime<Utc>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub avg_latency_ms: Option<i32>,
    pub total_requests: i32,
    pub failed_requests: i32,
    pub avg_memory_mb: Option<i32>,
    pub avg_gpu_utilization_percent: Option<f64>,
    pub avg_cpu_utilization_percent: Option<f64>,
    pub avg_temperature_c: Option<f64>,
    pub max_temperature_c: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnomaly {
    pub timestamp: DateTime<Utc>,
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrendSummary {
    pub stability_trend: Trend,
    pub performance_trend: Trend,
    pub noise_trend: Trend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthChange {
    pub date: String,
    pub change_type: String,
    pub metric: String,
    pub delta: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub avg_stability_score: u8,
    pub avg_performance_score: u8,
    pub avg_noise_score: u8,
    pub days_analyzed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSummary {
    pub health_summary: HealthSummary,
    pub boot_trends: BootTrends,
    pub cpu_trends: CpuTrends,
    pub error_trends: ErrorTrends,
    pub recent_repairs: Vec<RepairSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairSummary {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub timestamp: DateTime<Utc>,
    pub change_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairImpact {
    pub repair_id: i64,
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub metrics_before: Option<serde_json::Value>,
    pub metrics_after: Option<serde_json::Value>,
    pub success: bool,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate trend from a series of values
fn calculate_trend(values: &[f64]) -> Trend {
    if values.len() < 2 {
        return Trend::Flat;
    }

    // Simple linear regression slope
    let n = values.len() as f64;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        numerator += (x - x_mean) * (y - y_mean);
        denominator += (x - x_mean).powi(2);
    }

    if denominator == 0.0 {
        return Trend::Flat;
    }

    let slope = numerator / denominator;

    // Threshold for determining trend
    let threshold = y_mean * 0.02; // 2% of mean

    if slope > threshold {
        Trend::Up
    } else if slope < -threshold {
        Trend::Down
    } else {
        Trend::Flat
    }
}

/// Create a signature hash from an error message
fn create_signature_hash(message: &str) -> String {
    use sha2::{Digest, Sha256};

    // Normalize the message (remove numbers, timestamps, etc.)
    let normalized = message
        .chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase();

    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Calculate stability score from boot health and service crashes
fn calculate_stability_score(boot_health: Option<f64>, service_crashes: i64) -> u8 {
    let boot_component = boot_health.unwrap_or(50.0) * 0.6;
    let crash_penalty = (service_crashes as f64 * 10.0).min(40.0);
    let service_component = (100.0 - crash_penalty) * 0.4;

    (boot_component + service_component).round() as u8
}

/// Calculate performance score from boot health
fn calculate_performance_score(boot_health: Option<f64>) -> u8 {
    boot_health.unwrap_or(50.0).round() as u8
}

/// Calculate noise score from error count
fn calculate_noise_score(error_count: i64) -> u8 {
    if error_count == 0 {
        100
    } else if error_count < 10 {
        90
    } else if error_count < 50 {
        75
    } else if error_count < 100 {
        60
    } else if error_count < 200 {
        40
    } else {
        20
    }
}

/// Calculate metric deviations between baseline and current
fn calculate_metric_deviations(
    baseline: &serde_json::Value,
    current: &serde_json::Value,
) -> Vec<MetricDeviation> {
    let mut deviations = Vec::new();

    // This is a simplified implementation - real version would recursively compare JSON structures
    if let (Some(baseline_obj), Some(current_obj)) = (baseline.as_object(), current.as_object()) {
        for (key, baseline_value) in baseline_obj {
            if let (Some(baseline_num), Some(current_num)) = (
                baseline_value.as_f64(),
                current_obj.get(key).and_then(|v| v.as_f64()),
            ) {
                let deviation_percent = if baseline_num != 0.0 {
                    ((current_num - baseline_num) / baseline_num) * 100.0
                } else {
                    0.0
                };

                if deviation_percent.abs() > 5.0 {
                    deviations.push(MetricDeviation {
                        metric_name: key.clone(),
                        baseline_value: baseline_num,
                        current_value: current_num,
                        deviation_percent,
                    });
                }
            }
        }
    }

    deviations
}
