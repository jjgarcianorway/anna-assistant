//! Notification Configuration via Natural Language (v0.0.470).
//!
//! Allows users to configure notification settings through natural language:
//! - "set my email to user@example.com"
//! - "enable desktop notifications"
//! - "disable wall messages"
//! - "set quiet hours 22:00 to 08:00"
//!
//! Per VISION.md: "All settings changeable through annactl in natural language"

use crate::background_worker::notifications::{EmailConfig, NotificationConfig};

/// Notification configuration change detected from natural language
#[derive(Debug, Clone, PartialEq)]
pub enum NotifyConfigChange {
    /// Set email address
    SetEmail(String),
    /// Clear email
    ClearEmail,
    /// Enable/disable desktop notifications
    DesktopNotify(bool),
    /// Enable/disable wall messages
    WallNotify(bool),
    /// Set quiet hours (start, end in "HH:MM" format)
    QuietHours(String, String),
    /// Clear quiet hours
    ClearQuietHours,
    /// Set rate limit for a channel (channel, seconds)
    RateLimit(String, u64),
}

impl NotifyConfigChange {
    /// Human-readable description of the change
    pub fn description(&self) -> String {
        match self {
            NotifyConfigChange::SetEmail(email) => {
                format!("Set notification email to {}", email)
            }
            NotifyConfigChange::ClearEmail => "Removed notification email".to_string(),
            NotifyConfigChange::DesktopNotify(true) => {
                "Enabled desktop notifications".to_string()
            }
            NotifyConfigChange::DesktopNotify(false) => {
                "Disabled desktop notifications".to_string()
            }
            NotifyConfigChange::WallNotify(true) => {
                "Enabled wall broadcast messages".to_string()
            }
            NotifyConfigChange::WallNotify(false) => {
                "Disabled wall broadcast messages".to_string()
            }
            NotifyConfigChange::QuietHours(start, end) => {
                format!("Set quiet hours from {} to {}", start, end)
            }
            NotifyConfigChange::ClearQuietHours => "Cleared quiet hours".to_string(),
            NotifyConfigChange::RateLimit(channel, secs) => {
                format!("Set {} rate limit to {} seconds", channel, secs)
            }
        }
    }
}

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

/// Extract email from query
fn extract_email(lower: &str, original: &str) -> Option<String> {
    // Look for "email to X" or "email is X" or "my email X"
    let patterns = ["email to ", "email is ", "my email ", "email: "];

    for pattern in patterns {
        if let Some(pos) = lower.find(pattern) {
            let start = pos + pattern.len();
            let rest = &original[start..];
            // Extract email-like string
            if let Some(email) = extract_email_address(rest) {
                return Some(email);
            }
        }
    }

    // Also check for standalone email in query
    for word in original.split_whitespace() {
        if word.contains('@') && word.contains('.') {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '.' && c != '_' && c != '-');
            if is_valid_email(cleaned) {
                return Some(cleaned.to_string());
            }
        }
    }

    None
}

/// Extract email address from text
fn extract_email_address(text: &str) -> Option<String> {
    let email: String = text
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '@' || *c == '.' || *c == '_' || *c == '-' || *c == '+')
        .collect();

    if is_valid_email(&email) {
        Some(email)
    } else {
        None
    }
}

/// Basic email validation
fn is_valid_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];

    !local.is_empty() && !domain.is_empty() && domain.contains('.')
}

/// Extract quiet hours from query
fn extract_quiet_hours(lower: &str) -> Option<(String, String)> {
    // Look for patterns like "quiet hours 22:00 to 08:00"
    if !lower.contains("quiet") {
        return None;
    }

    // Extract times using regex-like patterns
    let times: Vec<String> = extract_times(lower);
    if times.len() >= 2 {
        return Some((times[0].clone(), times[1].clone()));
    }

    None
}

/// Extract time patterns (HH:MM) from text
fn extract_times(text: &str) -> Vec<String> {
    let mut times = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Look for digit patterns
        if chars[i].is_ascii_digit() {
            let start = i;
            // Collect digits
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            // Check for colon
            if i < chars.len() && chars[i] == ':' {
                i += 1;
                // Collect more digits
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let time_str: String = chars[start..i].iter().collect();
                if is_valid_time(&time_str) {
                    times.push(time_str);
                }
            }
        } else {
            i += 1;
        }
    }

    times
}

/// Check if string is valid time format
fn is_valid_time(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    let hour: u8 = parts[0].parse().unwrap_or(99);
    let minute: u8 = parts[1].parse().unwrap_or(99);
    hour < 24 && minute < 60
}

fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
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

    #[test]
    fn test_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user@domain.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("no@domain"));
    }

    #[test]
    fn test_valid_time() {
        assert!(is_valid_time("22:00"));
        assert!(is_valid_time("08:30"));
        assert!(is_valid_time("00:00"));
        assert!(!is_valid_time("25:00"));
        assert!(!is_valid_time("12:60"));
    }
}
