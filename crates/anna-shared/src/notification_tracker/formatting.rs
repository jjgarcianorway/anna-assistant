// v0.0.533: Notification Formatting (Phase 109)
// Formatting and utility functions for notifications

use super::record::NotificationRecord;
use super::tracker::NotificationTracker;

/// Format notification for display
pub fn format_notification(notif: &NotificationRecord) -> String {
    format!(
        "{} [{}]\n  Channel: {} | Priority: {}\n  Subject: {}\n  Status: {}{}",
        notif.id,
        notif.created_at,
        notif.channel,
        notif.priority,
        notif.subject,
        notif.status,
        if let Some(err) = &notif.error {
            format!("\n  Error: {}", err)
        } else {
            String::new()
        }
    )
}

/// Format notification compact
pub fn format_notification_compact(notif: &NotificationRecord) -> String {
    format!(
        "{}: {} [{}] - {}",
        notif.id, notif.subject, notif.channel, notif.status
    )
}

/// Format notification oneline
pub fn format_notification_oneline(notif: &NotificationRecord) -> String {
    format!("{} [{}]", notif.id, notif.status)
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &NotificationTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Notification Tracker ===\n\n");

    output.push_str(&format!("Total: {}\n", tracker.total()));
    output.push_str(&format!("Pending: {}\n", tracker.pending().len()));
    output.push_str(&format!(
        "Email: {}\n\n",
        if tracker.has_email() { "Configured" } else { "Not set" }
    ));

    output.push_str("--- By Channel ---\n");
    for (channel, count) in tracker.channel_stats() {
        output.push_str(&format!("  {}: {}\n", channel, count));
    }

    output.push_str("\n--- Delivery Status ---\n");
    for (status, count) in tracker.delivery_stats() {
        output.push_str(&format!("  {}: {}\n", status, count));
    }

    output
}

/// Check if query is notification-related
pub fn is_notification_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("notif")
        || lower.contains("email")
        || lower.contains("alert")
        || lower.contains("message")
        || lower.contains("remind")
}

/// Fun fact about notifications
pub fn notification_fun_fact() -> &'static str {
    "Anna respects your focus - she uses smart anti-spam to avoid flooding you with notifications. Quality over quantity!"
}
