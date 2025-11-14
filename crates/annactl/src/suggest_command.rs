//! Suggestion command - show 2-5 prioritized actionable suggestions
//!
//! Real Anna: `annactl suggest`
//! Purpose: Show the top 2-5 most important suggestions for this machine
//! Each suggestion must:
//! - Be backed by Arch Wiki or reputable source
//! - Explain WHY it matters in plain language
//! - Include references or links
//! - Respect prerequisites (drivers before apps, security before cosmetics)

use anyhow::Result;
use anna_common::terminal_format as fmt;
use std::time::Instant;

use crate::errors::*;
use crate::logging::LogEntry;
use crate::systemd;

/// Execute 'suggest' command - show prioritized suggestions
pub async fn execute_suggest_command(
    json: bool,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    println!("{}", fmt::bold("Prioritized Suggestions"));
    println!("{}", "=".repeat(50));
    println!();

    let mut suggestions: Vec<Suggestion> = Vec::new();

    // Check 1: Daemon status
    if let Ok(status) = systemd::get_service_status() {
        if status.needs_repair() {
            suggestions.push(Suggestion {
                priority: 1, // Highest priority
                title: "Anna daemon is not running or not enabled".to_string(),
                description: format!(
                    "The Anna daemon ({}) monitors your system in the background. \
                     Without it, Anna cannot detect issues proactively or provide \
                     automated suggestions.",
                    status.summary()
                ),
                fix: "Run: annactl repair".to_string(),
                reference: "[archwiki:Systemd#Using_units]".to_string(),
            });
        }
    }

    // TODO: Add more suggestions
    // - Missing security updates
    // - Orphaned packages
    // - Full disk warnings
    // - etc.

    // Display suggestions
    if suggestions.is_empty() {
        println!("{}", fmt::success("No urgent suggestions at this time."));
        println!();
        println!("{}", fmt::dimmed("Your system looks good! Anna will notify you if issues are detected."));
        println!();
    } else {
        println!("{}", fmt::dimmed(&format!("Found {} suggestion(s):", suggestions.len())));
        println!();

        for (i, suggestion) in suggestions.iter().enumerate() {
            println!("{}", fmt::bold(&format!("{}. {}", i + 1, suggestion.title)));
            println!();
            println!("   {}", suggestion.description);
            println!();
            println!("   {} {}", fmt::bold("Fix:"), suggestion.fix);
            println!("   {} {}", fmt::dimmed("Ref:"), fmt::dimmed(&suggestion.reference));
            println!();
        }
    }

    // Log command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "suggest".to_string(),
        allowed: Some(true),
        args: if json { vec!["--json".to_string()] } else { vec![] },
        exit_code: EXIT_SUCCESS,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: true,
        error: None,
    };
    let _ = log_entry.write();

    Ok(())
}

/// A suggestion for improving the system
struct Suggestion {
    /// Priority (1 = highest)
    priority: u32,
    /// Short title
    title: String,
    /// Detailed description explaining why it matters
    description: String,
    /// How to fix it
    fix: String,
    /// Reference to documentation
    reference: String,
}
