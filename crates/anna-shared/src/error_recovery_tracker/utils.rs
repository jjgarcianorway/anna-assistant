//! Error recovery utility functions
//!
//! Query detection and fun facts about error recovery.

use super::tracker::ErrorRecoveryTracker;

/// Check if query is about error recovery
pub fn is_error_recovery_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "error recovery",
        "recovery rate",
        "error handling",
        "how many errors",
        "failed recovery",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about error recovery
pub fn error_recovery_fun_fact(tracker: &ErrorRecoveryTracker) -> String {
    if tracker.records.is_empty() {
        return "No error recovery data yet!".to_string();
    }

    let facts = [
        format!("Anna has handled {} errors.", tracker.total_errors),
        format!("{}% of errors were successfully recovered.", tracker.recovery_rate()),
        format!("{} errors were recovered automatically.", tracker.total_recovered),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
