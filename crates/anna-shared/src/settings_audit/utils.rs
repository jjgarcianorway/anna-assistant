// v0.0.573: Settings Audit - Utility Functions
// Formatting and query helpers for audit logs

use super::log::AuditLog;

/// Format audit log for display
pub fn format_audit_log(log: &AuditLog, limit: usize) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Audit Log ===\n\n");
    output.push_str(&format!("Total entries: {}\n", log.count()));
    output.push_str(&format!("Security events: {}\n\n", log.security_events().len()));

    let recent = log.recent(limit);
    if recent.is_empty() {
        output.push_str("No entries.\n");
        return output;
    }

    output.push_str("Recent Activity:\n");
    for entry in recent {
        let time = entry.timestamp.format("%H:%M:%S");
        output.push_str(&format!(
            "[{}] {} [{}] {}\n",
            time, entry.event_type, entry.severity, entry.description
        ));
    }

    output
}

/// Check if query is about audit
pub fn is_audit_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("audit")
        || lower.contains("settings log")
        || lower.contains("change history")
        || lower.contains("who changed")
}

/// Fun fact about audit
pub fn audit_fun_fact() -> &'static str {
    "The audit log tracks all settings changes for compliance and security!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_audit_log() {
        let log = AuditLog::new();
        let output = format_audit_log(&log, 10);
        assert!(output.contains("Audit"));
    }

    #[test]
    fn test_is_audit_query() {
        assert!(is_audit_query("show audit log"));
        assert!(is_audit_query("change history"));
        assert!(!is_audit_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = audit_fun_fact();
        assert!(fact.contains("audit"));
    }
}
