//! Formatting functions for email notification display

use super::tracker::EmailNotificationTracker;

/// Format email tracker for display
pub fn format_email_tracker(tracker: &EmailNotificationTracker) -> String {
    let mut lines = vec!["=== Email Notifications ===".to_string()];
    lines.push(String::new());

    // Config status
    if let Some(config) = &tracker.config {
        if config.consent_given {
            lines.push(format!("Email: {} (consent given)", config.address));
        } else {
            lines.push("Email: Not configured (no consent)".to_string());
        }
    } else {
        lines.push("Email: Not configured".to_string());
    }

    if tracker.notifications.is_empty() {
        lines.push("No notifications yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(String::new());
    lines.push(format!("Total notifications: {}", tracker.total_count()));
    lines.push(format!("Sent: {}", tracker.sent_count()));
    lines.push(format!("Success rate: {:.1}%", tracker.success_rate()));
    lines.push(format!("Sent today: {}", tracker.sent_today));

    // Daily limit
    if tracker.daily_limit_reached() {
        lines.push("Daily limit reached!".to_string());
    }

    // Recent
    let recent = tracker.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent:".to_string());
        for n in recent {
            lines.push(format!(
                "  [{}] {} - {}",
                n.status.symbol(),
                n.notification_type.name(),
                n.subject
            ));
        }
    }

    lines.join("\n")
}

/// Format email tracker compact
pub fn format_email_tracker_compact(tracker: &EmailNotificationTracker) -> String {
    let configured = if tracker.is_configured() { "configured" } else { "not configured" };
    format!(
        "Email: {} | {} sent | {:.0}% success",
        configured,
        tracker.sent_count(),
        tracker.success_rate()
    )
}

/// Format email tracker one-line
pub fn format_email_tracker_oneline(tracker: &EmailNotificationTracker) -> String {
    format!(
        "{} notifications ({} sent)",
        tracker.total_count(),
        tracker.sent_count()
    )
}
