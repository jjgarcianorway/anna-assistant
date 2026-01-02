//! Application and formatting of notification configuration changes.

use crate::background_worker::notifications::{EmailConfig, NotificationConfig};
use super::types::NotifyConfigChange;

/// Apply a notification config change
pub fn apply_notify_change(config: &mut NotificationConfig, change: &NotifyConfigChange) {
    match change {
        NotifyConfigChange::SetEmail(email) => {
            config.email = Some(EmailConfig {
                to: email.clone(),
                ..EmailConfig::default()
            });
        }
        NotifyConfigChange::ClearEmail => {
            config.email = None;
        }
        NotifyConfigChange::DesktopNotify(enabled) => {
            config.desktop_enabled = *enabled;
        }
        NotifyConfigChange::WallNotify(enabled) => {
            config.wall_enabled = *enabled;
        }
        NotifyConfigChange::QuietHours(start, end) => {
            config.quiet_hours = Some(format!("{}-{}", start, end));
        }
        NotifyConfigChange::ClearQuietHours => {
            config.quiet_hours = None;
        }
        NotifyConfigChange::RateLimit(channel, secs) => {
            config.rate_limits.insert(channel.clone(), *secs);
        }
    }
}

/// Format notification settings for display
pub fn format_notification_settings(config: &NotificationConfig) -> String {
    let mut lines = vec![];

    // Email
    if let Some(ref email) = config.email {
        lines.push(format!("email             {}", email.to));
    } else {
        lines.push("email             not configured".to_string());
    }

    // Desktop
    lines.push(format!(
        "desktop           {}",
        if config.desktop_enabled { "enabled" } else { "disabled" }
    ));

    // Wall
    lines.push(format!(
        "wall              {}",
        if config.wall_enabled { "enabled" } else { "disabled" }
    ));

    // Quiet hours
    if let Some(ref hours) = config.quiet_hours {
        lines.push(format!("quiet_hours       {}", hours));
    } else {
        lines.push("quiet_hours       not set".to_string());
    }

    lines.push(String::new());
    lines.push("Configure via natural language:".to_string());
    lines.push("  \"set my email to user@example.com\"".to_string());
    lines.push("  \"enable desktop notifications\"".to_string());
    lines.push("  \"set quiet hours 22:00 to 08:00\"".to_string());

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_email_change() {
        let mut config = NotificationConfig::default();
        apply_notify_change(&mut config, &NotifyConfigChange::SetEmail("test@test.com".to_string()));
        assert!(config.email.is_some());
        assert_eq!(config.email.as_ref().unwrap().to, "test@test.com");
    }

    #[test]
    fn test_apply_desktop_change() {
        let mut config = NotificationConfig::default();
        assert!(!config.desktop_enabled);
        apply_notify_change(&mut config, &NotifyConfigChange::DesktopNotify(true));
        assert!(config.desktop_enabled);
    }

    #[test]
    fn test_format_settings() {
        let config = NotificationConfig::default();
        let output = format_notification_settings(&config);
        assert!(output.contains("email"));
        assert!(output.contains("desktop"));
        assert!(output.contains("wall"));
    }
}
