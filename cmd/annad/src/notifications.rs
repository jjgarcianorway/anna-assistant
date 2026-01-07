use anyhow::Result;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::warn;

/// Send a notification to the user
pub fn notify(uid: u32, message: &str, user_data_dir: &Path) {
    // Try desktop notification first
    if try_desktop_notification(uid, message) {
        return;
    }

    // Fallback to log file
    let _ = log_notification(user_data_dir, message);
}

/// Try to send a desktop notification via notify-send
fn try_desktop_notification(uid: u32, message: &str) -> bool {
    // Check if notify-send exists
    if which::which("notify-send").is_err() {
        return false;
    }

    // Try to send notification as the user
    let status = Command::new("sudo")
        .args([
            "-u",
            &format!("#{}", uid),
            "notify-send",
            "--app-name=Anna",
            "--icon=system-software-update",
            "Anna System Assistant",
            message,
        ])
        .status();

    match status {
        Ok(s) if s.success() => true,
        Ok(_) | Err(_) => {
            warn!(target: "notifications", "notify-send failed, falling back to log");
            false
        }
    }
}

/// Log notification to file
fn log_notification(user_data_dir: &Path, message: &str) -> Result<()> {
    let log_file = user_data_dir.join("notifications.log");

    // Ensure directory exists
    if let Some(parent) = log_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    let timestamp = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());

    writeln!(file, "[{}] {}", timestamp, message)?;

    Ok(())
}

/// Broadcast a system-wide message
#[allow(dead_code)]
pub fn broadcast_system_message(message: &str) {
    // Try wall for system-wide broadcasts
    if which::which("wall").is_ok() {
        let _ = Command::new("wall").arg(message).status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_log_notification() {
        let temp_dir = TempDir::new().unwrap();
        let message = "Test notification";

        log_notification(temp_dir.path(), message).unwrap();

        let log_file = temp_dir.path().join("notifications.log");
        assert!(log_file.exists());

        let contents = fs::read_to_string(&log_file).unwrap();
        assert!(contents.contains("Test notification"));
        assert!(contents.contains("["));
        assert!(contents.contains("]"));
    }

    #[test]
    fn test_log_notification_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir");

        log_notification(&nested_path, "Test").unwrap();

        let log_file = nested_path.join("notifications.log");
        assert!(log_file.exists());
    }
}
