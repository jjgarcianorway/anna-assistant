//! Knowledge Introspection - Query Anna's knowledge base
//!
//! v6.55.1: Knowledge Backup, Export, Introspection and Pruning
//!
//! This module provides introspection capabilities:
//! - Query what Anna knows about the system
//! - Get statistics per knowledge domain
//! - Generate knowledge summaries for user queries

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use std::collections::HashMap;

use crate::knowledge_domain::{DomainStats, KnowledgeDomain, KnowledgeSummary};

/// Query knowledge statistics for all domains
pub fn query_knowledge_summary(conn: &Connection, db_location: &str) -> Result<KnowledgeSummary> {
    let mut summary = KnowledgeSummary::empty(db_location.to_string());

    for domain in KnowledgeDomain::all() {
        let stats = query_domain_stats(conn, domain)?;
        summary.total_records += stats.record_count;
        summary.total_size_bytes += stats.size_bytes;
        summary.domains.insert(domain, stats);
    }

    Ok(summary)
}

/// Query statistics for a specific domain
pub fn query_domain_stats(conn: &Connection, domain: KnowledgeDomain) -> Result<DomainStats> {
    let mut stats = DomainStats::empty(domain);

    for table in domain.tables() {
        // Check if table exists
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        if !exists {
            continue;
        }

        // Get record count
        let count_sql = format!("SELECT COUNT(*) FROM {}", table);
        if let Ok(count) = conn.query_row(&count_sql, [], |row| row.get::<_, i64>(0)) {
            stats.record_count += count as u64;
        }

        // Try to get timestamp range if table has a timestamp column
        let timestamp_cols = ["ts", "timestamp", "window_start", "ts_start", "created_at"];
        for ts_col in timestamp_cols {
            // Check if column exists
            let col_exists: bool = conn
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name=?1",
                        table
                    ),
                    [ts_col],
                    |row| {
                        let count: i64 = row.get(0)?;
                        Ok(count > 0)
                    },
                )
                .unwrap_or(false);

            if !col_exists {
                continue;
            }

            // Get oldest and newest timestamps
            let oldest_sql = format!(
                "SELECT MIN({}) FROM {} WHERE {} IS NOT NULL",
                ts_col, table, ts_col
            );
            if let Ok(oldest_str) = conn.query_row(&oldest_sql, [], |row| {
                row.get::<_, Option<String>>(0)
            }) {
                if let Some(ts_str) = oldest_str {
                    if let Ok(ts) = parse_timestamp(&ts_str) {
                        if stats.oldest_record.is_none() || Some(ts) < stats.oldest_record {
                            stats.oldest_record = Some(ts);
                        }
                    }
                }
            }

            let newest_sql = format!(
                "SELECT MAX({}) FROM {} WHERE {} IS NOT NULL",
                ts_col, table, ts_col
            );
            if let Ok(newest_str) = conn.query_row(&newest_sql, [], |row| {
                row.get::<_, Option<String>>(0)
            }) {
                if let Some(ts_str) = newest_str {
                    if let Ok(ts) = parse_timestamp(&ts_str) {
                        if stats.newest_record.is_none() || Some(ts) > stats.newest_record {
                            stats.newest_record = Some(ts);
                        }
                    }
                }
            }

            break; // Found timestamp column, no need to check others
        }
    }

    // Calculate days of history
    if let (Some(oldest), Some(newest)) = (stats.oldest_record, stats.newest_record) {
        let duration = newest - oldest;
        stats.days_of_history = duration.num_days().max(0) as u32;
    }

    // Estimate size based on record count and domain
    stats.size_bytes = estimate_domain_size(domain, stats.record_count);

    Ok(stats)
}

/// Estimate storage size for a domain based on record count
fn estimate_domain_size(domain: KnowledgeDomain, record_count: u64) -> u64 {
    // Average bytes per record for each domain type
    let bytes_per_record = match domain {
        KnowledgeDomain::Telemetry => 200,
        KnowledgeDomain::DynamicPaths => 500,
        KnowledgeDomain::Watches => 300,
        KnowledgeDomain::UndoHistory => 1000,
        KnowledgeDomain::UserProfile => 500,
        KnowledgeDomain::MachineIdentity => 400,
        KnowledgeDomain::ServiceHealth => 150,
        KnowledgeDomain::NetworkTelemetry => 180,
        KnowledgeDomain::BootSessions => 200,
        KnowledgeDomain::LogSignatures => 300,
        KnowledgeDomain::LlmUsage => 250,
        KnowledgeDomain::Baselines => 500,
        KnowledgeDomain::UsagePatterns => 150,
        KnowledgeDomain::IssueTracking => 400,
        KnowledgeDomain::LearningPatterns => 600,
        KnowledgeDomain::TimelineEvents => 350,
    };

    record_count * bytes_per_record
}

/// Parse a timestamp string from the database
fn parse_timestamp(ts_str: &str) -> Result<DateTime<Utc>> {
    // Try ISO 8601 format first
    if let Ok(ts) = DateTime::parse_from_rfc3339(ts_str) {
        return Ok(ts.with_timezone(&Utc));
    }

    // Try SQLite datetime format
    if let Ok(ts) = chrono::NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d %H:%M:%S") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(ts, Utc));
    }

    // Try just date
    if let Ok(date) = chrono::NaiveDate::parse_from_str(ts_str, "%Y-%m-%d") {
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc));
    }

    anyhow::bail!("Could not parse timestamp: {}", ts_str)
}

/// Get a human-readable description of what Anna knows
pub fn describe_knowledge(summary: &KnowledgeSummary) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "🧠  Anna's Knowledge Summary ({})",
        summary.formatted_total_size()
    ));
    lines.push(format!("📍  Location: {}", summary.db_location));
    lines.push(format!(
        "📊  Total records: {}",
        format_number(summary.total_records)
    ));
    lines.push(String::new());

    // Group by category
    let by_category = summary.by_category();

    for category in [
        crate::knowledge_domain::KnowledgeCategory::SystemMetrics,
        crate::knowledge_domain::KnowledgeCategory::FileSystem,
        crate::knowledge_domain::KnowledgeCategory::ActionHistory,
        crate::knowledge_domain::KnowledgeCategory::UserBehavior,
        crate::knowledge_domain::KnowledgeCategory::Identity,
        crate::knowledge_domain::KnowledgeCategory::AnnaInternal,
    ] {
        if let Some(stats) = by_category.get(&category) {
            if stats.iter().any(|s| s.record_count > 0) {
                lines.push(format!("{}  {}", category.emoji(), category.display_name()));

                for stat in stats {
                    if stat.record_count > 0 {
                        let days = if stat.days_of_history > 0 {
                            format!(" ({} days)", stat.days_of_history)
                        } else {
                            String::new()
                        };
                        lines.push(format!(
                            "   {}  {}: {} records{}",
                            stat.domain.emoji(),
                            stat.domain.display_name(),
                            format_number(stat.record_count),
                            days
                        ));
                    }
                }
                lines.push(String::new());
            }
        }
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

/// Query what Anna knows about a specific topic
pub fn query_knowledge_about(
    conn: &Connection,
    topic: &str,
) -> Result<Vec<KnowledgeAboutItem>> {
    let mut results = Vec::new();
    let topic_lower = topic.to_lowercase();

    // Search for relevant knowledge based on topic
    if topic_lower.contains("cpu") || topic_lower.contains("processor") {
        results.extend(query_cpu_knowledge(conn)?);
    }

    if topic_lower.contains("memory") || topic_lower.contains("ram") {
        results.extend(query_memory_knowledge(conn)?);
    }

    if topic_lower.contains("disk") || topic_lower.contains("storage") {
        results.extend(query_disk_knowledge(conn)?);
    }

    if topic_lower.contains("network") || topic_lower.contains("internet") {
        results.extend(query_network_knowledge(conn)?);
    }

    if topic_lower.contains("service") || topic_lower.contains("systemd") {
        results.extend(query_service_knowledge(conn)?);
    }

    if topic_lower.contains("boot") || topic_lower.contains("startup") {
        results.extend(query_boot_knowledge(conn)?);
    }

    if topic_lower.contains("error") || topic_lower.contains("log") {
        results.extend(query_log_knowledge(conn)?);
    }

    // Generic machine knowledge
    if results.is_empty()
        || topic_lower.contains("machine")
        || topic_lower.contains("system")
        || topic_lower.contains("computer")
    {
        results.extend(query_machine_knowledge(conn)?);
    }

    Ok(results)
}

/// A piece of knowledge about a topic
#[derive(Debug, Clone)]
pub struct KnowledgeAboutItem {
    pub domain: KnowledgeDomain,
    pub description: String,
    pub value: String,
    pub confidence: f64,
    pub last_updated: Option<DateTime<Utc>>,
}

fn query_cpu_knowledge(conn: &Connection) -> Result<Vec<KnowledgeAboutItem>> {
    let mut items = Vec::new();

    // Check if we have CPU telemetry
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM cpu_windows", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::Telemetry,
            description: "CPU usage history".to_string(),
            value: format!("{} data points collected", count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    Ok(items)
}

fn query_memory_knowledge(conn: &Connection) -> Result<Vec<KnowledgeAboutItem>> {
    let mut items = Vec::new();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM mem_windows", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::Telemetry,
            description: "Memory usage history".to_string(),
            value: format!("{} data points collected", count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    let oom_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM oom_events", [], |row| row.get(0))
        .unwrap_or(0);

    if oom_count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::Telemetry,
            description: "OOM events".to_string(),
            value: format!("{} out-of-memory events recorded", oom_count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    Ok(items)
}

fn query_disk_knowledge(conn: &Connection) -> Result<Vec<KnowledgeAboutItem>> {
    let mut items = Vec::new();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM fs_capacity_daily", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    if count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::Telemetry,
            description: "Disk capacity history".to_string(),
            value: format!("{} data points", count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    Ok(items)
}

fn query_network_knowledge(conn: &Connection) -> Result<Vec<KnowledgeAboutItem>> {
    let mut items = Vec::new();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM net_windows", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::NetworkTelemetry,
            description: "Network telemetry".to_string(),
            value: format!("{} data points", count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    Ok(items)
}

fn query_service_knowledge(conn: &Connection) -> Result<Vec<KnowledgeAboutItem>> {
    let mut items = Vec::new();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM service_health", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::ServiceHealth,
            description: "Service health records".to_string(),
            value: format!("{} records", count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    Ok(items)
}

fn query_boot_knowledge(conn: &Connection) -> Result<Vec<KnowledgeAboutItem>> {
    let mut items = Vec::new();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM boot_sessions", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::BootSessions,
            description: "Boot sessions".to_string(),
            value: format!("{} boot sessions recorded", count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    Ok(items)
}

fn query_log_knowledge(conn: &Connection) -> Result<Vec<KnowledgeAboutItem>> {
    let mut items = Vec::new();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM log_signatures", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::LogSignatures,
            description: "Known log patterns".to_string(),
            value: format!("{} unique signatures", count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    Ok(items)
}

fn query_machine_knowledge(conn: &Connection) -> Result<Vec<KnowledgeAboutItem>> {
    let mut items = Vec::new();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM system_state_log", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    if count > 0 {
        items.push(KnowledgeAboutItem {
            domain: KnowledgeDomain::MachineIdentity,
            description: "System state snapshots".to_string(),
            value: format!("{} snapshots", count),
            confidence: 1.0,
            last_updated: None,
        });
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_parse_timestamp() {
        let ts = parse_timestamp("2025-11-25 12:00:00").unwrap();
        assert_eq!(ts.year(), 2025);

        let ts2 = parse_timestamp("2025-11-25").unwrap();
        assert_eq!(ts2.year(), 2025);
    }

    #[test]
    fn test_estimate_domain_size() {
        let size = estimate_domain_size(KnowledgeDomain::Telemetry, 1000);
        assert!(size > 0);
    }
}
