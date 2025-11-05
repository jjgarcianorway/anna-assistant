//! Notification system - Sends alerts to users via appropriate channel
//!
//! Supports:
//! - GUI notifications (notify-send for desktop environments)
//! - Terminal broadcasts (wall for TTY/SSH users) - only for critical issues
//! - Detects environment automatically
//! - Rate limiting to prevent spam (1 hour cooldown)

use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn};

// Global last notification time with mutex for thread safety
static LAST_NOTIFICATION: Mutex<Option<Instant>> = Mutex::new(None);

// Cooldown period: 1 hour between notifications
const NOTIFICATION_COOLDOWN: Duration = Duration::from_secs(3600);

/// Notification urgency level
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

/// Check if enough time has passed since last notification (cooldown check)
fn should_send_notification() -> bool {
    let mut last = LAST_NOTIFICATION.lock().unwrap();

    match *last {
        None => {
            // First notification ever - allow it
            *last = Some(Instant::now());
            true
        }
        Some(last_time) => {
            let elapsed = last_time.elapsed();
            if elapsed >= NOTIFICATION_COOLDOWN {
                // Cooldown period has passed - allow notification
                *last = Some(Instant::now());
                info!("Cooldown passed ({:.0} minutes), sending notification", elapsed.as_secs() / 60);
                true
            } else {
                // Still in cooldown period - skip notification
                let remaining = NOTIFICATION_COOLDOWN - elapsed;
                info!("Notification blocked by cooldown ({} minutes remaining)", remaining.as_secs() / 60);
                false
            }
        }
    }
}

/// Send a notification to all appropriate channels
pub async fn send_notification(title: &str, message: &str, urgency: NotificationUrgency) {
    // Check cooldown before sending
    if !should_send_notification() {
        info!("Skipping notification due to cooldown");
        return;
    }

    // Try GUI notification first (for desktop users)
    send_gui_notification(title, message, urgency).await;

    // For critical notifications, also broadcast to terminals (but NEVER use wall)
    // Wall is too intrusive and spams all terminals
    // We rely on GUI notifications only
    if matches!(urgency, NotificationUrgency::Critical) {
        info!("Critical notification sent (GUI only, no wall spam)");
    }
}

/// Send GUI notification using notify-send
async fn send_gui_notification(title: &str, message: &str, urgency: NotificationUrgency) {
    // Check if notify-send is available
    let has_notify_send = Command::new("which")
        .arg("notify-send")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_notify_send {
        return;
    }

    let urgency_str = match urgency {
        NotificationUrgency::Low => "low",
        NotificationUrgency::Normal => "normal",
        NotificationUrgency::Critical => "critical",
    };

    // Send notification to all active desktop sessions
    // We need to find all user sessions and send to each
    if let Ok(sessions) = get_active_sessions().await {
        for session in sessions {
            // Make notifications more visible with:
            // - Longer timeout (10 seconds instead of default 5)
            // - Category for proper desktop integration
            // - Better icon based on urgency
            let icon = match urgency {
                NotificationUrgency::Critical => "dialog-error",
                NotificationUrgency::Normal => "dialog-information",
                NotificationUrgency::Low => "dialog-information",
            };

            let result = Command::new("sudo")
                .args(&[
                    "-u", &session.username,
                    "DISPLAY=:0",
                    "DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/{}/bus",
                    "notify-send",
                    "--urgency", urgency_str,
                    "--icon", icon,
                    "--app-name", "Anna Assistant",
                    "--category", "system",
                    "--expire-time", "10000",  // 10 seconds - more visible
                    title,
                    message,
                ])
                .env("DISPLAY", ":0")
                .output();

            match result {
                Ok(output) if output.status.success() => {
                    info!("GUI notification sent to user {}", session.username);
                }
                Ok(_) => {
                    warn!("Failed to send GUI notification to {}", session.username);
                }
                Err(e) => {
                    warn!("Error sending GUI notification: {}", e);
                }
            }
        }
    }
}

// Removed send_terminal_broadcast() - wall is too intrusive and spams terminals
// GUI notifications with cooldown are sufficient for notifying users

/// User session info
#[derive(Debug)]
struct UserSession {
    username: String,
    #[allow(dead_code)]
    uid: u32,
}

/// Get active user sessions from loginctl
async fn get_active_sessions() -> Result<Vec<UserSession>, std::io::Error> {
    let output = Command::new("loginctl")
        .args(&["list-sessions", "--no-legend"])
        .output()?;

    let mut sessions = Vec::new();

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let username = parts[2].to_string();
                
                // Get UID for this user
                if let Ok(uid_output) = Command::new("id")
                    .args(&["-u", &username])
                    .output()
                {
                    if let Ok(uid_str) = String::from_utf8(uid_output.stdout) {
                        if let Ok(uid) = uid_str.trim().parse::<u32>() {
                            sessions.push(UserSession { username, uid });
                        }
                    }
                }
            }
        }
    }

    Ok(sessions)
}

/// Check if there are any critical SECURITY issues and notify
/// Only notifies for Priority::Mandatory in "Security & Privacy" category
/// This prevents spam while ensuring critical security issues are seen
pub async fn check_and_notify_critical(advice: &[anna_common::Advice]) {
    use anna_common::Priority;

    // ONLY notify for MANDATORY security issues
    let critical: Vec<_> = advice.iter()
        .filter(|a| {
            matches!(a.priority, Priority::Mandatory) &&
            a.category == "Security & Privacy"
        })
        .collect();

    if critical.is_empty() {
        return;
    }

    let count = critical.len();
    let title = if count == 1 {
        "🔒 Critical Security Issue Detected"
    } else {
        "🔒 Critical Security Issues Detected"
    };

    let message = if count == 1 {
        format!("Anna detected 1 CRITICAL SECURITY issue:\n\n• {}\n\nThis requires immediate attention!", critical[0].title)
    } else {
        let issues: Vec<String> = critical.iter()
            .take(3)
            .map(|a| format!("• {}", a.title))
            .collect();
        let mut msg = format!("Anna detected {} CRITICAL SECURITY issues:\n\n", count);
        msg.push_str(&issues.join("\n"));
        if count > 3 {
            msg.push_str(&format!("\n... and {} more", count - 3));
        }
        msg.push_str("\n\nThese require immediate attention!");
        msg
    };

    info!("Sending critical SECURITY notification for {} issues (wall + GUI)", count);
    send_notification(title, &message, NotificationUrgency::Critical).await;
}
