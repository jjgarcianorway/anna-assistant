//! Utility functions for ticket logs.

use super::types::TicketLog;
use std::fs;
use std::path::PathBuf;

/// Get ticket log directory
pub fn ticket_log_dir() -> PathBuf {
    // Try /var/lib/anna/tickets first, fall back to ~/.anna/tickets
    let var_lib = PathBuf::from("/var/lib/anna/tickets");
    if var_lib.exists() || fs::create_dir_all(&var_lib).is_ok() {
        return var_lib;
    }

    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".anna")
        .join("tickets")
}

/// Generate ISO 8601 timestamp
pub fn chrono_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Load all ticket logs (for analysis)
pub fn load_all_tickets() -> Vec<TicketLog> {
    let dir = ticket_log_dir();
    let mut tickets = vec![];

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(ticket) = serde_json::from_str::<TicketLog>(&json) {
                        tickets.push(ticket);
                    }
                }
            }
        }
    }

    // Sort by timestamp (newest first)
    tickets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    tickets
}

/// Load recent tickets (last N)
pub fn load_recent_tickets(limit: usize) -> Vec<TicketLog> {
    let mut tickets = load_all_tickets();
    tickets.truncate(limit);
    tickets
}
