// Anna v0.11.0 - Storage Listener
//
// Watches /proc/self/mountinfo for filesystem mount/unmount changes.
// Triggers storage domain checks when mount state changes.

use crate::events::{create_event, EventDomain, SystemEvent};
use anyhow::Result;
use std::fs;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

const MOUNTINFO_PATH: &str = "/proc/self/mountinfo";
const POLL_INTERVAL_SECS: u64 = 5;

/// Spawn storage listener task
pub fn spawn_listener(tx: mpsc::UnboundedSender<SystemEvent>) -> JoinHandle<()> {
    info!("Starting storage listener (poll interval: {}s)", POLL_INTERVAL_SECS);

    tokio::spawn(async move {
        if let Err(e) = watch_mountinfo(tx).await {
            warn!("Storage listener error: {}", e);
        }
    })
}

/// Watch /proc/self/mountinfo for changes
async fn watch_mountinfo(tx: mpsc::UnboundedSender<SystemEvent>) -> Result<()> {
    let mut last_content = read_mountinfo()?;

    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

        match read_mountinfo() {
            Ok(current) => {
                if current != last_content {
                    let changes = detect_changes(&last_content, &current);

                    debug!("Mountinfo changed: {}", changes);

                    let event = create_event(
                        EventDomain::Storage,
                        format!("Mount state changed: {}", changes),
                    );

                    if let Err(e) = tx.send(event) {
                        warn!("Failed to send storage event: {}", e);
                        break;
                    }

                    last_content = current;
                }
            }
            Err(e) => {
                warn!("Failed to read mountinfo: {}", e);
            }
        }
    }

    Ok(())
}

/// Read /proc/self/mountinfo
fn read_mountinfo() -> Result<String> {
    fs::read_to_string(MOUNTINFO_PATH)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", MOUNTINFO_PATH, e))
}

/// Detect what changed between two mountinfo states
fn detect_changes(old: &str, new: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let added = new_lines.len().saturating_sub(old_lines.len());
    let removed = old_lines.len().saturating_sub(new_lines.len());

    if added > 0 && removed == 0 {
        format!("{} mount(s) added", added)
    } else if removed > 0 && added == 0 {
        format!("{} mount(s) removed", removed)
    } else if added > 0 || removed > 0 {
        format!("{} added, {} removed", added, removed)
    } else {
        "mount options changed".to_string()
    }
}

/// Simulate a storage event (for testing)
#[cfg(test)]
pub fn simulate_event(cause: &str) -> SystemEvent {
    create_event(EventDomain::Storage, cause)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_mountinfo() {
        // Should succeed on Linux
        let result = read_mountinfo();
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_detect_changes() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline2\nline3\nline4";

        let changes = detect_changes(old, new);
        assert!(changes.contains("added"));
    }

    #[test]
    fn test_simulate_event() {
        let event = simulate_event("test mount");
        assert_eq!(event.domain, EventDomain::Storage);
        assert!(event.cause.contains("test mount"));
    }
}
