//! Query detection for quick status.

/// Detect if query is asking for quick status
pub fn is_quick_status_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "quick status",
        "quick check",
        "health check",
        "system ok",
        "everything ok",
        "any problems",
        "any issues",
        "status check",
        "how's the system",
        "how is the system",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}
