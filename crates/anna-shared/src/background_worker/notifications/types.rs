//! Notification types and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Notification configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Email configuration
    pub email: Option<EmailConfig>,
    /// Desktop notification enabled
    pub desktop_enabled: bool,
    /// Wall message enabled
    pub wall_enabled: bool,
    /// Global quiet hours (e.g., "22:00-08:00")
    pub quiet_hours: Option<String>,
    /// Per-channel rate limits (channel -> min seconds between alerts)
    #[serde(default)]
    pub rate_limits: HashMap<String, u64>,
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// Recipient email address
    pub to: String,
    /// From address (optional)
    pub from: Option<String>,
    /// SMTP command (e.g., "sendmail", "msmtp")
    pub command: String,
    /// Additional arguments
    #[serde(default)]
    pub args: Vec<String>,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            to: String::new(),
            from: None,
            command: "sendmail".to_string(),
            args: vec!["-t".to_string()],
        }
    }
}

/// Notification channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Desktop,
    Wall,
}

impl NotificationChannel {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Desktop => "desktop",
            Self::Wall => "wall",
        }
    }
}

/// Alert priority for notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert!(config.email.is_none());
        assert!(!config.desktop_enabled);
        assert!(!config.wall_enabled);
    }
}
