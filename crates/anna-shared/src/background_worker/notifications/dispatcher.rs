//! Notification dispatcher implementation.

use super::result::{NotificationStatus, NotifyResult};
use super::types::{AlertPriority, EmailConfig, NotificationChannel, NotificationConfig};
use super::utils::{chrono_lite, now_timestamp, parse_time};
use std::collections::HashMap;
use std::process::Command;

/// Notification dispatcher
pub struct NotificationDispatcher {
    config: NotificationConfig,
    /// Last alert time per channel (for rate limiting)
    last_alert: HashMap<NotificationChannel, u64>,
}

impl NotificationDispatcher {
    /// Create new dispatcher
    pub fn new(config: NotificationConfig) -> Self {
        Self {
            config,
            last_alert: HashMap::new(),
        }
    }

    /// Send notification to all enabled channels
    pub fn send(&mut self, subject: &str, body: &str, priority: AlertPriority) -> NotifyResult {
        let mut results = NotifyResult::default();
        let now = now_timestamp();

        // Check quiet hours
        if self.is_quiet_hours() && priority != AlertPriority::Critical {
            results.skipped_quiet_hours = true;
            return results;
        }

        // Email
        if let Some(ref email_config) = self.config.email {
            if self.can_send(NotificationChannel::Email, now) {
                match self.send_email(email_config, subject, body) {
                    Ok(_) => {
                        results.email_sent = true;
                        self.last_alert.insert(NotificationChannel::Email, now);
                    }
                    Err(e) => results.email_error = Some(e),
                }
            } else {
                results.email_rate_limited = true;
            }
        }

        // Desktop
        if self.config.desktop_enabled {
            if self.can_send(NotificationChannel::Desktop, now) {
                match self.send_desktop(subject, body) {
                    Ok(_) => {
                        results.desktop_sent = true;
                        self.last_alert.insert(NotificationChannel::Desktop, now);
                    }
                    Err(e) => results.desktop_error = Some(e),
                }
            } else {
                results.desktop_rate_limited = true;
            }
        }

        // Wall (only for high/critical priority)
        if self.config.wall_enabled && priority >= AlertPriority::High {
            if self.can_send(NotificationChannel::Wall, now) {
                match self.send_wall(subject, body) {
                    Ok(_) => {
                        results.wall_sent = true;
                        self.last_alert.insert(NotificationChannel::Wall, now);
                    }
                    Err(e) => results.wall_error = Some(e),
                }
            } else {
                results.wall_rate_limited = true;
            }
        }

        results
    }

    /// Check if we can send to a channel (rate limiting)
    fn can_send(&self, channel: NotificationChannel, now: u64) -> bool {
        let rate_limit = self
            .config
            .rate_limits
            .get(channel.display())
            .copied()
            .unwrap_or(300); // Default 5 minute rate limit

        if let Some(last) = self.last_alert.get(&channel) {
            now.saturating_sub(*last) >= rate_limit
        } else {
            true
        }
    }

    /// Check if currently in quiet hours
    fn is_quiet_hours(&self) -> bool {
        let Some(ref quiet) = self.config.quiet_hours else {
            return false;
        };

        // Parse "HH:MM-HH:MM" format
        let parts: Vec<&str> = quiet.split('-').collect();
        if parts.len() != 2 {
            return false;
        }

        let now = chrono_lite::current_hour_minute();
        let (now_h, now_m) = now;

        let start = parse_time(parts[0]);
        let end = parse_time(parts[1]);

        if let (Some((sh, sm)), Some((eh, em))) = (start, end) {
            let now_mins = now_h * 60 + now_m;
            let start_mins = sh * 60 + sm;
            let end_mins = eh * 60 + em;

            if start_mins <= end_mins {
                // Same day range (e.g., 09:00-17:00)
                now_mins >= start_mins && now_mins < end_mins
            } else {
                // Overnight range (e.g., 22:00-08:00)
                now_mins >= start_mins || now_mins < end_mins
            }
        } else {
            false
        }
    }

    /// Send email notification
    fn send_email(&self, config: &EmailConfig, subject: &str, body: &str) -> Result<(), String> {
        let from = config.from.as_deref().unwrap_or("anna@localhost");
        let message = format!(
            "From: {}\nTo: {}\nSubject: [Anna] {}\n\n{}",
            from, config.to, subject, body
        );

        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);

        let child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", config.command, e))?;

        use std::io::Write;
        if let Some(mut stdin) = child.stdin {
            stdin
                .write_all(message.as_bytes())
                .map_err(|e| format!("Failed to write email: {}", e))?;
        }

        Ok(())
    }

    /// Send desktop notification
    fn send_desktop(&self, subject: &str, body: &str) -> Result<(), String> {
        // Try notify-send (Linux)
        let result = Command::new("notify-send")
            .arg("-a")
            .arg("Anna")
            .arg(subject)
            .arg(body)
            .output();

        match result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(String::from_utf8_lossy(&output.stderr).to_string()),
            Err(e) => Err(format!("notify-send not available: {}", e)),
        }
    }

    /// Send wall message
    fn send_wall(&self, subject: &str, body: &str) -> Result<(), String> {
        let message = format!("[Anna Alert] {}\n{}", subject, body);

        let result = Command::new("wall").arg(&message).output();

        match result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(String::from_utf8_lossy(&output.stderr).to_string()),
            Err(e) => Err(format!("wall not available: {}", e)),
        }
    }

    /// Check if we can send to a channel (for testing/status)
    pub fn can_send_to(&self, channel: NotificationChannel) -> bool {
        let now = now_timestamp();
        self.can_send(channel, now)
    }

    /// Get status summary
    pub fn status(&self) -> NotificationStatus {
        NotificationStatus {
            email_configured: self.config.email.is_some(),
            desktop_enabled: self.config.desktop_enabled,
            wall_enabled: self.config.wall_enabled,
            quiet_hours: self.config.quiet_hours.clone(),
            last_email: self.last_alert.get(&NotificationChannel::Email).copied(),
            last_desktop: self.last_alert.get(&NotificationChannel::Desktop).copied(),
            last_wall: self.last_alert.get(&NotificationChannel::Wall).copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiting() {
        let config = NotificationConfig {
            desktop_enabled: true,
            rate_limits: [("desktop".to_string(), 60)].into_iter().collect(),
            ..Default::default()
        };

        let dispatcher = NotificationDispatcher::new(config);
        // Should be able to send first time
        assert!(dispatcher.can_send(NotificationChannel::Desktop, 1000));
    }
}
