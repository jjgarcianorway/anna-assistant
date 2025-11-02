// Anna v0.11.0 - Packages Listener
//
// Watches /var/lib/pacman/local for package database changes.
// Detects package install/upgrade/remove operations.
// Triggers packages domain checks when changes detected.

use crate::events::{create_event, EventDomain, SystemEvent};
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

const PACMAN_DB_PATH: &str = "/var/lib/pacman/local";
const POLL_INTERVAL_SECS: u64 = 5;

/// Spawn packages listener task
pub fn spawn_listener(tx: mpsc::UnboundedSender<SystemEvent>) -> JoinHandle<()> {
    info!(
        "Starting packages listener (poll interval: {}s)",
        POLL_INTERVAL_SECS
    );

    tokio::spawn(async move {
        if let Err(e) = watch_packages(tx).await {
            warn!("Packages listener error: {}", e);
        }
    })
}

/// Watch pacman database for changes
async fn watch_packages(tx: mpsc::UnboundedSender<SystemEvent>) -> Result<()> {
    // Check if pacman database exists
    if !Path::new(PACMAN_DB_PATH).exists() {
        warn!(
            "Pacman database not found at {}, packages listener disabled",
            PACMAN_DB_PATH
        );
        return Ok(());
    }

    let mut last_mtime = get_mtime(PACMAN_DB_PATH)?;
    let mut last_count = count_packages(PACMAN_DB_PATH)?;

    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

        match get_mtime(PACMAN_DB_PATH) {
            Ok(current_mtime) => {
                if current_mtime != last_mtime {
                    let current_count = count_packages(PACMAN_DB_PATH)?;

                    let change_type = if current_count > last_count {
                        format!("Package(s) installed (+{})", current_count - last_count)
                    } else if current_count < last_count {
                        format!("Package(s) removed (-{})", last_count - current_count)
                    } else {
                        "Package(s) upgraded".to_string()
                    };

                    debug!("Package database changed: {}", change_type);

                    let event = create_event(EventDomain::Packages, change_type);

                    if let Err(e) = tx.send(event) {
                        warn!("Failed to send packages event: {}", e);
                        break;
                    }

                    last_mtime = current_mtime;
                    last_count = current_count;
                }
            }
            Err(e) => {
                warn!("Failed to get pacman database mtime: {}", e);
            }
        }
    }

    Ok(())
}

/// Get modification time of a directory
fn get_mtime(path: &str) -> Result<SystemTime> {
    let metadata = fs::metadata(path)
        .map_err(|e| anyhow::anyhow!("Failed to get metadata for {}: {}", path, e))?;

    metadata
        .modified()
        .map_err(|e| anyhow::anyhow!("Failed to get mtime: {}", e))
}

/// Count packages in pacman database
fn count_packages(path: &str) -> Result<usize> {
    let entries = fs::read_dir(path)
        .map_err(|e| anyhow::anyhow!("Failed to read directory {}: {}", path, e))?;

    let count = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .count();

    Ok(count)
}

/// Simulate a packages event (for testing)
#[cfg(test)]
pub fn simulate_event(cause: &str) -> SystemEvent {
    create_event(EventDomain::Packages, cause)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_db_exists() {
        // This test will pass if running on Arch Linux
        // On other systems, it will show the path doesn't exist (expected)
        let exists = Path::new(PACMAN_DB_PATH).exists();
        println!("Pacman DB exists: {}", exists);
    }

    #[test]
    fn test_simulate_event() {
        let event = simulate_event("test package install");
        assert_eq!(event.domain, EventDomain::Packages);
        assert!(event.cause.contains("test package"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_count_packages() {
        if Path::new(PACMAN_DB_PATH).exists() {
            let count = count_packages(PACMAN_DB_PATH);
            assert!(count.is_ok());
            println!("Package count: {}", count.unwrap());
        }
    }
}
