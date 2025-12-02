//! Change Journal v7.18.0 - What changed, when, and how
//!
//! Records concrete events from system logs and filesystem:
//! - Package lifecycle: install, upgrade, downgrade, remove
//! - Service lifecycle: enable, disable, mask, unmask
//! - Config file changes: mtime/content hash changes
//! - Kernel changes: install, remove, boot
//!
//! Sources:
//! - /var/log/pacman.log (package events)
//! - systemctl show (service state changes)
//! - Filesystem mtime + content hash (config changes)
//! - journalctl -k (kernel boot events)

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

/// Directory for change journal storage
pub const JOURNAL_DIR: &str = "/var/lib/anna/journal";
pub const JOURNAL_FILE: &str = "/var/lib/anna/journal/changes.jsonl";

/// Types of changes we track
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    PkgInstall,
    PkgUpgrade,
    PkgDowngrade,
    PkgRemove,
    SvcEnable,
    SvcDisable,
    SvcMask,
    SvcUnmask,
    ConfigChange,
    KernelInstall,
    KernelRemove,
    KernelBoot,
}

impl ChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::PkgInstall => "pkg_install",
            ChangeType::PkgUpgrade => "pkg_upgrade",
            ChangeType::PkgDowngrade => "pkg_downgrade",
            ChangeType::PkgRemove => "pkg_remove",
            ChangeType::SvcEnable => "svc_enable",
            ChangeType::SvcDisable => "svc_disable",
            ChangeType::SvcMask => "svc_mask",
            ChangeType::SvcUnmask => "svc_unmask",
            ChangeType::ConfigChange => "config_change",
            ChangeType::KernelInstall => "kernel_install",
            ChangeType::KernelRemove => "kernel_remove",
            ChangeType::KernelBoot => "kernel_boot",
        }
    }
}

/// A single change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    /// Unix timestamp
    pub timestamp: u64,
    /// Type of change
    pub change_type: ChangeType,
    /// Subject: package name, unit name, config path, or kernel version
    pub subject: String,
    /// Optional details (version, old/new state, hash)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ChangeDetails>,
}

/// Additional details for a change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetails {
    /// Current/new version (for installs and upgrades)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// New version (alias for upgrades, same as version)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_version: Option<String>,
    /// Previous version (for upgrades/downgrades)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_version: Option<String>,
    /// Previous state (for service changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_state: Option<String>,
    /// New state (for service changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_state: Option<String>,
    /// Content hash (for config file changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// Config file path (for config changes where subject is package name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
}

impl ChangeEvent {
    /// Create a new change event with current timestamp
    pub fn new(change_type: ChangeType, subject: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            timestamp,
            change_type,
            subject,
            details: None,
        }
    }

    /// Add details to the event
    pub fn with_details(mut self, details: ChangeDetails) -> Self {
        self.details = Some(details);
        self
    }

    /// Format timestamp for display
    pub fn format_time(&self) -> String {
        if let Some(dt) = DateTime::from_timestamp(self.timestamp as i64, 0) {
            let local: DateTime<Local> = dt.into();
            local.format("%Y-%m-%d %H:%M").to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Format for display in [RECENT CHANGES]
    pub fn format_short(&self) -> String {
        let time = self.format_time();
        let type_str = self.change_type.as_str();

        let detail = if let Some(ref d) = self.details {
            if let Some(ref v) = d.version {
                format!("  {}", v)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        format!("{}  {:14} {}{}", time, type_str, self.subject, detail)
    }
}

/// Change journal writer
pub struct ChangeJournalWriter {
    path: String,
}

impl ChangeJournalWriter {
    pub fn new() -> Self {
        Self {
            path: JOURNAL_FILE.to_string(),
        }
    }

    /// Ensure journal directory exists
    fn ensure_dir(&self) -> std::io::Result<()> {
        let dir = Path::new(&self.path).parent().unwrap_or(Path::new(JOURNAL_DIR));
        fs::create_dir_all(dir)
    }

    /// Append an event to the journal
    pub fn append(&self, event: &ChangeEvent) -> std::io::Result<()> {
        self.ensure_dir()?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let json = serde_json::to_string(event).unwrap_or_default();
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Append multiple events
    pub fn append_batch(&self, events: &[ChangeEvent]) -> std::io::Result<()> {
        self.ensure_dir()?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        for event in events {
            let json = serde_json::to_string(event).unwrap_or_default();
            writeln!(file, "{}", json)?;
        }

        Ok(())
    }
}

impl Default for ChangeJournalWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Change journal reader
pub struct ChangeJournalReader {
    path: String,
}

impl ChangeJournalReader {
    pub fn new() -> Self {
        Self {
            path: JOURNAL_FILE.to_string(),
        }
    }

    /// Read all events from journal
    pub fn read_all(&self) -> Vec<ChangeEvent> {
        let mut events = Vec::new();

        if let Ok(file) = File::open(&self.path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                if let Ok(event) = serde_json::from_str::<ChangeEvent>(&line) {
                    events.push(event);
                }
            }
        }

        events
    }

    /// Read events since a timestamp
    pub fn read_since(&self, since: u64) -> Vec<ChangeEvent> {
        self.read_all()
            .into_iter()
            .filter(|e| e.timestamp >= since)
            .collect()
    }

    /// Read last N events
    pub fn read_last(&self, count: usize) -> Vec<ChangeEvent> {
        let all = self.read_all();
        let start = all.len().saturating_sub(count);
        all[start..].to_vec()
    }

    /// Read events for a specific subject (package, service, config path)
    pub fn read_for_subject(&self, subject: &str) -> Vec<ChangeEvent> {
        self.read_all()
            .into_iter()
            .filter(|e| e.subject == subject || e.subject.contains(subject))
            .collect()
    }

    /// Read events of a specific type
    pub fn read_by_type(&self, change_type: ChangeType) -> Vec<ChangeEvent> {
        self.read_all()
            .into_iter()
            .filter(|e| e.change_type == change_type)
            .collect()
    }
}

impl Default for ChangeJournalReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse pacman.log to extract package events
pub fn scan_pacman_log(since: Option<u64>) -> Vec<ChangeEvent> {
    let mut events = Vec::new();
    let log_path = "/var/log/pacman.log";

    let file = match File::open(log_path) {
        Ok(f) => f,
        Err(_) => return events,
    };

    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        // Format: [2025-01-15T10:30:45+0100] [ALPM] installed nano (8.3-1)
        if !line.contains("[ALPM]") {
            continue;
        }

        // Parse timestamp
        let timestamp = parse_pacman_timestamp(&line);

        // Filter by since if provided
        if let Some(s) = since {
            if timestamp < s {
                continue;
            }
        }

        // Parse action and package
        if let Some(event) = parse_pacman_line(&line, timestamp) {
            events.push(event);
        }
    }

    events
}

/// Parse pacman.log timestamp
fn parse_pacman_timestamp(line: &str) -> u64 {
    // Format: [2025-01-15T10:30:45+0100]
    if let Some(start) = line.find('[') {
        if let Some(end) = line.find(']') {
            let ts_str = &line[start + 1..end];
            // Try to parse ISO 8601 with timezone
            if let Ok(dt) = DateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S%z") {
                return dt.timestamp() as u64;
            }
            // Try without timezone
            if let Ok(dt) = NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S") {
                if let Some(local) = Local.from_local_datetime(&dt).single() {
                    return local.timestamp() as u64;
                }
            }
        }
    }
    0
}

/// Parse a single pacman.log line into a ChangeEvent
fn parse_pacman_line(line: &str, timestamp: u64) -> Option<ChangeEvent> {
    // [timestamp] [ALPM] installed package (version)
    // [timestamp] [ALPM] upgraded package (old -> new)
    // [timestamp] [ALPM] removed package (version)
    // [timestamp] [ALPM] downgraded package (old -> new)

    let line_lower = line.to_lowercase();

    let (change_type, prefix) = if line_lower.contains("] installed ") {
        (ChangeType::PkgInstall, "installed ")
    } else if line_lower.contains("] upgraded ") {
        (ChangeType::PkgUpgrade, "upgraded ")
    } else if line_lower.contains("] removed ") {
        (ChangeType::PkgRemove, "removed ")
    } else if line_lower.contains("] downgraded ") {
        (ChangeType::PkgDowngrade, "downgraded ")
    } else {
        return None;
    };

    // Find the action in the line
    let action_start = line_lower.find(prefix)?;
    let rest = &line[action_start + prefix.len()..];

    // Extract package name and version
    let parts: Vec<&str> = rest.trim().splitn(2, ' ').collect();
    let pkg_name = parts.first()?.to_string();

    let details = if parts.len() > 1 {
        let version_part = parts[1].trim_matches(|c| c == '(' || c == ')');

        if version_part.contains(" -> ") {
            // upgrade/downgrade: old -> new
            let versions: Vec<&str> = version_part.split(" -> ").collect();
            Some(ChangeDetails {
                version: versions.get(1).map(|s| s.to_string()),
                new_version: versions.get(1).map(|s| s.to_string()),
                old_version: versions.first().map(|s| s.to_string()),
                old_state: None,
                new_state: None,
                content_hash: None,
                config_path: None,
            })
        } else {
            // install/remove: just version
            Some(ChangeDetails {
                version: Some(version_part.to_string()),
                new_version: Some(version_part.to_string()),
                old_version: None,
                old_state: None,
                new_state: None,
                content_hash: None,
                config_path: None,
            })
        }
    } else {
        None
    };

    let mut event = ChangeEvent {
        timestamp,
        change_type,
        subject: pkg_name,
        details: None,
    };

    if let Some(d) = details {
        event = event.with_details(d);
    }

    Some(event)
}

/// Track config file for changes (mtime and content hash)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTracker {
    pub path: String,
    pub last_mtime: u64,
    pub content_hash: String,
}

impl ConfigTracker {
    /// Create tracker for a config file
    pub fn new(path: &str) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        let mtime = metadata.modified().ok()?
            .duration_since(UNIX_EPOCH).ok()?
            .as_secs();

        let content = fs::read(path).ok()?;
        let hash = simple_hash(&content);

        Some(Self {
            path: path.to_string(),
            last_mtime: mtime,
            content_hash: hash,
        })
    }

    /// Check if file has changed
    pub fn has_changed(&self) -> Option<ChangeEvent> {
        let metadata = fs::metadata(&self.path).ok()?;
        let current_mtime = metadata.modified().ok()?
            .duration_since(UNIX_EPOCH).ok()?
            .as_secs();

        if current_mtime == self.last_mtime {
            return None;
        }

        // mtime changed, check content
        let content = fs::read(&self.path).ok()?;
        let current_hash = simple_hash(&content);

        if current_hash == self.content_hash {
            return None;
        }

        // Content actually changed
        let event = ChangeEvent::new(ChangeType::ConfigChange, self.path.clone())
            .with_details(ChangeDetails {
                version: None,
                new_version: None,
                old_version: None,
                old_state: None,
                new_state: None,
                content_hash: Some(current_hash),
                config_path: Some(self.path.clone()),
            });

        Some(event)
    }
}

/// Simple hash for content comparison (not cryptographic)
fn simple_hash(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Get kernel boot events from journalctl
pub fn scan_kernel_boots() -> Vec<ChangeEvent> {
    use std::process::Command;

    let mut events = Vec::new();

    // Get list of boots with timestamps
    let output = Command::new("journalctl")
        .args(["--list-boots", "--no-pager"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);

            for line in stdout.lines() {
                // Format: -1 boot_id timestamp timestamp
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    // Try to parse the boot timestamp
                    // Format varies, try to extract date and time
                    if let Some(ts) = parse_boot_timestamp(&parts[2..]) {
                        // Get kernel version for this boot
                        let boot_idx = parts[0];
                        if let Some(kernel) = get_kernel_for_boot(boot_idx) {
                            let event = ChangeEvent {
                                timestamp: ts,
                                change_type: ChangeType::KernelBoot,
                                subject: kernel,
                                details: None,
                            };
                            events.push(event);
                        }
                    }
                }
            }
        }
    }

    events
}

/// Parse boot timestamp from journalctl --list-boots output
fn parse_boot_timestamp(parts: &[&str]) -> Option<u64> {
    // Try to combine date and time parts
    if parts.len() >= 2 {
        let date_time = format!("{} {}", parts[0], parts[1]);
        // Try common formats
        for fmt in &["%Y-%m-%d %H:%M:%S", "%a %Y-%m-%d %H:%M:%S"] {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&date_time, fmt) {
                if let Some(local) = Local.from_local_datetime(&dt).single() {
                    return Some(local.timestamp() as u64);
                }
            }
        }
    }
    None
}

/// Get kernel version for a specific boot
fn get_kernel_for_boot(boot_idx: &str) -> Option<String> {
    use std::process::Command;

    let output = Command::new("journalctl")
        .args(["-b", boot_idx, "-k", "--no-pager", "-n", "50"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Look for Linux version line
    for line in stdout.lines() {
        if line.contains("Linux version") {
            // Extract version: "Linux version 6.17.9-arch1-1 (..."
            let parts: Vec<&str> = line.split("Linux version").collect();
            if parts.len() > 1 {
                let version_part = parts[1].trim();
                if let Some(version) = version_part.split_whitespace().next() {
                    return Some(version.to_string());
                }
            }
        }
    }

    None
}

/// Get history for a specific package
pub fn get_package_history(pkg_name: &str) -> Vec<ChangeEvent> {
    let reader = ChangeJournalReader::new();
    let mut events: Vec<ChangeEvent> = reader.read_for_subject(pkg_name)
        .into_iter()
        .filter(|e| matches!(
            e.change_type,
            ChangeType::PkgInstall | ChangeType::PkgUpgrade |
            ChangeType::PkgDowngrade | ChangeType::PkgRemove
        ))
        .collect();

    // Also scan pacman.log for events we might not have recorded yet
    let pacman_events = scan_pacman_log(None);
    for event in pacman_events {
        if event.subject == pkg_name && !events.iter().any(|e| e.timestamp == event.timestamp) {
            events.push(event);
        }
    }

    events.sort_by_key(|e| e.timestamp);
    events
}

/// Get history for a specific config path
pub fn get_config_history(config_path: &str) -> Vec<ChangeEvent> {
    let reader = ChangeJournalReader::new();
    reader.read_for_subject(config_path)
        .into_iter()
        .filter(|e| e.change_type == ChangeType::ConfigChange)
        .collect()
}

/// Get recent changes across all types
pub fn get_recent_changes(count: usize) -> Vec<ChangeEvent> {
    let reader = ChangeJournalReader::new();
    let mut events = reader.read_last(count * 2); // Read extra to account for filtering

    // Supplement with recent pacman events
    let week_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(7 * 24 * 3600);

    let pacman_events = scan_pacman_log(Some(week_ago));

    // Merge, deduplicate by timestamp+subject
    let mut seen: HashMap<(u64, String), bool> = HashMap::new();
    for e in &events {
        seen.insert((e.timestamp, e.subject.clone()), true);
    }

    for e in pacman_events {
        if !seen.contains_key(&(e.timestamp, e.subject.clone())) {
            events.push(e);
        }
    }

    // Sort by timestamp descending and take last N
    events.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
    events.truncate(count);

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_event_format() {
        let event = ChangeEvent::new(ChangeType::PkgInstall, "nano".to_string())
            .with_details(ChangeDetails {
                version: Some("8.7-1".to_string()),
                new_version: Some("8.7-1".to_string()),
                old_version: None,
                old_state: None,
                new_state: None,
                content_hash: None,
                config_path: None,
            });

        let short = event.format_short();
        assert!(short.contains("pkg_install"));
        assert!(short.contains("nano"));
    }

    #[test]
    fn test_parse_pacman_line() {
        let line = "[2025-01-15T10:30:45+0100] [ALPM] installed nano (8.3-1)";
        let timestamp = parse_pacman_timestamp(line);
        assert!(timestamp > 0);

        let event = parse_pacman_line(line, timestamp);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.change_type, ChangeType::PkgInstall);
        assert_eq!(event.subject, "nano");
    }

    #[test]
    fn test_simple_hash() {
        let hash1 = simple_hash(b"hello");
        let hash2 = simple_hash(b"hello");
        let hash3 = simple_hash(b"world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
