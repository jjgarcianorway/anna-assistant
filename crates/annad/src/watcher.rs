//! System Watcher - Monitors system changes and triggers advice refresh
//!
//! Watches for:
//! - Package installations/removals (pacman database)
//! - System reboots (uptime tracking)
//! - Configuration changes (important system files)

use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher, Event};
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[allow(dead_code)]
pub enum SystemEvent {
    PackageChange,
    ConfigChange(String),
    Reboot,
}

pub struct SystemWatcher {
    _watcher: RecommendedWatcher,
}

impl SystemWatcher {
    pub fn new(tx: mpsc::UnboundedSender<SystemEvent>) -> Result<Self> {
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Some(path) = event.paths.first() {
                        let path_str = path.to_string_lossy();
                        
                        // Check what changed
                        if path_str.contains("/var/lib/pacman/local") {
                            info!("Package database changed");
                            let _ = tx.send(SystemEvent::PackageChange);
                        } else if path_str.contains("/etc/pacman.conf") 
                               || path_str.contains("/etc/ssh/sshd_config")
                               || path_str.contains("/etc/fstab") {
                            info!("Configuration file changed: {}", path_str);
                            let _ = tx.send(SystemEvent::ConfigChange(path_str.to_string()));
                        }
                    }
                }
                Err(e) => warn!("Watch error: {:?}", e),
            }
        })?;

        // Watch pacman database for package changes
        watcher.watch(Path::new("/var/lib/pacman/local"), RecursiveMode::NonRecursive)?;
        
        // Watch important config files
        if Path::new("/etc/pacman.conf").exists() {
            watcher.watch(Path::new("/etc/pacman.conf"), RecursiveMode::NonRecursive)?;
        }
        if Path::new("/etc/ssh/sshd_config").exists() {
            watcher.watch(Path::new("/etc/ssh/sshd_config"), RecursiveMode::NonRecursive)?;
        }
        if Path::new("/etc/fstab").exists() {
            watcher.watch(Path::new("/etc/fstab"), RecursiveMode::NonRecursive)?;
        }

        info!("System watcher initialized");

        Ok(Self {
            _watcher: watcher,
        })
    }
}

/// Check if system was recently rebooted
pub async fn check_reboot(last_check: std::time::Instant) -> bool {
    // Read system uptime
    if let Ok(uptime_str) = tokio::fs::read_to_string("/proc/uptime").await {
        if let Some(uptime_secs) = uptime_str.split_whitespace().next() {
            if let Ok(uptime) = uptime_secs.parse::<f64>() {
                let uptime_duration = std::time::Duration::from_secs_f64(uptime);
                let elapsed_since_check = last_check.elapsed();
                
                // If uptime is less than time since last check, system rebooted!
                if uptime_duration < elapsed_since_check {
                    info!("System reboot detected!");
                    return true;
                }
            }
        }
    }
    
    false
}
