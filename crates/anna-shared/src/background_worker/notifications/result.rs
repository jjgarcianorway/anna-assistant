//! Notification results and status types.

use serde::{Deserialize, Serialize};

/// Result of notification attempt
#[derive(Debug, Clone, Default)]
pub struct NotifyResult {
    pub email_sent: bool,
    pub email_error: Option<String>,
    pub email_rate_limited: bool,
    pub desktop_sent: bool,
    pub desktop_error: Option<String>,
    pub desktop_rate_limited: bool,
    pub wall_sent: bool,
    pub wall_error: Option<String>,
    pub wall_rate_limited: bool,
    pub skipped_quiet_hours: bool,
}

impl NotifyResult {
    /// Check if any notification was sent
    pub fn any_sent(&self) -> bool {
        self.email_sent || self.desktop_sent || self.wall_sent
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        let mut parts = vec![];
        if self.email_sent {
            parts.push("email");
        }
        if self.desktop_sent {
            parts.push("desktop");
        }
        if self.wall_sent {
            parts.push("wall");
        }
        if parts.is_empty() {
            if self.skipped_quiet_hours {
                "skipped (quiet hours)".to_string()
            } else {
                "no channels enabled".to_string()
            }
        } else {
            format!("sent via {}", parts.join(", "))
        }
    }
}

/// Notification status for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStatus {
    pub email_configured: bool,
    pub desktop_enabled: bool,
    pub wall_enabled: bool,
    pub quiet_hours: Option<String>,
    pub last_email: Option<u64>,
    pub last_desktop: Option<u64>,
    pub last_wall: Option<u64>,
}

impl std::fmt::Display for NotificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[notifications]")?;
        writeln!(
            f,
            "  email              {}",
            if self.email_configured {
                "configured"
            } else {
                "not configured"
            }
        )?;
        writeln!(
            f,
            "  desktop            {}",
            if self.desktop_enabled {
                "enabled"
            } else {
                "disabled"
            }
        )?;
        writeln!(
            f,
            "  wall               {}",
            if self.wall_enabled {
                "enabled"
            } else {
                "disabled"
            }
        )?;
        if let Some(ref hours) = self.quiet_hours {
            writeln!(f, "  quiet_hours        {}", hours)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_result_summary() {
        let mut result = NotifyResult::default();
        assert_eq!(result.summary(), "no channels enabled");

        result.email_sent = true;
        assert_eq!(result.summary(), "sent via email");

        result.desktop_sent = true;
        assert_eq!(result.summary(), "sent via email, desktop");
    }
}
