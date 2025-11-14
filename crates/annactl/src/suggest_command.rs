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
use std::time::Instant;

use crate::errors::*;
use crate::logging::LogEntry;

/// Execute 'suggest' command - show prioritized suggestions
pub async fn execute_suggest_command(
    json: bool,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    // TODO: Implement real suggestion prioritization
    // This should:
    // - Detect system issues and missing components
    // - Prioritize by: severity, prerequisites, user profile
    // - Show ONLY top 2-5 suggestions
    // - Filter out discarded suggestions
    // - Provide clear explanations and citations
    // - Suggest concrete next steps

    println!("Prioritized Suggestions");
    println!("=======================\n");
    println!("[TODO: Suggestion prioritization engine]\n");
    println!("This command will show the 2-5 most important");
    println!("suggestions for improving your system.\n");
    println!("Each suggestion will explain:");
    println!("  - What the issue is");
    println!("  - Why it matters");
    println!("  - How to fix it");
    println!("  - Reference links (Arch Wiki, etc.)\n");

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
