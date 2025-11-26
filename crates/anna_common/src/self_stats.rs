//! Self Statistics - Anna's Own Performance Metrics
//!
//! v6.56.0: Self Biography, Usage Analytics, Meta Diagnostics
//!
//! Track Anna's own activity:
//! - Query count and latency
//! - LLM token usage
//! - Action success/failure rates
//! - Uptime and restarts
//! - Storage usage trends

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Anna's self-statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfStats {
    /// When these stats were collected
    pub collected_at: DateTime<Utc>,

    /// Total queries handled since first run
    pub total_queries: u64,

    /// Queries handled today
    pub queries_today: u64,

    /// Queries handled this week
    pub queries_this_week: u64,

    /// Average query latency in milliseconds
    pub avg_latency_ms: f64,

    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,

    /// Total LLM tokens consumed
    pub total_llm_tokens: u64,

    /// LLM tokens consumed today
    pub llm_tokens_today: u64,

    /// Total actions executed
    pub total_actions: u64,

    /// Successful actions
    pub successful_actions: u64,

    /// Failed actions
    pub failed_actions: u64,

    /// Action success rate (0.0 - 1.0)
    pub action_success_rate: f64,

    /// Total rollbacks performed
    pub total_rollbacks: u64,

    /// Database size in bytes
    pub database_size_bytes: u64,

    /// Daemon uptime in seconds
    pub uptime_seconds: u64,

    /// Number of daemon restarts
    pub restart_count: u64,

    /// First activity timestamp
    pub first_activity: Option<DateTime<Utc>>,

    /// Days since first run
    pub days_active: u32,
}

impl SelfStats {
    /// Create empty stats
    pub fn empty() -> Self {
        Self {
            collected_at: Utc::now(),
            total_queries: 0,
            queries_today: 0,
            queries_this_week: 0,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            total_llm_tokens: 0,
            llm_tokens_today: 0,
            total_actions: 0,
            successful_actions: 0,
            failed_actions: 0,
            action_success_rate: 0.0,
            total_rollbacks: 0,
            database_size_bytes: 0,
            uptime_seconds: 0,
            restart_count: 0,
            first_activity: None,
            days_active: 0,
        }
    }

    /// Format uptime as human-readable string
    pub fn uptime_human(&self) -> String {
        let secs = self.uptime_seconds;
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else if secs < 86400 {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        } else {
            let days = secs / 86400;
            let hours = (secs % 86400) / 3600;
            format!("{}d {}h", days, hours)
        }
    }

    /// Format database size as human-readable string
    pub fn db_size_human(&self) -> String {
        let bytes = self.database_size_bytes;
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Query activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    pub timestamp: DateTime<Utc>,
    pub query: String,
    pub intent: String,
    pub latency_ms: u64,
    pub llm_tokens: Option<u64>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Lifecycle timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: LifecycleEventType,
    pub details: String,
}

/// Types of lifecycle events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleEventType {
    /// Daemon started
    DaemonStart,
    /// Daemon stopped
    DaemonStop,
    /// Daemon crashed
    DaemonCrash,
    /// Configuration changed
    ConfigChange,
    /// Database migration
    DbMigration,
    /// Update installed
    UpdateInstalled,
    /// First run setup
    FirstRun,
}

impl LifecycleEventType {
    /// Get emoji for event type
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::DaemonStart => "🟢",
            Self::DaemonStop => "🔴",
            Self::DaemonCrash => "💥",
            Self::ConfigChange => "⚙️",
            Self::DbMigration => "🗃️",
            Self::UpdateInstalled => "⬆️",
            Self::FirstRun => "🎉",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DaemonStart => "Daemon started",
            Self::DaemonStop => "Daemon stopped",
            Self::DaemonCrash => "Daemon crashed",
            Self::ConfigChange => "Config changed",
            Self::DbMigration => "DB migration",
            Self::UpdateInstalled => "Update installed",
            Self::FirstRun => "First run",
        }
    }
}

/// Initialize the activity log tables
pub fn init_activity_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS activity_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            query TEXT NOT NULL,
            intent TEXT NOT NULL,
            latency_ms INTEGER NOT NULL,
            llm_tokens INTEGER,
            success INTEGER NOT NULL,
            error_message TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_activity_timestamp ON activity_log(timestamp)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS lifecycle_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            event_type TEXT NOT NULL,
            details TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_lifecycle_timestamp ON lifecycle_events(timestamp)",
        [],
    )?;

    Ok(())
}

/// Log a query activity
pub fn log_activity(conn: &Connection, entry: &ActivityLogEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_log (timestamp, query, intent, latency_ms, llm_tokens, success, error_message)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.timestamp.to_rfc3339(),
            entry.query,
            entry.intent,
            entry.latency_ms as i64,
            entry.llm_tokens.map(|t| t as i64),
            entry.success as i64,
            entry.error_message,
        ],
    )?;
    Ok(())
}

/// Log a lifecycle event
pub fn log_lifecycle_event(conn: &Connection, event: &LifecycleEvent) -> Result<()> {
    let event_type_str = format!("{:?}", event.event_type);
    conn.execute(
        "INSERT INTO lifecycle_events (timestamp, event_type, details)
         VALUES (?1, ?2, ?3)",
        params![
            event.timestamp.to_rfc3339(),
            event_type_str,
            event.details,
        ],
    )?;
    Ok(())
}

/// Collect self statistics from the database
pub fn collect_self_stats(conn: &Connection, db_path: &str) -> Result<SelfStats> {
    let mut stats = SelfStats::empty();

    // Initialize tables if needed
    init_activity_tables(conn)?;

    let now = Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let week_start = now - Duration::days(7);

    // Total queries
    stats.total_queries = conn
        .query_row("SELECT COUNT(*) FROM activity_log", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or(0) as u64;

    // Queries today
    stats.queries_today = conn
        .query_row(
            "SELECT COUNT(*) FROM activity_log WHERE timestamp >= ?1",
            params![today_start.to_string()],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64;

    // Queries this week
    stats.queries_this_week = conn
        .query_row(
            "SELECT COUNT(*) FROM activity_log WHERE timestamp >= ?1",
            params![week_start.to_rfc3339()],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64;

    // Average latency
    stats.avg_latency_ms = conn
        .query_row("SELECT AVG(latency_ms) FROM activity_log", [], |row| {
            row.get::<_, Option<f64>>(0)
        })
        .unwrap_or(None)
        .unwrap_or(0.0);

    // P95 latency (approximate)
    let total = stats.total_queries as usize;
    if total > 0 {
        let p95_idx = (total as f64 * 0.95) as i64;
        stats.p95_latency_ms = conn
            .query_row(
                "SELECT latency_ms FROM activity_log ORDER BY latency_ms DESC LIMIT 1 OFFSET ?1",
                params![total as i64 - p95_idx - 1],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0) as f64;
    }

    // Total LLM tokens
    stats.total_llm_tokens = conn
        .query_row(
            "SELECT COALESCE(SUM(llm_tokens), 0) FROM activity_log",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64;

    // LLM tokens today
    stats.llm_tokens_today = conn
        .query_row(
            "SELECT COALESCE(SUM(llm_tokens), 0) FROM activity_log WHERE timestamp >= ?1",
            params![today_start.to_string()],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64;

    // Action counts - check action_episodes table
    let actions_exist: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='action_episodes'",
            [],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )
        .unwrap_or(false);

    if actions_exist {
        stats.total_actions = conn
            .query_row("SELECT COUNT(*) FROM action_episodes", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap_or(0) as u64;

        stats.successful_actions = conn
            .query_row(
                "SELECT COUNT(*) FROM action_episodes WHERE status = 'completed'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0) as u64;

        stats.failed_actions = conn
            .query_row(
                "SELECT COUNT(*) FROM action_episodes WHERE status = 'failed'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0) as u64;

        if stats.total_actions > 0 {
            stats.action_success_rate =
                stats.successful_actions as f64 / stats.total_actions as f64;
        }
    }

    // Database size
    if let Ok(metadata) = std::fs::metadata(db_path) {
        stats.database_size_bytes = metadata.len();
    }

    // First activity
    if let Ok(first_ts) = conn.query_row(
        "SELECT MIN(timestamp) FROM activity_log",
        [],
        |row| row.get::<_, Option<String>>(0),
    ) {
        if let Some(ts_str) = first_ts {
            if let Ok(ts) = DateTime::parse_from_rfc3339(&ts_str) {
                stats.first_activity = Some(ts.with_timezone(&Utc));
                stats.days_active = (now - ts.with_timezone(&Utc)).num_days().max(1) as u32;
            }
        }
    }

    // Restart count from lifecycle events
    stats.restart_count = conn
        .query_row(
            "SELECT COUNT(*) FROM lifecycle_events WHERE event_type = 'DaemonStart'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64;

    Ok(stats)
}

/// Get usage by intent category
pub fn get_usage_by_intent(conn: &Connection) -> Result<HashMap<String, u64>> {
    let mut usage = HashMap::new();

    let mut stmt = conn.prepare(
        "SELECT intent, COUNT(*) FROM activity_log GROUP BY intent ORDER BY COUNT(*) DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        let intent: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        Ok((intent, count as u64))
    })?;

    for row in rows {
        if let Ok((intent, count)) = row {
            usage.insert(intent, count);
        }
    }

    Ok(usage)
}

/// Get recent lifecycle events
pub fn get_recent_lifecycle_events(conn: &Connection, limit: usize) -> Result<Vec<LifecycleEvent>> {
    let mut events = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT timestamp, event_type, details FROM lifecycle_events
         ORDER BY timestamp DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit as i64], |row| {
        let ts_str: String = row.get(0)?;
        let event_type_str: String = row.get(1)?;
        let details: String = row.get(2)?;

        Ok((ts_str, event_type_str, details))
    })?;

    for row in rows {
        if let Ok((ts_str, event_type_str, details)) = row {
            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let event_type = match event_type_str.as_str() {
                "DaemonStart" => LifecycleEventType::DaemonStart,
                "DaemonStop" => LifecycleEventType::DaemonStop,
                "DaemonCrash" => LifecycleEventType::DaemonCrash,
                "ConfigChange" => LifecycleEventType::ConfigChange,
                "DbMigration" => LifecycleEventType::DbMigration,
                "UpdateInstalled" => LifecycleEventType::UpdateInstalled,
                "FirstRun" => LifecycleEventType::FirstRun,
                _ => LifecycleEventType::DaemonStart,
            };

            events.push(LifecycleEvent {
                timestamp,
                event_type,
                details,
            });
        }
    }

    Ok(events)
}

/// Generate a human-readable self biography
pub fn generate_self_biography(stats: &SelfStats) -> String {
    let mut lines = Vec::new();

    lines.push("📊  Anna's Self Biography".to_string());
    lines.push(String::new());

    // Activity summary
    lines.push("📈  Activity Summary".to_string());
    lines.push(format!(
        "   Total queries handled: {}",
        format_number(stats.total_queries)
    ));
    lines.push(format!("   Queries today: {}", stats.queries_today));
    lines.push(format!("   Queries this week: {}", stats.queries_this_week));

    if stats.days_active > 0 {
        let avg_per_day = stats.total_queries as f64 / stats.days_active as f64;
        lines.push(format!("   Average per day: {:.1}", avg_per_day));
    }

    lines.push(String::new());

    // Performance
    lines.push("⚡  Performance".to_string());
    lines.push(format!("   Average latency: {:.0} ms", stats.avg_latency_ms));
    lines.push(format!("   P95 latency: {:.0} ms", stats.p95_latency_ms));

    lines.push(String::new());

    // LLM usage
    if stats.total_llm_tokens > 0 {
        lines.push("🧠  LLM Usage".to_string());
        lines.push(format!(
            "   Total tokens: {}",
            format_number(stats.total_llm_tokens)
        ));
        lines.push(format!(
            "   Tokens today: {}",
            format_number(stats.llm_tokens_today)
        ));
        lines.push(String::new());
    }

    // Actions
    if stats.total_actions > 0 {
        lines.push("🎯  Actions".to_string());
        lines.push(format!(
            "   Total actions: {}",
            format_number(stats.total_actions)
        ));
        lines.push(format!(
            "   Success rate: {:.1}%",
            stats.action_success_rate * 100.0
        ));
        lines.push(format!("   Rollbacks: {}", stats.total_rollbacks));
        lines.push(String::new());
    }

    // System
    lines.push("💾  System".to_string());
    lines.push(format!("   Database size: {}", stats.db_size_human()));
    lines.push(format!("   Restarts: {}", stats.restart_count));

    if let Some(first) = stats.first_activity {
        lines.push(format!("   First run: {}", first.format("%Y-%m-%d")));
        lines.push(format!("   Days active: {}", stats.days_active));
    }

    lines.join("\n")
}

/// Format a number with thousands separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptime_human() {
        let mut stats = SelfStats::empty();

        stats.uptime_seconds = 45;
        assert_eq!(stats.uptime_human(), "45s");

        stats.uptime_seconds = 125;
        assert_eq!(stats.uptime_human(), "2m 5s");

        stats.uptime_seconds = 3661;
        assert_eq!(stats.uptime_human(), "1h 1m");

        stats.uptime_seconds = 90061;
        assert_eq!(stats.uptime_human(), "1d 1h");
    }

    #[test]
    fn test_db_size_human() {
        let mut stats = SelfStats::empty();

        stats.database_size_bytes = 500;
        assert_eq!(stats.db_size_human(), "500 B");

        stats.database_size_bytes = 1536;
        assert_eq!(stats.db_size_human(), "1.5 KB");

        stats.database_size_bytes = 1572864;
        assert_eq!(stats.db_size_human(), "1.5 MB");

        stats.database_size_bytes = 1610612736;
        assert_eq!(stats.db_size_human(), "1.50 GB");
    }

    #[test]
    fn test_lifecycle_event_types() {
        assert_eq!(LifecycleEventType::DaemonStart.emoji(), "🟢");
        assert_eq!(LifecycleEventType::DaemonCrash.emoji(), "💥");
        assert_eq!(
            LifecycleEventType::FirstRun.display_name(),
            "First run"
        );
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_generate_biography() {
        let mut stats = SelfStats::empty();
        stats.total_queries = 1500;
        stats.queries_today = 25;
        stats.queries_this_week = 150;
        stats.avg_latency_ms = 85.0;
        stats.database_size_bytes = 5242880;
        stats.days_active = 30;

        let bio = generate_self_biography(&stats);
        assert!(bio.contains("Anna's Self Biography"));
        assert!(bio.contains("1,500"));
        assert!(bio.contains("25"));
        assert!(bio.contains("85 ms"));
        assert!(bio.contains("5.0 MB"));
    }

    #[test]
    fn test_activity_log_entry() {
        let entry = ActivityLogEntry {
            timestamp: Utc::now(),
            query: "What's my CPU?".to_string(),
            intent: "SystemStatus".to_string(),
            latency_ms: 120,
            llm_tokens: Some(500),
            success: true,
            error_message: None,
        };

        assert_eq!(entry.intent, "SystemStatus");
        assert!(entry.success);
    }

    #[test]
    fn test_init_and_log_activity() {
        let conn = Connection::open_in_memory().unwrap();
        init_activity_tables(&conn).unwrap();

        let entry = ActivityLogEntry {
            timestamp: Utc::now(),
            query: "test query".to_string(),
            intent: "Test".to_string(),
            latency_ms: 100,
            llm_tokens: None,
            success: true,
            error_message: None,
        };

        log_activity(&conn, &entry).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM activity_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_log_lifecycle_event() {
        let conn = Connection::open_in_memory().unwrap();
        init_activity_tables(&conn).unwrap();

        let event = LifecycleEvent {
            timestamp: Utc::now(),
            event_type: LifecycleEventType::DaemonStart,
            details: "Started successfully".to_string(),
        };

        log_lifecycle_event(&conn, &event).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM lifecycle_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_collect_empty_stats() {
        let conn = Connection::open_in_memory().unwrap();
        let stats = collect_self_stats(&conn, ":memory:").unwrap();

        assert_eq!(stats.total_queries, 0);
        assert_eq!(stats.queries_today, 0);
    }

    #[test]
    fn test_get_usage_by_intent() {
        let conn = Connection::open_in_memory().unwrap();
        init_activity_tables(&conn).unwrap();

        // Log some activities
        for intent in ["SystemStatus", "SystemStatus", "Help", "Report"] {
            let entry = ActivityLogEntry {
                timestamp: Utc::now(),
                query: "test".to_string(),
                intent: intent.to_string(),
                latency_ms: 100,
                llm_tokens: None,
                success: true,
                error_message: None,
            };
            log_activity(&conn, &entry).unwrap();
        }

        let usage = get_usage_by_intent(&conn).unwrap();
        assert_eq!(usage.get("SystemStatus"), Some(&2));
        assert_eq!(usage.get("Help"), Some(&1));
        assert_eq!(usage.get("Report"), Some(&1));
    }

    #[test]
    fn test_recent_lifecycle_events() {
        let conn = Connection::open_in_memory().unwrap();
        init_activity_tables(&conn).unwrap();

        // Log some events
        for event_type in [
            LifecycleEventType::DaemonStart,
            LifecycleEventType::ConfigChange,
            LifecycleEventType::DaemonStop,
        ] {
            let event = LifecycleEvent {
                timestamp: Utc::now(),
                event_type,
                details: "test".to_string(),
            };
            log_lifecycle_event(&conn, &event).unwrap();
        }

        let events = get_recent_lifecycle_events(&conn, 2).unwrap();
        assert_eq!(events.len(), 2);
    }
}
