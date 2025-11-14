//! Discard command - let user ignore suggestions they don't care about
//!
//! Real Anna: Via REPL - ask "ignore that suggestion"
//! Purpose: Mark a suggestion as discarded so it won't be shown again
//! Behavior:
//! - Marks suggestion as discarded in context database
//! - Won't show it again unless there's a strong reason
//! - Can be un-discarded later if needed

use anyhow::Result;
use std::time::Instant;

use crate::errors::*;
use crate::logging::LogEntry;

/// Execute 'discard' command - ignore a suggestion
pub async fn execute_discard_command(
    key: String,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    // TODO: Implement suggestion discarding
    // This should:
    // - Validate that suggestion key exists
    // - Mark it as discarded in context DB
    // - Show confirmation message
    // - Explain that it can be un-discarded later

    println!("Discard Suggestion");
    println!("==================\n");
    println!("[TODO: Discard implementation]\n");
    println!("Would discard suggestion: {}\n", key);
    println!("This suggestion will no longer appear when you ask 'what should I improve?'");
    println!("unless a critical security issue makes it relevant again.\n");

    // Log command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "discard".to_string(),
        allowed: Some(true),
        args: vec![key],
        exit_code: EXIT_SUCCESS,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: true,
        error: None,
    };
    let _ = log_entry.write();

    Ok(())
}
