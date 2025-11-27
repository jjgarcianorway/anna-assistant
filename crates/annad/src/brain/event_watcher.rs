//! Event Watcher v0.11.0
//!
//! Monitors system events to trigger learning jobs.

use anna_common::{LearningEvent, LearningJob};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Event watcher for system changes
pub struct EventWatcher {
    /// Channel to send learning events
    event_tx: mpsc::Sender<LearningJob>,
}

impl EventWatcher {
    pub fn new(event_tx: mpsc::Sender<LearningJob>) -> Self {
        Self { event_tx }
    }

    /// Start watching for events (spawns background tasks)
    pub fn start(&self) {
        info!("Starting event watcher");

        // Start pacman log watcher
        self.start_pacman_watcher();

        // Start periodic checks
        self.start_periodic_checks();
    }

    /// Watch pacman.log for package changes
    fn start_pacman_watcher(&self) {
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let pacman_log = PathBuf::from("/var/log/pacman.log");

            if !pacman_log.exists() {
                warn!("pacman.log not found, package change detection disabled");
                return;
            }

            // Read last position
            let mut last_pos = match tokio::fs::metadata(&pacman_log).await {
                Ok(meta) => meta.len(),
                Err(_) => 0,
            };

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                // Check for new content
                let current_size = match tokio::fs::metadata(&pacman_log).await {
                    Ok(meta) => meta.len(),
                    Err(_) => continue,
                };

                if current_size > last_pos {
                    // Read new lines
                    if let Ok(content) = tokio::fs::read_to_string(&pacman_log).await {
                        let new_content: String = content.chars().skip(last_pos as usize).collect();

                        for line in new_content.lines() {
                            if let Some(event) = Self::parse_pacman_line(line) {
                                let job = LearningJob::new(event);
                                if let Err(e) = tx.send(job).await {
                                    error!("Failed to send learning event: {}", e);
                                }
                            }
                        }
                    }

                    last_pos = current_size;
                }
            }
        });
    }

    /// Parse a pacman log line for package events
    fn parse_pacman_line(line: &str) -> Option<LearningEvent> {
        // Format: [2025-11-27T12:00:00+0100] [ALPM] installed vim (9.0.1)
        // Format: [2025-11-27T12:00:00+0100] [ALPM] removed vim (9.0.1)
        // Format: [2025-11-27T12:00:00+0100] [ALPM] upgraded vim (9.0.0 -> 9.0.1)

        if !line.contains("[ALPM]") {
            return None;
        }

        let parts: Vec<&str> = line.split(']').collect();
        if parts.len() < 3 {
            return None;
        }

        let action_part = parts[2].trim();

        if action_part.starts_with("installed ") {
            let rest = action_part.strip_prefix("installed ")?;
            let (name, version) = Self::parse_pkg_name_version(rest)?;
            return Some(LearningEvent::PackageAdded {
                name,
                version: Some(version),
            });
        }

        if action_part.starts_with("removed ") {
            let rest = action_part.strip_prefix("removed ")?;
            let (name, _) = Self::parse_pkg_name_version(rest)?;
            return Some(LearningEvent::PackageRemoved { name });
        }

        if action_part.starts_with("upgraded ") {
            let rest = action_part.strip_prefix("upgraded ")?;
            // Format: pkg (old_ver -> new_ver)
            let paren_pos = rest.find('(')?;
            let name = rest[..paren_pos].trim().to_string();
            let versions = &rest[paren_pos + 1..rest.len() - 1];
            let arrow_pos = versions.find("->")?;
            let old_version = versions[..arrow_pos].trim().to_string();
            let new_version = versions[arrow_pos + 2..].trim().to_string();

            return Some(LearningEvent::PackageUpgraded {
                name,
                old_version,
                new_version,
            });
        }

        None
    }

    /// Parse package name and version from "name (version)" format
    fn parse_pkg_name_version(s: &str) -> Option<(String, String)> {
        let paren_pos = s.find('(')?;
        let name = s[..paren_pos].trim().to_string();
        let version = s[paren_pos + 1..s.len() - 1].to_string();
        Some((name, version))
    }

    /// Start periodic checks for changes
    fn start_periodic_checks(&self) {
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            // Run hygiene check every 6 hours
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(6 * 3600));

            loop {
                interval.tick().await;

                debug!("Running periodic hygiene check");
                let event = LearningEvent::ScheduledRefresh {
                    target: "hygiene".to_string(),
                };
                let job = LearningJob::new(event);
                if let Err(e) = tx.send(job).await {
                    error!("Failed to send hygiene job: {}", e);
                }
            }
        });
    }
}

/// Create an event channel and watcher
pub fn create_event_channel() -> (mpsc::Sender<LearningJob>, mpsc::Receiver<LearningJob>) {
    mpsc::channel(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pacman_installed() {
        let line = "[2025-11-27T12:00:00+0100] [ALPM] installed vim (9.0.1)";
        let event = EventWatcher::parse_pacman_line(line);

        match event {
            Some(LearningEvent::PackageAdded { name, version }) => {
                assert_eq!(name, "vim");
                assert_eq!(version, Some("9.0.1".to_string()));
            }
            _ => panic!("Expected PackageAdded event"),
        }
    }

    #[test]
    fn test_parse_pacman_removed() {
        let line = "[2025-11-27T12:00:00+0100] [ALPM] removed vim (9.0.1)";
        let event = EventWatcher::parse_pacman_line(line);

        match event {
            Some(LearningEvent::PackageRemoved { name }) => {
                assert_eq!(name, "vim");
            }
            _ => panic!("Expected PackageRemoved event"),
        }
    }

    #[test]
    fn test_parse_pacman_upgraded() {
        let line = "[2025-11-27T12:00:00+0100] [ALPM] upgraded vim (9.0.0 -> 9.0.1)";
        let event = EventWatcher::parse_pacman_line(line);

        match event {
            Some(LearningEvent::PackageUpgraded {
                name,
                old_version,
                new_version,
            }) => {
                assert_eq!(name, "vim");
                assert_eq!(old_version, "9.0.0");
                assert_eq!(new_version, "9.0.1");
            }
            _ => panic!("Expected PackageUpgraded event"),
        }
    }

    #[test]
    fn test_parse_pacman_ignore_other() {
        let line = "[2025-11-27T12:00:00+0100] [PACMAN] Running 'pacman -Syu'";
        let event = EventWatcher::parse_pacman_line(line);
        assert!(event.is_none());
    }
}
