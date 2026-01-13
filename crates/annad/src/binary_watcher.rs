//! Binary change watcher for automatic daemon restart after local rebuilds.
//!
//! v0.1.1: Auto-restart when binary changes (solves the "why isn't my code running" problem)
//!
//! This watches the daemon binary file and triggers a restart when it changes.
//! Essential for development workflow where you rebuild frequently.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};
use tokio::time::interval;
use tracing::{info, warn, debug};

/// Get the path to the current executable
fn get_binary_path() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

/// Get modification time and inode of the binary
fn get_binary_info(path: &PathBuf) -> Option<(SystemTime, u64)> {
    fs::metadata(path)
        .ok()
        .map(|m| (m.modified().unwrap_or(SystemTime::UNIX_EPOCH), m.ino()))
}

/// Background loop that watches for binary changes
pub async fn binary_watch_loop() {
    let binary_path = match get_binary_path() {
        Some(p) => p,
        None => {
            warn!("Could not determine binary path, binary watch disabled");
            return;
        }
    };

    let initial_info = match get_binary_info(&binary_path) {
        Some(info) => info,
        None => {
            warn!("Could not get binary info, binary watch disabled");
            return;
        }
    };

    info!(
        "Binary watch started: {} (inode: {})",
        binary_path.display(),
        initial_info.1
    );

    let mut check_interval = interval(Duration::from_secs(5));
    let mut last_info = initial_info;

    loop {
        check_interval.tick().await;

        if let Some(current_info) = get_binary_info(&binary_path) {
            // Check if mtime or inode changed (inode changes on cp/mv)
            if current_info.0 != last_info.0 || current_info.1 != last_info.1 {
                info!(
                    "Binary changed! mtime: {:?} -> {:?}, inode: {} -> {}",
                    last_info.0, current_info.0, last_info.1, current_info.1
                );
                info!("Triggering daemon restart...");

                // Give a moment for any in-flight requests
                tokio::time::sleep(Duration::from_millis(500)).await;

                trigger_restart();
                return;
            }
        }

        debug!("Binary unchanged, continuing...");
    }
}

/// Trigger a daemon restart
/// v0.3.12: Use exec() for dev mode restart instead of relying on systemd
fn trigger_restart() {
    // Get current binary path for exec
    let binary_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            warn!("Cannot determine binary path: {}", e);
            std::process::exit(0);
        }
    };

    // Check if we're running under systemd
    let under_systemd = std::env::var("INVOCATION_ID").is_ok();

    if under_systemd {
        // Running under systemd - let systemd handle the restart
        info!("Running under systemd, exiting for restart...");
        std::process::exit(0);
    } else {
        // Dev mode - exec the new binary directly
        info!("Dev mode: exec'ing new binary...");

        // Use exec to replace current process with new binary
        use std::os::unix::process::CommandExt;
        let err = Command::new(&binary_path)
            .args(std::env::args().skip(1))
            .exec();

        // exec() only returns on error
        warn!("exec failed: {}", err);
        std::process::exit(1);
    }
}
