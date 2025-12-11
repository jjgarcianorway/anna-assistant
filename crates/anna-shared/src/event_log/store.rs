//! Event log file store (v0.0.190).
//! v0.0.404: Added clear() for reset functionality.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use super::aggregation::AggregatedEvents;
use super::types::EventRecord;

/// Event log store with rotation
pub struct EventLog {
    path: std::path::PathBuf,
    max_entries: usize,
}

impl EventLog {
    /// Create a new event log store
    pub fn new(path: impl AsRef<Path>, max_entries: usize) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            max_entries,
        }
    }

    /// Default location in state directory
    /// v0.0.169: Use user-writable path to ensure events are actually persisted
    pub fn default_path() -> std::path::PathBuf {
        // Try /var/lib/anna first (if daemon has access)
        let var_path = std::path::PathBuf::from("/var/lib/anna");
        if var_path.exists() && var_path.is_dir() {
            return var_path.join("events.jsonl");
        }
        // Fallback to home directory
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(".anna").join("events.jsonl")
    }

    /// Append an event record
    pub fn append(&self, record: &EventRecord) -> std::io::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let line = serde_json::to_string(record)?;
        writeln!(file, "{}", line)?;

        // Check if rotation is needed
        self.maybe_rotate()?;

        Ok(())
    }

    /// Read all events
    pub fn read_all(&self) -> std::io::Result<Vec<EventRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(record) = serde_json::from_str::<EventRecord>(&line) {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Read events from the last N days
    pub fn read_recent(&self, days: u64) -> std::io::Result<Vec<EventRecord>> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            .saturating_sub(days * 86400);

        let all = self.read_all()?;
        Ok(all.into_iter().filter(|r| r.timestamp >= cutoff).collect())
    }

    /// Rotate log if it exceeds max entries
    fn maybe_rotate(&self) -> std::io::Result<()> {
        let records = self.read_all()?;
        if records.len() <= self.max_entries {
            return Ok(());
        }

        // Keep only the most recent entries
        let keep_count = self.max_entries * 3 / 4; // Keep 75% after rotation
        let to_keep = &records[records.len() - keep_count..];

        // Write to temp file then rename (atomic)
        let temp_path = self.path.with_extension("jsonl.tmp");
        {
            let mut file = File::create(&temp_path)?;
            for record in to_keep {
                let line = serde_json::to_string(record)?;
                writeln!(file, "{}", line)?;
            }
        }
        fs::rename(&temp_path, &self.path)?;

        Ok(())
    }

    /// Get aggregated stats from events
    pub fn aggregate(&self) -> std::io::Result<AggregatedEvents> {
        let records = self.read_all()?;
        Ok(AggregatedEvents::from_records(&records))
    }

    /// Clear all events (v0.0.404: for reset functionality)
    pub fn clear(&self) -> std::io::Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

/// Clear the default event log (v0.0.404: convenience function for reset)
pub fn clear_event_log() -> std::io::Result<()> {
    let log = EventLog::new(EventLog::default_path(), 1000);
    log.clear()
}
