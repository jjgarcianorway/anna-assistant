//! Historian v1 - Durable Health & Telemetry History
//!
//! Beta.279: Historian v1 & Proactive Rules Completion
//!
//! This module implements a simple, deterministic on-disk history store that:
//! - Records key health and telemetry information over time
//! - Exposes a narrow, efficient API to the proactive engine
//! - Allows completing correlation rules that depend on temporal context
//! - Maintains bounded retention (last N entries or M days)
//!
//! Architecture principles:
//! - Append-only writes to JSONL format
//! - Bounded retention (512 entries for v1)
//! - Graceful degradation on errors (never crash annad)
//! - Schema versioning for future evolution
//! - Minimal I/O overhead per health check
//!
//! Citation: [PROACTIVE_ENGINE_DESIGN.md], [ROOT_CAUSE_CORRELATION_MATRIX.md]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, error, warn};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Schema version for history events
const SCHEMA_VERSION: u8 = 1;

/// Default retention: last N entries
const DEFAULT_RETENTION_ENTRIES: usize = 512;

/// History file name
const HISTORY_FILENAME: &str = "history.jsonl";

// ============================================================================
// DATA MODEL
// ============================================================================

/// Compact history event for on-disk storage
///
/// This struct is append to disk after each health check cycle.
/// Keep fields compact and focused on proactive correlation needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    /// Schema version for backward compatibility
    pub schema_version: u8,

    /// Timestamp (UTC, RFC3339)
    pub timestamp_utc: DateTime<Utc>,

    /// Kernel version
    pub kernel_version: String,

    /// Hostname
    pub hostname: String,

    /// Root partition disk usage (0-100%)
    pub disk_root_usage_pct: u8,

    /// Maximum disk usage across other partitions (0-100%)
    pub disk_other_max_usage_pct: u8,

    /// Count of failed systemd services
    pub failed_services_count: u16,

    /// Count of degraded systemd services
    pub degraded_services_count: u16,

    /// High CPU flag (>80% sustained)
    pub high_cpu_flag: bool,

    /// High memory flag (>85% sustained)
    pub high_memory_flag: bool,

    /// Network packet loss percentage (0-100, max observed)
    pub network_packet_loss_pct: u8,

    /// Network latency milliseconds (max observed)
    pub network_latency_ms: u16,

    /// Boot ID or hash (to detect reboots)
    pub boot_id: String,

    /// Kernel changed since previous event
    pub kernel_changed: bool,

    /// Device hotplug flag (USB/PCIe events)
    pub device_hotplug_flag: bool,
}

impl HistoryEvent {
    /// Create a new history event with current timestamp
    pub fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            timestamp_utc: Utc::now(),
            kernel_version: String::new(),
            hostname: String::new(),
            disk_root_usage_pct: 0,
            disk_other_max_usage_pct: 0,
            failed_services_count: 0,
            degraded_services_count: 0,
            high_cpu_flag: false,
            high_memory_flag: false,
            network_packet_loss_pct: 0,
            network_latency_ms: 0,
            boot_id: String::new(),
            kernel_changed: false,
            device_hotplug_flag: false,
        }
    }
}

impl Default for HistoryEvent {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HISTORIAN
// ============================================================================

/// Historian manages the on-disk history store
///
/// This struct handles:
/// - Appending new history events
/// - Loading recent history for queries
/// - Enforcing retention policy
/// - Graceful error handling
pub struct Historian {
    /// Path to history.jsonl file
    history_path: PathBuf,

    /// Retention policy: max entries
    max_entries: usize,

    /// Last known boot ID for kernel change detection
    last_boot_id: Option<String>,

    /// Last known kernel version for change detection
    last_kernel_version: Option<String>,
}

impl Historian {
    /// Create a new historian with default retention
    ///
    /// # Arguments
    /// * `base_dir` - Base directory for Anna state (e.g., /var/lib/anna/state)
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let history_path = base_dir.as_ref().join(HISTORY_FILENAME);

        // Ensure base directory exists
        if let Some(parent) = history_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create historian directory: {:?}", parent))?;
        }

        Ok(Self {
            history_path,
            max_entries: DEFAULT_RETENTION_ENTRIES,
            last_boot_id: None,
            last_kernel_version: None,
        })
    }

    /// Append a history event to disk
    ///
    /// This is called after each health check cycle.
    /// Never panics - logs errors and continues gracefully.
    pub fn append(&mut self, event: &HistoryEvent) -> Result<()> {
        // Update last known values for change detection
        self.last_boot_id = Some(event.boot_id.clone());
        self.last_kernel_version = Some(event.kernel_version.clone());

        // Serialize event to JSON line
        let json_line = serde_json::to_string(event)
            .with_context(|| "Failed to serialize history event")?;

        // Append to file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.history_path)
            .with_context(|| format!("Failed to open history file: {:?}", self.history_path))?;

        let mut writer = BufWriter::new(file);
        writeln!(writer, "{}", json_line)
            .with_context(|| "Failed to write history event")?;

        writer.flush()
            .with_context(|| "Failed to flush history file")?;

        debug!(
            "Historian: Appended event at {}",
            event.timestamp_utc.to_rfc3339()
        );

        // Check if rotation is needed
        self.rotate_if_needed()?;

        Ok(())
    }

    /// Load all history events from disk
    ///
    /// Returns events in chronological order (oldest first).
    /// Skips corrupted lines and continues gracefully.
    pub fn load_all(&self) -> Result<Vec<HistoryEvent>> {
        if !self.history_path.exists() {
            debug!("Historian: No history file exists yet");
            return Ok(Vec::new());
        }

        let file = File::open(&self.history_path)
            .with_context(|| format!("Failed to open history file: {:?}", self.history_path))?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();
        let mut line_num = 0;

        for line in reader.lines() {
            line_num += 1;

            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    warn!("Historian: Skipping corrupted line {}: {}", line_num, e);
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<HistoryEvent>(&line) {
                Ok(event) => {
                    // Validate schema version
                    if event.schema_version > SCHEMA_VERSION {
                        warn!(
                            "Historian: Skipping future schema version {} at line {}",
                            event.schema_version, line_num
                        );
                        continue;
                    }
                    events.push(event);
                }
                Err(e) => {
                    warn!("Historian: Skipping unparseable line {}: {}", line_num, e);
                    continue;
                }
            }
        }

        debug!("Historian: Loaded {} events from disk", events.len());
        Ok(events)
    }

    /// Load recent history within a time window
    ///
    /// Returns events in chronological order (oldest first).
    pub fn load_recent(&self, window: chrono::Duration) -> Result<Vec<HistoryEvent>> {
        let all_events = self.load_all()?;
        let cutoff = Utc::now() - window;

        let recent: Vec<_> = all_events
            .into_iter()
            .filter(|e| e.timestamp_utc >= cutoff)
            .collect();

        debug!(
            "Historian: Loaded {} events within {:?} window",
            recent.len(),
            window
        );

        Ok(recent)
    }

    /// Rotate history file if it exceeds retention limit
    ///
    /// Keeps only the last N entries.
    fn rotate_if_needed(&self) -> Result<()> {
        let events = self.load_all()?;

        if events.len() <= self.max_entries {
            return Ok(()); // No rotation needed
        }

        debug!(
            "Historian: Rotating history ({}  entries, max {})",
            events.len(),
            self.max_entries
        );

        // Keep only last max_entries
        let keep_from = events.len() - self.max_entries;
        let kept_events = &events[keep_from..];

        // Write to temporary file
        let temp_path = self.history_path.with_extension("jsonl.tmp");
        let file = File::create(&temp_path)
            .with_context(|| format!("Failed to create temp history file: {:?}", temp_path))?;

        let mut writer = BufWriter::new(file);
        for event in kept_events {
            let json_line = serde_json::to_string(event)?;
            writeln!(writer, "{}", json_line)?;
        }
        writer.flush()?;

        // Atomic rename
        std::fs::rename(&temp_path, &self.history_path)
            .with_context(|| "Failed to rotate history file")?;

        debug!("Historian: Rotation complete, kept {} entries", kept_events.len());
        Ok(())
    }

    /// Get last known boot ID (for kernel change detection)
    pub fn last_boot_id(&self) -> Option<&str> {
        self.last_boot_id.as_deref()
    }

    /// Get last known kernel version (for kernel change detection)
    pub fn last_kernel_version(&self) -> Option<&str> {
        self.last_kernel_version.as_deref()
    }

    /// Reset historian state (for testing)
    #[cfg(test)]
    pub fn reset(&mut self) -> Result<()> {
        if self.history_path.exists() {
            std::fs::remove_file(&self.history_path)?;
        }
        self.last_boot_id = None;
        self.last_kernel_version = None;
        Ok(())
    }

    /// Set max entries (for testing)
    ///
    /// This method is available for both unit and integration tests.
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Safe append with error recovery
///
/// This function attempts to append an event and recovers gracefully on failure.
/// Never panics.
pub fn safe_append(historian: &mut Historian, event: &HistoryEvent) {
    if let Err(e) = historian.append(event) {
        error!("Historian: Failed to append event: {}. History will be incomplete.", e);
        // Continue without panicking - historian failure should not crash daemon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_historian() {
        let temp_dir = TempDir::new().unwrap();
        let historian = Historian::new(temp_dir.path()).unwrap();
        assert!(historian.history_path.parent().unwrap().exists());
    }

    #[test]
    fn test_append_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut historian = Historian::new(temp_dir.path()).unwrap();

        let mut event = HistoryEvent::new();
        event.hostname = "test-host".to_string();
        event.disk_root_usage_pct = 75;

        historian.append(&event).unwrap();

        let loaded = historian.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].hostname, "test-host");
        assert_eq!(loaded[0].disk_root_usage_pct, 75);
    }

    #[test]
    fn test_retention_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let mut historian = Historian::new(temp_dir.path()).unwrap();
        historian.max_entries = 10; // Small limit for testing

        // Append 15 events
        for i in 0..15 {
            let mut event = HistoryEvent::new();
            event.hostname = format!("host-{}", i);
            historian.append(&event).unwrap();
        }

        // Should have rotated to keep only last 10
        let loaded = historian.load_all().unwrap();
        assert_eq!(loaded.len(), 10);
        assert_eq!(loaded[0].hostname, "host-5"); // First kept entry
        assert_eq!(loaded[9].hostname, "host-14"); // Last entry
    }

    #[test]
    fn test_load_recent() {
        let temp_dir = TempDir::new().unwrap();
        let mut historian = Historian::new(temp_dir.path()).unwrap();

        // Append events at different times
        for i in 0..5 {
            let mut event = HistoryEvent::new();
            event.timestamp_utc = Utc::now() - chrono::Duration::hours(i);
            event.hostname = format!("host-{}", i);
            historian.append(&event).unwrap();
        }

        // Load last 2 hours
        let recent = historian.load_recent(chrono::Duration::hours(2)).unwrap();
        assert!(recent.len() >= 2); // Should include events from last 2 hours
    }

    #[test]
    fn test_corrupted_line_handling() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join(HISTORY_FILENAME);

        // Write a valid event followed by corrupted data
        let mut event = HistoryEvent::new();
        event.hostname = "valid-host".to_string();
        let valid_json = serde_json::to_string(&event).unwrap();

        let content = format!("{}\n{{\n{}\n", valid_json, valid_json);
        fs::write(&history_path, content).unwrap();

        let historian = Historian::new(temp_dir.path()).unwrap();
        let loaded = historian.load_all().unwrap();

        // Should load 2 valid events, skip corrupted line
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].hostname, "valid-host");
    }

    #[test]
    fn test_safe_append_with_error() {
        let temp_dir = TempDir::new().unwrap();
        let mut historian = Historian::new("/invalid/path/that/does/not/exist").unwrap_or_else(|_| {
            Historian::new(temp_dir.path()).unwrap()
        });

        let event = HistoryEvent::new();

        // Should not panic even if append fails
        safe_append(&mut historian, &event);
    }
}
