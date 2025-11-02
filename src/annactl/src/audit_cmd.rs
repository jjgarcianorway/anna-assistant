//! Audit command for Anna v0.14.0 "Orion III"
//!
//! View and analyze audit logs

use anyhow::Result;
use anna_common::{header, section, TermCaps};
use std::path::PathBuf;

use crate::audit_log::AuditLog;

/// Audit command mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditMode {
    Summary,
    Last { n: usize },
    Actor { actor: String },
    Export { path: PathBuf },
    Json,
}

/// Run audit command
pub fn run_audit(mode: AuditMode) -> Result<()> {
    let audit = AuditLog::new()?;

    match mode {
        AuditMode::Summary => {
            let summary = audit.get_summary()?;
            print_summary(&summary);
        }
        AuditMode::Last { n } => {
            let entries = audit.get_last(n)?;
            print_entries(&entries, false);
        }
        AuditMode::Actor { actor } => {
            let entries = audit.get_by_actor(&actor)?;
            print_entries(&entries, false);
        }
        AuditMode::Export { path } => {
            audit.export(&path)?;
            println!("✅ Audit log exported to: {}", path.display());
        }
        AuditMode::Json => {
            let entries = audit.load_all()?;
            let json_str = serde_json::to_string_pretty(&entries)?;
            println!("{}", json_str);
        }
    }

    Ok(())
}

/// Print audit summary
fn print_summary(summary: &crate::audit_log::AuditSummary) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Audit Log Summary"));
    println!();

    println!("  Total Entries: {}", summary.total_entries);
    println!();

    // By actor
    println!("{}", section(&caps, "By Actor"));
    println!();

    let mut actors: Vec<_> = summary.by_actor.iter().collect();
    actors.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    for (actor, count) in actors {
        let emoji = match actor.as_str() {
            "auto" => "🤖",
            "user" => "👤",
            "advisor" => "🧠",
            "scheduler" => "⏰",
            _ => "📋",
        };
        println!("  {} {:<12} {}", emoji, actor, count);
    }
    println!();

    // By result
    println!("{}", section(&caps, "By Result"));
    println!();

    let mut results: Vec<_> = summary.by_result.iter().collect();
    results.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    for (result, count) in results {
        let emoji = match result.as_str() {
            "success" => "✅",
            "fail" => "❌",
            "pending" => "⏳",
            "ignored" => "🔇",
            _ => "❓",
        };
        println!("  {} {:<12} {}", emoji, result, count);
    }
    println!();

    // By action type
    println!("{}", section(&caps, "By Action Type"));
    println!();

    let mut types: Vec<_> = summary.by_action_type.iter().collect();
    types.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    for (action_type, count) in types {
        println!("  {:<12} {}", action_type, count);
    }
    println!();
}

/// Print audit entries
fn print_entries(entries: &[crate::audit_log::AuditEntry], verbose: bool) {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Audit Log Entries"));
    println!();

    if entries.is_empty() {
        println!("No entries found");
        return;
    }

    for entry in entries {
        println!("{}", section(&caps, &format_timestamp(entry.timestamp)));
        println!();

        println!("  Actor:   {} {}", entry.actor_emoji(), entry.actor);
        println!("  Action:  {}", entry.action);
        println!("  Type:    {}", entry.action_type);
        println!("  Result:  {} {}", entry.result_emoji(), entry.result);

        if let Some(ref impact) = entry.impact {
            println!("  Impact:  {}", impact);
        }

        if let Some(ref rollback) = entry.rollback_cmd {
            println!("  Rollback: {}", rollback);
        }

        if verbose {
            if let Some(ref details) = entry.details {
                println!("  Details: {}", details);
            }

            if let Some(ref action_id) = entry.related_action_id {
                println!("  Action ID: {}", action_id);
            }
        }

        println!();
    }

    println!("Total: {} entries", entries.len());
}

/// Format UNIX timestamp as human-readable date
fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Utc, TimeZone};

    let dt: DateTime<Utc> = Utc.timestamp_opt(timestamp as i64, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp() {
        let timestamp = 1699000000u64;
        let formatted = format_timestamp(timestamp);

        assert!(formatted.contains("2023"));
        assert!(formatted.contains("UTC"));
    }
}
