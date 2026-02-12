//! Proactive Suggestions System.
//! Anna analyzes system state and learned patterns to suggest improvements.

mod types;
mod system;
mod services;
mod security;
mod performance;

pub use types::{Suggestion, SuggestionPriority, SuggestionsState, format_suggestions};

use anyhow::Result;
use tracing::{debug, info};

/// Scan system for proactive suggestions.
pub async fn scan_for_suggestions() -> Result<Vec<Suggestion>> {
    let mut suggestions = Vec::new();

    debug!("Scanning for proactive suggestions");

    // System checks
    if let Some(s) = system::check_pacman_cache().await {
        suggestions.push(s);
    }

    if let Some(s) = system::check_orphaned_packages().await {
        suggestions.push(s);
    }

    if let Some(s) = system::check_disk_trends().await {
        suggestions.push(s);
    }

    if let Some(s) = system::check_telegram_setup().await {
        suggestions.push(s);
    }

    // Service checks
    if let Some(s) = services::check_recurring_failures().await {
        suggestions.push(s);
    }

    // Security checks
    if let Some(s) = security::check_security_updates().await {
        suggestions.push(s);
    }

    if let Some(s) = security::check_ssh_security().await {
        suggestions.push(s);
    }

    if let Some(s) = security::check_firewall().await {
        suggestions.push(s);
    }

    // Performance checks
    if let Some(s) = performance::check_boot_performance().await {
        suggestions.push(s);
    }

    if let Some(s) = performance::check_memory_usage().await {
        suggestions.push(s);
    }

    if let Some(s) = performance::check_cpu_usage().await {
        suggestions.push(s);
    }

    info!("Found {} proactive suggestions", suggestions.len());
    Ok(suggestions)
}
