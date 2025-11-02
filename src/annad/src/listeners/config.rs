// Anna v0.11.0 - Config Listener
//
// Watches critical /etc files for configuration drift.
// Uses polling for simplicity (inotify can be added later).
// Triggers config domain checks when files change.

use crate::events::{create_event, EventDomain, SystemEvent};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

const POLL_INTERVAL_SECS: u64 = 10;

/// Files to watch for configuration drift
const WATCH_PATHS: &[&str] = &[
    "/etc/resolv.conf",
    "/etc/fstab",
    "/etc/hostname",
    "/etc/hosts",
    "/etc/mkinitcpio.conf",
    "/etc/default/grub",
];

/// Spawn config listener task
pub fn spawn_listener(tx: mpsc::UnboundedSender<SystemEvent>) -> JoinHandle<()> {
    info!(
        "Starting config listener (poll interval: {}s, {} files)",
        POLL_INTERVAL_SECS,
        WATCH_PATHS.len()
    );

    tokio::spawn(async move {
        if let Err(e) = watch_config(tx).await {
            warn!("Config listener error: {}", e);
        }
    })
}

/// Watch config files for changes
async fn watch_config(tx: mpsc::UnboundedSender<SystemEvent>) -> Result<()> {
    let mut last_states = HashMap::new();

    // Initialize state
    for path_str in WATCH_PATHS {
        let path = PathBuf::from(path_str);
        if let Ok(state) = get_file_state(&path) {
            last_states.insert(path, state);
        }
    }

    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

        let mut changed_files = Vec::new();

        for path_str in WATCH_PATHS {
            let path = PathBuf::from(path_str);

            if !path.exists() {
                continue;
            }

            match get_file_state(&path) {
                Ok(current_state) => {
                    if let Some(last_state) = last_states.get(&path) {
                        if *last_state != current_state {
                            changed_files.push(path.display().to_string());
                            last_states.insert(path.clone(), current_state);
                        }
                    } else {
                        // New file appeared
                        changed_files.push(format!("{} (new)", path.display()));
                        last_states.insert(path, current_state);
                    }
                }
                Err(e) => {
                    debug!("Failed to get state for {:?}: {}", path, e);
                }
            }
        }

        if !changed_files.is_empty() {
            let cause = if changed_files.len() == 1 {
                format!("Config changed: {}", changed_files[0])
            } else {
                format!("Config changed: {} files", changed_files.len())
            };

            debug!("{}", cause);

            let event = create_event(EventDomain::Config, cause);

            if let Err(e) = tx.send(event) {
                warn!("Failed to send config event: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Get file state (mtime + size)
#[derive(Debug, PartialEq)]
struct FileState {
    mtime: SystemTime,
    size: u64,
}

fn get_file_state(path: &Path) -> Result<FileState> {
    let metadata = fs::metadata(path)
        .map_err(|e| anyhow::anyhow!("Failed to get metadata for {:?}: {}", path, e))?;

    let mtime = metadata
        .modified()
        .map_err(|e| anyhow::anyhow!("Failed to get mtime: {}", e))?;

    Ok(FileState {
        mtime,
        size: metadata.len(),
    })
}

/// Simulate a config event (for testing)
#[cfg(test)]
pub fn simulate_event(cause: &str) -> SystemEvent {
    create_event(EventDomain::Config, cause)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_paths_exist() {
        for path_str in WATCH_PATHS {
            let path = Path::new(path_str);
            if path.exists() {
                println!("{} exists", path_str);

                if let Ok(state) = get_file_state(path) {
                    println!("  mtime: {:?}, size: {}", state.mtime, state.size);
                }
            }
        }
    }

    #[test]
    fn test_simulate_event() {
        let event = simulate_event("test config change");
        assert_eq!(event.domain, EventDomain::Config);
        assert!(event.cause.contains("test config"));
    }
}
