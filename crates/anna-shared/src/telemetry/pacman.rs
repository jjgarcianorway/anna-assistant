//! Pacman telemetry - tracks package install/upgrade/removal events.
//!
//! Parses /var/log/pacman.log to extract recent package changes.
//! Uses checkpoint offsets to avoid re-reading entire log.
//!
//! v0.0.29: Initial implementation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Maximum recent events to store
const MAX_RECENT_EVENTS: usize = 10;

/// Default pacman log path
pub const PACMAN_LOG_PATH: &str = "/var/log/pacman.log";

/// Package event type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageEventKind {
    Installed,
    Upgraded,
    Removed,
    Reinstalled,
}

impl std::fmt::Display for PackageEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installed => write!(f, "installed"),
            Self::Upgraded => write!(f, "upgraded"),
            Self::Removed => write!(f, "removed"),
            Self::Reinstalled => write!(f, "reinstalled"),
        }
    }
}

/// Single package event from pacman log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEvent {
    /// Event timestamp (from log)
    pub timestamp: String,
    /// Event kind
    pub kind: PackageEventKind,
    /// Package name
    pub package: String,
    /// Version (new version for install/upgrade, old version for remove)
    pub version: Option<String>,
}

impl PackageEvent {
    pub fn new(kind: PackageEventKind, package: impl Into<String>) -> Self {
        Self {
            timestamp: String::new(),
            kind,
            package: package.into(),
            version: None,
        }
    }

    pub fn with_timestamp(mut self, ts: impl Into<String>) -> Self {
        self.timestamp = ts.into();
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Pacman telemetry snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PacmanSnapshot {
    /// Checkpoint: byte offset in log file when last read
    pub checkpoint_offset: u64,
    /// Recent package events (most recent first)
    pub recent_events: Vec<PackageEvent>,
    /// Timestamp when snapshot was captured (epoch seconds)
    pub captured_at_ts: u64,
}

impl PacmanSnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        let captured_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            captured_at_ts,
            ..Default::default()
        }
    }

    /// Add an event, maintaining max size
    pub fn add_event(&mut self, event: PackageEvent) {
        self.recent_events.insert(0, event);
        while self.recent_events.len() > MAX_RECENT_EVENTS {
            self.recent_events.pop();
        }
    }

    /// Count events by kind
    pub fn count_by_kind(&self, kind: PackageEventKind) -> usize {
        self.recent_events.iter().filter(|e| e.kind == kind).count()
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        if self.recent_events.is_empty() {
            return "no recent package changes".to_string();
        }

        let installed = self.count_by_kind(PackageEventKind::Installed);
        let upgraded = self.count_by_kind(PackageEventKind::Upgraded);
        let removed = self.count_by_kind(PackageEventKind::Removed);

        let mut parts = Vec::new();
        if installed > 0 {
            parts.push(format!("{} installed", installed));
        }
        if upgraded > 0 {
            parts.push(format!("{} upgraded", upgraded));
        }
        if removed > 0 {
            parts.push(format!("{} removed", removed));
        }

        if parts.is_empty() {
            "recent package activity".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Parse a single line from pacman.log
///
/// Format: [YYYY-MM-DDTHH:MM:SS+ZZZZ] [ALPM] <action> <package> (<version>)
///
/// Examples:
/// - [2024-12-05T10:30:15+0100] [ALPM] installed vim (9.0.2155-1)
/// - [2024-12-05T10:31:20+0100] [ALPM] upgraded firefox (130.0-1 -> 131.0-1)
/// - [2024-12-05T10:32:00+0100] [ALPM] removed nano (7.2-1)
pub fn parse_pacman_log_line(line: &str) -> Option<PackageEvent> {
    let line = line.trim();

    // Must start with timestamp
    if !line.starts_with('[') {
        return None;
    }

    // Find ALPM section
    if !line.contains("[ALPM]") {
        return None;
    }

    // Extract timestamp
    let ts_end = line.find(']')?;
    let timestamp = &line[1..ts_end];

    // Find action after [ALPM]
    let alpm_pos = line.find("[ALPM]")?;
    let after_alpm = &line[alpm_pos + 7..].trim();

    // Parse action
    let parts: Vec<&str> = after_alpm.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return None;
    }

    let action = parts[0].to_lowercase();
    let package_part = parts[1];

    let kind = match action.as_str() {
        "installed" => PackageEventKind::Installed,
        "upgraded" => PackageEventKind::Upgraded,
        "removed" => PackageEventKind::Removed,
        "reinstalled" => PackageEventKind::Reinstalled,
        _ => return None,
    };

    // Extract version if present (in parentheses)
    let version = if parts.len() >= 3 {
        let v = parts[2..].join(" ");
        let v = v.trim_start_matches('(').trim_end_matches(')');
        if !v.is_empty() {
            Some(v.to_string())
        } else {
            None
        }
    } else {
        None
    };

    Some(
        PackageEvent::new(kind, package_part)
            .with_timestamp(timestamp)
            .with_version(version.unwrap_or_default()),
    )
}

/// Parse pacman log from a given offset, returning new events
pub fn parse_pacman_log_from_offset(
    log_path: &str,
    start_offset: u64,
) -> std::io::Result<(Vec<PackageEvent>, u64)> {
    use std::io::{BufRead, BufReader, Seek, SeekFrom};

    let file = std::fs::File::open(log_path)?;
    let file_len = file.metadata()?.len();

    // If file is smaller than offset, log was rotated - start from beginning
    let actual_offset = if file_len < start_offset {
        0
    } else {
        start_offset
    };

    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(actual_offset))?;

    let mut events = Vec::new();
    let mut current_offset = actual_offset;

    for line in reader.lines() {
        let line = line?;
        current_offset += line.len() as u64 + 1; // +1 for newline

        if let Some(event) = parse_pacman_log_line(&line) {
            events.push(event);
        }
    }

    Ok((events, current_offset))
}

/// Get pacman snapshot file path
pub fn pacman_snapshot_path() -> PathBuf {
    super::telemetry_dir().join("pacman.json")
}

/// Load pacman snapshot from disk
pub fn load_pacman_snapshot() -> Option<PacmanSnapshot> {
    let path = pacman_snapshot_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(snap) = serde_json::from_str(&content) {
                return Some(snap);
            }
        }
    }
    None
}

/// Save pacman snapshot to disk
pub fn save_pacman_snapshot(snap: &PacmanSnapshot) -> std::io::Result<()> {
    let path = pacman_snapshot_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(snap)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, content)
}

/// Update pacman snapshot with recent log entries
pub fn update_pacman_snapshot() -> Option<PacmanSnapshot> {
    let log_path = PACMAN_LOG_PATH;

    // Check if log exists
    if !std::path::Path::new(log_path).exists() {
        return None;
    }

    let mut snap = load_pacman_snapshot().unwrap_or_else(PacmanSnapshot::new);

    // Parse new entries from checkpoint
    if let Ok((events, new_offset)) = parse_pacman_log_from_offset(log_path, snap.checkpoint_offset)
    {
        for event in events {
            snap.add_event(event);
        }
        snap.checkpoint_offset = new_offset;
        snap.captured_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if save_pacman_snapshot(&snap).is_ok() {
            return Some(snap);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_installed() {
        let line = "[2024-12-05T10:30:15+0100] [ALPM] installed vim (9.0.2155-1)";
        let event = parse_pacman_log_line(line).unwrap();

        assert_eq!(event.kind, PackageEventKind::Installed);
        assert_eq!(event.package, "vim");
        assert_eq!(event.version, Some("9.0.2155-1".to_string()));
        assert_eq!(event.timestamp, "2024-12-05T10:30:15+0100");
    }

    #[test]
    fn test_parse_upgraded() {
        let line = "[2024-12-05T10:31:20+0100] [ALPM] upgraded firefox (130.0-1 -> 131.0-1)";
        let event = parse_pacman_log_line(line).unwrap();

        assert_eq!(event.kind, PackageEventKind::Upgraded);
        assert_eq!(event.package, "firefox");
    }

    #[test]
    fn test_parse_removed() {
        let line = "[2024-12-05T10:32:00+0100] [ALPM] removed nano (7.2-1)";
        let event = parse_pacman_log_line(line).unwrap();

        assert_eq!(event.kind, PackageEventKind::Removed);
        assert_eq!(event.package, "nano");
    }

    #[test]
    fn test_parse_invalid_lines() {
        assert!(parse_pacman_log_line("").is_none());
        assert!(parse_pacman_log_line("random text").is_none());
        assert!(parse_pacman_log_line("[2024-12-05] not alpm").is_none());
        assert!(parse_pacman_log_line("[2024-12-05] [ALPM] unknown action pkg").is_none());
    }

    #[test]
    fn test_pacman_snapshot_add_event() {
        let mut snap = PacmanSnapshot::new();

        for i in 0..15 {
            snap.add_event(PackageEvent::new(
                PackageEventKind::Installed,
                format!("pkg{}", i),
            ));
        }

        // Should be capped at MAX_RECENT_EVENTS
        assert_eq!(snap.recent_events.len(), MAX_RECENT_EVENTS);
        // Most recent should be first
        assert_eq!(snap.recent_events[0].package, "pkg14");
    }

    #[test]
    fn test_pacman_snapshot_summary() {
        let mut snap = PacmanSnapshot::new();
        assert_eq!(snap.summary(), "no recent package changes");

        snap.add_event(PackageEvent::new(PackageEventKind::Installed, "vim"));
        snap.add_event(PackageEvent::new(PackageEventKind::Installed, "nano"));
        snap.add_event(PackageEvent::new(PackageEventKind::Upgraded, "firefox"));

        let summary = snap.summary();
        assert!(summary.contains("2 installed"));
        assert!(summary.contains("1 upgraded"));
    }

    #[test]
    fn test_pacman_snapshot_serialization() {
        let mut snap = PacmanSnapshot::new();
        snap.checkpoint_offset = 12345;
        snap.add_event(
            PackageEvent::new(PackageEventKind::Installed, "vim")
                .with_timestamp("2024-12-05T10:30:00+0100")
                .with_version("9.0-1"),
        );

        let json = serde_json::to_string(&snap).unwrap();
        let parsed: PacmanSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.checkpoint_offset, 12345);
        assert_eq!(parsed.recent_events.len(), 1);
        assert_eq!(parsed.recent_events[0].package, "vim");
    }
}
