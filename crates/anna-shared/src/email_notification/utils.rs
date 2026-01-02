//! Utility functions for email notifications

use super::tracker::EmailNotificationTracker;

/// Check if query is about email notifications
pub fn is_email_notification_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "email notification",
        "email me",
        "notify me",
        "send email",
        "notification history",
        "email config",
        "configure email",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about email notifications
pub fn email_fun_fact(tracker: &EmailNotificationTracker) -> String {
    if tracker.notifications.is_empty() {
        return "No email notifications sent yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has sent {} email notifications.",
            tracker.sent_count()
        ),
        format!(
            "Email success rate is {:.1}%.",
            tracker.success_rate()
        ),
        format!(
            "{} notifications sent today.",
            tracker.sent_today
        ),
        format!(
            "{} notifications are pending.",
            tracker.pending().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
