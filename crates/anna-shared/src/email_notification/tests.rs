//! Tests for email notification system

#[cfg(test)]
mod tests {
    use crate::email_notification::{
        email_fun_fact, format_email_tracker, is_email_notification_query,
        EmailNotificationTracker, NotificationRecord, NotificationStatus, NotificationType,
    };

    fn make_notification(ntype: NotificationType, status: NotificationStatus) -> NotificationRecord {
        NotificationRecord {
            id: format!("NOT-{:?}", ntype),
            notification_type: ntype,
            status,
            subject: "Test notification".to_string(),
            ticket_id: None,
            created_at: 1234567890,
            sent_at: None,
            error: None,
            retry_count: 0,
        }
    }

    #[test]
    fn test_notification_status() {
        assert_eq!(NotificationStatus::Sent.name(), "Sent");
        assert_eq!(NotificationStatus::Failed.symbol(), "✗");
    }

    #[test]
    fn test_notification_type() {
        assert_eq!(NotificationType::TaskComplete.name(), "Task Complete");
        assert_eq!(NotificationType::ResearchResult.name(), "Research Result");
    }

    #[test]
    fn test_configure_email() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.configure("user@example.com".to_string(), true, 1234567890);

        assert!(tracker.is_configured());
        assert_eq!(tracker.config.as_ref().unwrap().address, "user@example.com");
    }

    #[test]
    fn test_not_configured_without_consent() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.configure("user@example.com".to_string(), false, 1234567890);

        assert!(!tracker.is_configured());
    }

    #[test]
    fn test_record_notification() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.sent_count(), 1);
        assert_eq!(tracker.sent_today, 1);
    }

    #[test]
    fn test_update_status() {
        let mut tracker = EmailNotificationTracker::new();
        let mut notification = make_notification(NotificationType::TaskComplete, NotificationStatus::Pending);
        notification.id = "NOT-001".to_string();
        tracker.record(notification);

        assert!(tracker.update_status("NOT-001", NotificationStatus::Sent, None));
        assert_eq!(tracker.sent_count(), 1);
    }

    #[test]
    fn test_daily_limit() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.configure("user@example.com".to_string(), true, 1234567890);
        tracker.config.as_mut().unwrap().daily_limit = Some(2);

        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));
        assert!(!tracker.daily_limit_reached());

        tracker.record(make_notification(NotificationType::TaskFailed, NotificationStatus::Sent));
        assert!(tracker.daily_limit_reached());
    }

    #[test]
    fn test_reset_daily() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.sent_today = 5;
        tracker.reset_daily("2025-12-13");

        assert_eq!(tracker.sent_today, 0);
        assert_eq!(tracker.last_reset_date, Some("2025-12-13".to_string()));
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Failed));

        assert!((tracker.success_rate() - 66.66).abs() < 1.0);
    }

    #[test]
    fn test_format_email_tracker() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.configure("user@example.com".to_string(), true, 1234567890);
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));

        let output = format_email_tracker(&tracker);
        assert!(output.contains("Email Notifications"));
        assert!(output.contains("user@example.com"));
    }

    #[test]
    fn test_is_email_notification_query() {
        assert!(is_email_notification_query("email me when done"));
        assert!(is_email_notification_query("configure email"));
        assert!(is_email_notification_query("notification history"));
        assert!(!is_email_notification_query("what is the weather?"));
    }

    #[test]
    fn test_email_fun_fact() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));

        let fact = email_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
