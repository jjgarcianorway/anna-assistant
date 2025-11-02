//! Snapshot diff command for annactl
//!
//! Compare telemetry snapshots to visualize system state changes

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::io::{self, Write};

/// Snapshot diff change type (mirrored from annad)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiffChange {
    Added { value: String },
    Removed { value: String },
    Modified { old_value: String, new_value: String },
    Unchanged { value: String },
}

/// Diff node structure (mirrored from annad)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffNode {
    pub path: String,
    pub change: DiffChange,
    pub children: Vec<DiffNode>,
    pub metadata: Option<DiffMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffMetadata {
    pub field_type: String,
    pub delta: Option<f64>,
    pub delta_pct: Option<f64>,
    pub severity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_fields: usize,
    pub added_count: usize,
    pub removed_count: usize,
    pub modified_count: usize,
    pub unchanged_count: usize,
    pub significant_changes: usize,
    pub time_delta_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub nodes: Vec<DiffNode>,
    pub summary: DiffSummary,
    pub old_timestamp: String,
    pub new_timestamp: String,
}

/// Display snapshot diff with beautiful TUI
pub fn display_diff(diff: &SnapshotDiff, json_output: bool, show_unchanged: bool) -> Result<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(diff)?);
        return Ok(());
    }

    // Beautiful TUI output
    print_diff_header(diff)?;
    print_diff_summary(&diff.summary)?;
    print_diff_tree(&diff.nodes, show_unchanged, 0)?;
    print_diff_footer()?;

    Ok(())
}

fn print_diff_header(diff: &SnapshotDiff) -> Result<()> {
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";

    println!("{}╭─ Snapshot Diff ──────────────────────────────────────────{}", dim, reset);
    println!("{}│{}", dim, reset);
    println!("{}│{}  {}Before:{}  {}", dim, reset, bold, reset, diff.old_timestamp);
    println!("{}│{}  {}After:{}   {}", dim, reset, bold, reset, diff.new_timestamp);
    println!("{}│{}", dim, reset);

    Ok(())
}

fn print_diff_summary(summary: &DiffSummary) -> Result<()> {
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let green = "\x1b[32m";
    let red = "\x1b[31m";
    let yellow = "\x1b[33m";

    println!("{}│{}  {}Summary:{}", dim, reset, bold, reset);
    println!(
        "{}│{}  • Total Fields:        {}",
        dim, reset, summary.total_fields
    );

    if summary.added_count > 0 {
        println!(
            "{}│{}  • {}Added:{}              {} fields",
            dim, reset, green, reset, summary.added_count
        );
    }

    if summary.removed_count > 0 {
        println!(
            "{}│{}  • {}Removed:{}            {} fields",
            dim, reset, red, reset, summary.removed_count
        );
    }

    if summary.modified_count > 0 {
        println!(
            "{}│{}  • {}Modified:{}           {} fields",
            dim, reset, yellow, reset, summary.modified_count
        );
    }

    if summary.unchanged_count > 0 {
        println!(
            "{}│{}  • Unchanged:          {} fields",
            dim, reset, summary.unchanged_count
        );
    }

    if summary.significant_changes > 0 {
        println!(
            "{}│{}  • {}Significant Changes:{} {}",
            dim, reset, bold, reset, summary.significant_changes
        );
    }

    println!("{}│{}", dim, reset);

    Ok(())
}

fn print_diff_tree(nodes: &[DiffNode], show_unchanged: bool, depth: usize) -> Result<()> {
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";

    for node in nodes {
        // Skip unchanged if not requested
        if !show_unchanged && matches!(node.change, DiffChange::Unchanged { .. }) {
            continue;
        }

        let indent = "  ".repeat(depth);
        let (symbol, color, value_str) = format_change(&node.change);

        // Print path with change indicator
        println!(
            "{}│{}{}  {}{} {}{}{}",
            dim, reset, indent, color, symbol, node.path, reset, dim
        );

        // Print delta if available
        if let Some(ref meta) = node.metadata {
            if let Some(delta) = meta.delta {
                let delta_color = if delta > 0.0 { "\x1b[32m" } else { "\x1b[31m" };
                println!(
            "{}│{}{}    {}Δ {}{:+.2}{} ({}{:+.1}%{})",
                    dim,
                    reset,
                    indent,
                    dim,
                    delta_color,
                    delta,
                    reset,
                    delta_color,
                    meta.delta_pct.unwrap_or(0.0),
                    reset
                );
            }

            // Print severity warning for significant changes
            if meta.severity >= 0.7 {
                let severity_color = if meta.severity >= 0.9 {
                    "\x1b[35m" // Magenta for critical
                } else {
                    "\x1b[33m" // Yellow for warning
                };
                let severity_label = if meta.severity >= 0.9 { "CRITICAL" } else { "WARNING" };
                println!(
                    "{}│{}{}    {}⚠ {}{}{}",
                    dim, reset, indent, severity_color, severity_label, reset, dim
                );
            }
        }

        // Print value details
        println!("{}│{}{}    {}{}", dim, reset, indent, value_str, dim);

        // Recursively print children
        if !node.children.is_empty() {
            print_diff_tree(&node.children, show_unchanged, depth + 1)?;
        }
    }

    Ok(())
}

fn format_change(change: &DiffChange) -> (&'static str, &'static str, String) {
    match change {
        DiffChange::Added { value } => {
            ("✓", "\x1b[32m", format!("    + {}", value))
        }
        DiffChange::Removed { value } => {
            ("✗", "\x1b[31m", format!("    - {}", value))
        }
        DiffChange::Modified { old_value, new_value } => {
            ("~", "\x1b[33m", format!("    {} → {}", old_value, new_value))
        }
        DiffChange::Unchanged { value } => {
            ("=", "\x1b[2m", format!("    = {}", value))
        }
    }
}

fn print_diff_footer() -> Result<()> {
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";

    println!("{}│{}", dim, reset);
    println!("{}╰──────────────────────────────────────────────────────────{}", dim, reset);

    Ok(())
}

/// Mock function for now - will be replaced with actual RPC call
pub async fn fetch_and_compare_snapshots(
    _snapshot_id_1: &str,
    _snapshot_id_2: &str,
) -> Result<SnapshotDiff> {
    // TODO: Implement actual RPC call to daemon
    // For now, return a mock diff
    Ok(SnapshotDiff {
        nodes: vec![],
        summary: DiffSummary {
            total_fields: 0,
            added_count: 0,
            removed_count: 0,
            modified_count: 0,
            unchanged_count: 0,
            significant_changes: 0,
            time_delta_secs: 0,
        },
        old_timestamp: "2025-11-02T10:00:00Z".to_string(),
        new_timestamp: "2025-11-02T10:05:00Z".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_change_added() {
        let change = DiffChange::Added {
            value: "new_field".to_string(),
        };
        let (symbol, _color, _value) = format_change(&change);
        assert_eq!(symbol, "✓");
    }

    #[test]
    fn test_format_change_removed() {
        let change = DiffChange::Removed {
            value: "old_field".to_string(),
        };
        let (symbol, _color, _value) = format_change(&change);
        assert_eq!(symbol, "✗");
    }

    #[test]
    fn test_format_change_modified() {
        let change = DiffChange::Modified {
            old_value: "10".to_string(),
            new_value: "20".to_string(),
        };
        let (symbol, _color, value) = format_change(&change);
        assert_eq!(symbol, "~");
        assert!(value.contains("10 → 20"));
    }
}
