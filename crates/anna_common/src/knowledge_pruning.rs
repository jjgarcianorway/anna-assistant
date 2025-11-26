//! Knowledge Pruning - Clean up old knowledge
//!
//! v6.55.1: Knowledge Backup, Export, Introspection and Pruning
//!
//! This module provides pruning capabilities:
//! - Delete old records based on age
//! - Selective pruning by domain
//! - Dry run mode for preview

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use std::collections::HashMap;

use crate::knowledge_domain::{KnowledgeDomain, PruningCriteria, PruningResult};

/// Execute pruning based on criteria
pub fn prune_knowledge(conn: &Connection, criteria: &PruningCriteria) -> Result<PruningResult> {
    let mut result = PruningResult::empty(criteria.dry_run);

    // Calculate cutoff date
    let cutoff = Utc::now() - Duration::days(criteria.older_than_days as i64);

    // Determine which domains to prune
    let domains_to_prune = if let Some(ref domains) = criteria.domains {
        domains.clone()
    } else {
        KnowledgeDomain::all()
            .into_iter()
            .filter(|d| d.supports_time_pruning())
            .collect()
    };

    for domain in domains_to_prune {
        if !domain.supports_time_pruning() {
            result.errors.push(format!(
                "Domain '{}' does not support time-based pruning",
                domain.display_name()
            ));
            continue;
        }

        let deleted = prune_domain(conn, domain, cutoff, criteria.dry_run)?;
        if deleted > 0 {
            result.deleted_per_domain.insert(domain, deleted);
            result.total_deleted += deleted;
        }
    }

    // Estimate space reclaimed (rough estimate)
    result.space_reclaimed_bytes = estimate_space_reclaimed(&result.deleted_per_domain);

    // Run VACUUM if not dry run and we deleted something
    if !criteria.dry_run && result.total_deleted > 0 {
        if let Err(e) = conn.execute("VACUUM", []) {
            result.errors.push(format!("Failed to vacuum database: {}", e));
        }
    }

    Ok(result)
}

/// Prune a single domain
fn prune_domain(
    conn: &Connection,
    domain: KnowledgeDomain,
    cutoff: DateTime<Utc>,
    dry_run: bool,
) -> Result<u64> {
    let mut total_deleted = 0u64;
    let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

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

        // Find timestamp column
        let timestamp_col = find_timestamp_column(conn, table)?;
        let Some(ts_col) = timestamp_col else {
            continue;
        };

        if dry_run {
            // Count records that would be deleted
            let count_sql = format!(
                "SELECT COUNT(*) FROM {} WHERE {} < ?1",
                table, ts_col
            );
            let count: i64 = conn
                .query_row(&count_sql, [&cutoff_str], |row| row.get(0))
                .unwrap_or(0);
            total_deleted += count as u64;
        } else {
            // Actually delete the records
            let delete_sql = format!(
                "DELETE FROM {} WHERE {} < ?1",
                table, ts_col
            );
            let deleted = conn.execute(&delete_sql, [&cutoff_str])?;
            total_deleted += deleted as u64;
        }
    }

    Ok(total_deleted)
}

/// Find the timestamp column in a table
fn find_timestamp_column(conn: &Connection, table: &str) -> Result<Option<String>> {
    let timestamp_cols = [
        "ts",
        "timestamp",
        "window_start",
        "ts_start",
        "created_at",
        "set_at",
        "detected_at",
        "first_seen",
    ];

    for col in timestamp_cols {
        let exists: bool = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name=?1",
                    table
                ),
                [col],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        if exists {
            return Ok(Some(col.to_string()));
        }
    }

    Ok(None)
}

/// Estimate space reclaimed based on deleted records
fn estimate_space_reclaimed(deleted: &HashMap<KnowledgeDomain, u64>) -> u64 {
    let mut total = 0u64;

    for (domain, count) in deleted {
        let bytes_per_record = match domain {
            KnowledgeDomain::Telemetry => 200,
            KnowledgeDomain::DynamicPaths => 500,
            KnowledgeDomain::Watches => 300,
            KnowledgeDomain::UndoHistory => 1000,
            KnowledgeDomain::ServiceHealth => 150,
            KnowledgeDomain::NetworkTelemetry => 180,
            KnowledgeDomain::BootSessions => 200,
            KnowledgeDomain::LogSignatures => 300,
            KnowledgeDomain::LlmUsage => 250,
            KnowledgeDomain::Baselines => 500,
            KnowledgeDomain::UsagePatterns => 150,
            KnowledgeDomain::IssueTracking => 400,
            KnowledgeDomain::TimelineEvents => 350,
            _ => 200, // Default
        };
        total += count * bytes_per_record;
    }

    total
}

/// Get a human-readable pruning preview
pub fn describe_pruning_preview(result: &PruningResult) -> String {
    if result.total_deleted == 0 {
        return "📭  No records would be deleted".to_string();
    }

    let mut lines = Vec::new();

    if result.was_dry_run {
        lines.push("🔍  Pruning Preview (dry run)".to_string());
    } else {
        lines.push("🧹  Pruning Complete".to_string());
    }

    lines.push(String::new());

    for (domain, count) in &result.deleted_per_domain {
        lines.push(format!(
            "   {}  {}: {} records",
            domain.emoji(),
            domain.display_name(),
            format_number(*count)
        ));
    }

    lines.push(String::new());
    lines.push(format!(
        "📊  Total: {} records",
        format_number(result.total_deleted)
    ));
    lines.push(format!(
        "💾  Space reclaimed: {}",
        format_size(result.space_reclaimed_bytes)
    ));

    if !result.errors.is_empty() {
        lines.push(String::new());
        lines.push("⚠️  Errors:".to_string());
        for error in &result.errors {
            lines.push(format!("   • {}", error));
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

/// Format bytes as human-readable size
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Parse natural language pruning request
pub fn parse_pruning_request(input: &str) -> Option<PruningCriteria> {
    let lower = input.to_lowercase();

    // Extract days from input
    let days = extract_days(&lower)?;

    // Extract domains if specified
    let domains = extract_domains(&lower);

    Some(PruningCriteria {
        domains,
        older_than_days: days,
        dry_run: !lower.contains("confirm")
            && !lower.contains("yes")
            && !lower.contains("really")
            && !lower.contains("actually"),
    })
}

/// Extract number of days from natural language
fn extract_days(input: &str) -> Option<u32> {
    // Common patterns
    if input.contains("30 day") {
        return Some(30);
    }
    if input.contains("60 day") {
        return Some(60);
    }
    if input.contains("90 day") {
        return Some(90);
    }
    if input.contains("180 day") || input.contains("6 month") {
        return Some(180);
    }
    if input.contains("365 day") || input.contains("1 year") || input.contains("a year") {
        return Some(365);
    }

    // Try to extract number + "days"
    let words: Vec<&str> = input.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if *word == "days" && i > 0 {
            if let Ok(num) = words[i - 1].parse::<u32>() {
                return Some(num);
            }
        }
    }

    // Default: 90 days if just "prune" or "forget" without specific days
    if input.contains("prune")
        || input.contains("forget")
        || input.contains("clean")
        || input.contains("delete")
    {
        return Some(90);
    }

    None
}

/// Extract specific domains from natural language
fn extract_domains(input: &str) -> Option<Vec<KnowledgeDomain>> {
    let lower = input.to_lowercase();

    let mut domains = Vec::new();

    if lower.contains("telemetry") || lower.contains("metrics") {
        domains.push(KnowledgeDomain::Telemetry);
    }
    if lower.contains("network") {
        domains.push(KnowledgeDomain::NetworkTelemetry);
    }
    if lower.contains("boot") || lower.contains("startup") {
        domains.push(KnowledgeDomain::BootSessions);
    }
    if lower.contains("service") {
        domains.push(KnowledgeDomain::ServiceHealth);
    }
    if lower.contains("log") || lower.contains("error") {
        domains.push(KnowledgeDomain::LogSignatures);
    }
    if lower.contains("llm") || lower.contains("brain") {
        domains.push(KnowledgeDomain::LlmUsage);
    }
    if lower.contains("history") || lower.contains("undo") {
        domains.push(KnowledgeDomain::UndoHistory);
    }

    if domains.is_empty() {
        None // All pruneable domains
    } else {
        Some(domains)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pruning_request_days() {
        let criteria = parse_pruning_request("forget telemetry older than 90 days").unwrap();
        assert_eq!(criteria.older_than_days, 90);
        assert!(criteria.dry_run); // No confirmation

        let criteria = parse_pruning_request("delete data older than 30 days confirm").unwrap();
        assert_eq!(criteria.older_than_days, 30);
        assert!(!criteria.dry_run); // Has confirmation
    }

    #[test]
    fn test_parse_pruning_request_domains() {
        let criteria = parse_pruning_request("prune telemetry older than 90 days").unwrap();
        assert!(criteria.domains.is_some());
        let domains = criteria.domains.unwrap();
        assert!(domains.contains(&KnowledgeDomain::Telemetry));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_estimate_space_reclaimed() {
        let mut deleted = HashMap::new();
        deleted.insert(KnowledgeDomain::Telemetry, 1000);
        let space = estimate_space_reclaimed(&deleted);
        assert!(space > 0);
    }
}
