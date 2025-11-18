//! Internal Suggestions Engine
//!
//! Generates prioritized suggestions for improving the system. Called by:
//! - REPL when user asks "what should I improve", "any problems", etc.
//! - annactl status to show top suggestions
//!
//! NOT exposed as a public CLI command.

use anna_common::terminal_format as fmt;
use anyhow::Result;

use crate::systemd;

/// A suggestion for improving the system
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Priority (1 = highest, critical)
    pub priority: u32,
    /// Short title
    pub title: String,
    /// Detailed explanation of why it matters
    pub explanation: String,
    /// What Anna can do to fix it
    pub recommended_fix: String,
    /// Reference to documentation
    pub reference: String,
}

/// Get all current suggestions, ordered by priority
///
/// This is idempotent - safe to call repeatedly
pub fn get_suggestions() -> Result<Vec<Suggestion>> {
    let mut suggestions = Vec::new();

    // Check 1: Daemon health
    if let Ok(status) = systemd::get_service_status() {
        if status.needs_repair() {
            suggestions.push(Suggestion {
                priority: 1, // Highest - Anna can't function without daemon
                title: "Anna daemon is not running properly".to_string(),
                explanation: format!(
                    "The Anna daemon ({}) monitors your system in the background. \
                     Without it, I cannot detect issues proactively, provide automated \
                     suggestions, or help you maintain your system. I can fix this for you.",
                    status.summary()
                ),
                recommended_fix: "I can enable and start the daemon service".to_string(),
                reference: "[archwiki:Systemd#Using_units]".to_string(),
            });
        }
    }

    // Check 2: System updates
    // TODO: Check for pending updates

    // Check 3: Disk space
    // TODO: Check disk usage

    // Check 4: Security updates
    // TODO: Check for security-critical updates

    // Check 5: Orphaned packages
    // TODO: Check for orphans

    // Sort by priority (lower number = higher priority)
    suggestions.sort_by_key(|s| s.priority);

    Ok(suggestions)
}

/// Display suggestions to user in conversational format
pub fn display_suggestions(suggestions: &[Suggestion]) {
    if suggestions.is_empty() {
        println!("{}", fmt::success("No urgent issues right now!"));
        println!();
        println!("Your system looks good. I'll let you know if I notice anything.");
        println!();
        return;
    }

    println!(
        "I found {} thing{} you might want to address:",
        suggestions.len(),
        if suggestions.len() == 1 { "" } else { "s" }
    );
    println!();

    for (i, suggestion) in suggestions.iter().enumerate() {
        let priority_icon = match suggestion.priority {
            1 => "🔴", // Critical
            2 => "🟡", // Warning
            _ => "ℹ️", // Info
        };

        println!(
            "{} {}",
            fmt::bold(&format!("{}. {}", i + 1, priority_icon)),
            fmt::bold(&suggestion.title)
        );
        println!();
        println!("   {}", suggestion.explanation);
        println!();
        println!(
            "   {} {}",
            fmt::bold("What I can do:"),
            suggestion.recommended_fix
        );
        println!(
            "   {} {}",
            fmt::dimmed("Reference:"),
            fmt::dimmed(&suggestion.reference)
        );
        println!();
    }
}

/// Get a short summary of top suggestions for status display
pub fn get_status_summary() -> Result<Option<String>> {
    let suggestions = get_suggestions()?;

    if suggestions.is_empty() {
        return Ok(None);
    }

    // Show only critical/high priority suggestions in status
    let critical: Vec<_> = suggestions.iter().filter(|s| s.priority <= 2).collect();

    if critical.is_empty() {
        return Ok(None);
    }

    let mut summary = String::new();
    summary.push_str(&format!("{}\n", fmt::bold("Top Suggestions:")));

    for (i, suggestion) in critical.iter().take(3).enumerate() {
        summary.push_str(&format!("  {}. {}\n", i + 1, suggestion.title));
    }

    if critical.len() > 3 {
        summary.push_str(&format!("  ... and {} more\n", critical.len() - 3));
    }

    summary.push_str(&format!(
        "\n  {} Ask me: \"what should I improve?\"\n",
        fmt::dimmed("To see details:")
    ));

    Ok(Some(summary))
}
