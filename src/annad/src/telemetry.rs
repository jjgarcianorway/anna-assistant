use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// Maximum number of daily event files to keep
const MAX_EVENT_FILES: usize = 5;

/// Event types logged by Anna
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    DaemonStarted,
    RpcCall {
        name: String,
        status: String,
    },
    ConfigChanged {
        scope: String,
        key: String,
    },
}

#[derive(Debug, Serialize)]
struct EventRecord {
    timestamp: DateTime<Utc>,
    #[serde(flatten)]
    event: Event,
}

/// Get the system-wide events directory
fn system_events_dir() -> PathBuf {
    PathBuf::from("/var/lib/anna/events")
}

/// Get the user-specific events directory
fn user_events_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".local/share/anna/events"))
}

/// Get the appropriate events directory based on context
fn events_dir() -> PathBuf {
    // If we can determine a user context, use user dir
    // Otherwise use system dir (we're root)
    if let Some(user_dir) = user_events_dir() {
        if user_dir.exists() || std::env::var("USER").is_ok() {
            return user_dir;
        }
    }
    system_events_dir()
}

/// Initialize telemetry system
pub fn init() -> Result<()> {
    let sys_dir = system_events_dir();
    if !sys_dir.exists() {
        fs::create_dir_all(&sys_dir)?;
    }

    // Also create user dir if we can determine the user
    if let Some(user_dir) = user_events_dir() {
        if let Ok(user) = std::env::var("USER") {
            if user != "root" && !user_dir.exists() {
                // Try to create, but don't fail if we can't
                let _ = fs::create_dir_all(&user_dir);
            }
        }
    }

    // Rotate old files
    rotate_old_files()?;

    Ok(())
}

/// Get today's event log file path
fn today_log_file() -> PathBuf {
    let dir = events_dir();
    let today = Utc::now().format("%Y-%m-%d");
    dir.join(format!("events_{}.jsonl", today))
}

/// Log an event to the appropriate file
pub fn log_event(event: Event) -> Result<()> {
    let log_file = today_log_file();

    // Ensure parent directory exists
    if let Some(parent) = log_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let record = EventRecord {
        timestamp: Utc::now(),
        event,
    };

    let json = serde_json::to_string(&record)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    writeln!(file, "{}", json)?;

    Ok(())
}

/// Rotate old event files, keeping only MAX_EVENT_FILES most recent
/// This is called automatically on init, but can also be triggered manually
pub fn rotate_old_files_now() -> Result<()> {
    rotate_old_files()
}

fn rotate_old_files() -> Result<()> {
    let dir = events_dir();
    if !dir.exists() {
        return Ok(());
    }

    let mut event_files: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() &&
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("events_") && n.ends_with(".jsonl"))
                .unwrap_or(false)
        })
        .collect();

    // Sort by filename (which includes date)
    event_files.sort();

    // Remove old files if we have more than MAX_EVENT_FILES
    if event_files.len() > MAX_EVENT_FILES {
        let to_remove = event_files.len() - MAX_EVENT_FILES;
        for file in event_files.iter().take(to_remove) {
            let _ = fs::remove_file(file);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = Event::DaemonStarted;
        let record = EventRecord {
            timestamp: Utc::now(),
            event,
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("daemon_started"));
    }
}
