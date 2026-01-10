//! System alert handling for proactive notifications.

use anna_shared::monitor::{IssueStore, Severity};

/// Get pending critical/warning alerts that haven't been shown to user
/// Returns formatted alert messages and marks them as notified
pub fn get_pending_alerts() -> Option<Vec<String>> {
    let mut store = IssueStore::load().ok()?;
    let unnotified = store.get_unnotified();

    if unnotified.is_empty() {
        return None;
    }

    // Only show critical and warning alerts (not info)
    let alerts: Vec<String> = unnotified
        .iter()
        .filter(|issue| matches!(issue.severity, Severity::Critical | Severity::Warning))
        .map(|issue| {
            let icon = match issue.severity {
                Severity::Critical => "🔴 CRITICAL",
                Severity::Warning => "🟡 Warning",
                Severity::Info => "ℹ️ Info",
            };
            let mut msg = format!("{}: {}", icon, issue.summary);
            if let Some(ref fix) = issue.suggested_fix {
                msg.push_str(&format!("\n   Suggested fix: {}", fix));
            }
            msg
        })
        .collect();

    if alerts.is_empty() {
        return None;
    }

    // Mark as notified
    store.mark_notified();
    let _ = store.save();

    Some(alerts)
}
