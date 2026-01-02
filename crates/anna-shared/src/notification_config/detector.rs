//! Natural language detection for notification configuration changes.

use super::parsers::{extract_email, extract_quiet_hours, matches_any};
use super::types::NotifyConfigChange;

/// Detect notification configuration changes from natural language
pub fn detect_notify_config(query: &str) -> Option<NotifyConfigChange> {
    let lower = query.to_lowercase();

    // Email configuration
    if let Some(email) = extract_email(&lower, query) {
        return Some(NotifyConfigChange::SetEmail(email));
    }

    if matches_any(&lower, &["remove email", "remove my email", "clear email", "no email", "disable email"]) {
        return Some(NotifyConfigChange::ClearEmail);
    }

    // Desktop notifications
    if matches_any(&lower, &[
        "enable desktop", "turn on desktop", "desktop notifications on",
        "enable notify", "turn on notifications"
    ]) {
        return Some(NotifyConfigChange::DesktopNotify(true));
    }

    if matches_any(&lower, &[
        "disable desktop", "turn off desktop", "desktop notifications off",
        "no desktop notifications", "disable notifications"
    ]) {
        return Some(NotifyConfigChange::DesktopNotify(false));
    }

    // Wall messages
    if matches_any(&lower, &[
        "enable wall", "turn on wall", "wall messages on",
        "broadcast on", "enable broadcast"
    ]) {
        return Some(NotifyConfigChange::WallNotify(true));
    }

    if matches_any(&lower, &[
        "disable wall", "turn off wall", "wall messages off",
        "no wall", "disable broadcast", "no broadcast"
    ]) {
        return Some(NotifyConfigChange::WallNotify(false));
    }

    // Quiet hours
    if let Some((start, end)) = extract_quiet_hours(&lower) {
        return Some(NotifyConfigChange::QuietHours(start, end));
    }

    if matches_any(&lower, &[
        "clear quiet hours", "remove quiet hours", "no quiet hours",
        "disable quiet hours", "always notify"
    ]) {
        return Some(NotifyConfigChange::ClearQuietHours);
    }

    None
}

/// Check if query is asking to show notification settings
pub fn is_show_notifications(query: &str) -> bool {
    let lower = query.to_lowercase();
    matches_any(&lower, &[
        "show notifications", "notification settings", "my notifications",
        "notification config", "show notify", "how am i notified"
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_email() {
        let change = detect_notify_config("set my email to user@example.com");
        assert_eq!(change, Some(NotifyConfigChange::SetEmail("user@example.com".to_string())));
    }

    #[test]
    fn test_detect_email_inline() {
        let change = detect_notify_config("notify me at test@domain.org please");
        assert_eq!(change, Some(NotifyConfigChange::SetEmail("test@domain.org".to_string())));
    }

    #[test]
    fn test_detect_clear_email() {
        let change = detect_notify_config("remove my email");
        assert_eq!(change, Some(NotifyConfigChange::ClearEmail));
    }

    #[test]
    fn test_detect_desktop_enable() {
        let change = detect_notify_config("enable desktop notifications");
        assert_eq!(change, Some(NotifyConfigChange::DesktopNotify(true)));
    }

    #[test]
    fn test_detect_desktop_disable() {
        let change = detect_notify_config("turn off desktop notifications");
        assert_eq!(change, Some(NotifyConfigChange::DesktopNotify(false)));
    }

    #[test]
    fn test_detect_wall_enable() {
        let change = detect_notify_config("enable wall messages");
        assert_eq!(change, Some(NotifyConfigChange::WallNotify(true)));
    }

    #[test]
    fn test_detect_wall_disable() {
        let change = detect_notify_config("disable broadcast");
        assert_eq!(change, Some(NotifyConfigChange::WallNotify(false)));
    }

    #[test]
    fn test_detect_quiet_hours() {
        let change = detect_notify_config("set quiet hours 22:00 to 08:00");
        assert_eq!(change, Some(NotifyConfigChange::QuietHours("22:00".to_string(), "08:00".to_string())));
    }

    #[test]
    fn test_detect_clear_quiet_hours() {
        let change = detect_notify_config("clear quiet hours");
        assert_eq!(change, Some(NotifyConfigChange::ClearQuietHours));
    }

    #[test]
    fn test_is_show_notifications() {
        assert!(is_show_notifications("show notification settings"));
        assert!(is_show_notifications("how am i notified"));
        assert!(!is_show_notifications("what time is it"));
    }
}
