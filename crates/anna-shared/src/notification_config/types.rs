//! Notification configuration change types.

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
