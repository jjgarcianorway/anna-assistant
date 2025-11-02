//! Event logging system for Anna v0.12.9 "Orion"
//!
//! In-memory ring buffer with JSONL persistence
//! - 1,000 entries in memory
//! - 5 files × 5 MB rotation at ~/.local/state/anna/events.jsonl

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const MAX_MEMORY_ENTRIES: usize = 1000;
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024; // 5 MB
const MAX_FILES: usize = 5;

/// Event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Event {
    Error {
        code: i32,
        msg: String,
        #[serde(default)]
        timestamp: i64,
    },
    Warning {
        msg: String,
        #[serde(default)]
        timestamp: i64,
    },
    Change {
        key: String,
        old: serde_json::Value,
        new: serde_json::Value,
        #[serde(default)]
        timestamp: i64,
    },
    Advice {
        key: String,
        msg: String,
        #[serde(default)]
        timestamp: i64,
    },
}

impl Event {
    /// Get event timestamp
    pub fn timestamp(&self) -> i64 {
        match self {
            Event::Error { timestamp, .. } => *timestamp,
            Event::Warning { timestamp, .. } => *timestamp,
            Event::Change { timestamp, .. } => *timestamp,
            Event::Advice { timestamp, .. } => *timestamp,
        }
    }

    /// Get event type as string
    pub fn event_type(&self) -> &str {
        match self {
            Event::Error { .. } => "error",
            Event::Warning { .. } => "warning",
            Event::Change { .. } => "change",
            Event::Advice { .. } => "advice",
        }
    }

    /// Create error event
    pub fn error(code: i32, msg: impl Into<String>) -> Self {
        Event::Error {
            code,
            msg: msg.into(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create warning event
    pub fn warning(msg: impl Into<String>) -> Self {
        Event::Warning {
            msg: msg.into(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create change event
    pub fn change(key: impl Into<String>, old: serde_json::Value, new: serde_json::Value) -> Self {
        Event::Change {
            key: key.into(),
            old,
            new,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create advice event
    pub fn advice(key: impl Into<String>, msg: impl Into<String>) -> Self {
        Event::Advice {
            key: key.into(),
            msg: msg.into(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Event filter criteria
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub event_type: Option<String>,
    pub since_timestamp: Option<i64>,
    pub limit: Option<usize>,
}

/// Event logger with ring buffer and JSONL persistence
pub struct EventLogger {
    buffer: Arc<Mutex<VecDeque<Event>>>,
    log_path: PathBuf,
}

impl EventLogger {
    /// Create new event logger
    pub fn new(log_dir: Option<PathBuf>) -> Result<Self> {
        let log_path = if let Some(dir) = log_dir {
            dir.join("events.jsonl")
        } else {
            // Default: ~/.local/state/anna/events.jsonl
            let home = std::env::var("HOME").context("HOME not set")?;
            let state_dir = Path::new(&home).join(".local/state/anna");
            fs::create_dir_all(&state_dir)?;
            state_dir.join("events.jsonl")
        };

        // Load existing events from disk
        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_MEMORY_ENTRIES)));
        let mut logger = EventLogger { buffer, log_path };

        logger.load_from_disk()?;

        Ok(logger)
    }

    /// Log an event
    pub fn log(&self, event: Event) -> Result<()> {
        // Add to in-memory buffer
        {
            let mut buf = self.buffer.lock().unwrap();
            if buf.len() >= MAX_MEMORY_ENTRIES {
                buf.pop_front(); // Remove oldest
            }
            buf.push_back(event.clone());
        }

        // Append to disk
        self.append_to_disk(&event)?;

        // Check if rotation needed
        self.rotate_if_needed()?;

        Ok(())
    }

    /// Get events matching filter
    pub fn get_events(&self, filter: &EventFilter) -> Vec<Event> {
        let buf = self.buffer.lock().unwrap();

        buf.iter()
            .filter(|event| {
                // Filter by type
                if let Some(ref t) = filter.event_type {
                    if event.event_type() != t {
                        return false;
                    }
                }

                // Filter by timestamp
                if let Some(since) = filter.since_timestamp {
                    if event.timestamp() < since {
                        return false;
                    }
                }

                true
            })
            .rev() // Most recent first
            .take(filter.limit.unwrap_or(usize::MAX))
            .cloned()
            .collect()
    }

    /// Get event count by type
    pub fn count_by_type(&self) -> EventCounts {
        let buf = self.buffer.lock().unwrap();

        let mut counts = EventCounts::default();
        for event in buf.iter() {
            match event {
                Event::Error { .. } => counts.errors += 1,
                Event::Warning { .. } => counts.warnings += 1,
                Event::Change { .. } => counts.changes += 1,
                Event::Advice { .. } => counts.advice += 1,
            }
        }

        counts
    }

    /// Append event to disk
    fn append_to_disk(&self, event: &Event) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let json = serde_json::to_string(event)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Load events from disk into memory
    fn load_from_disk(&mut self) -> Result<()> {
        if !self.log_path.exists() {
            return Ok(());
        }

        let file = File::open(&self.log_path)?;
        let reader = BufReader::new(file);

        let mut events = VecDeque::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(event) = serde_json::from_str::<Event>(&line) {
                    events.push_back(event);
                }
            }
        }

        // Keep only last MAX_MEMORY_ENTRIES
        if events.len() > MAX_MEMORY_ENTRIES {
            events.drain(0..events.len() - MAX_MEMORY_ENTRIES);
        }

        *self.buffer.lock().unwrap() = events;

        Ok(())
    }

    /// Rotate log files if needed
    fn rotate_if_needed(&self) -> Result<()> {
        let metadata = fs::metadata(&self.log_path)?;

        if metadata.len() < MAX_FILE_SIZE {
            return Ok(());
        }

        // Rotate files: events.jsonl.4 -> delete, .3 -> .4, .2 -> .3, .1 -> .2, current -> .1
        for i in (1..MAX_FILES).rev() {
            let old_path = if i == 1 {
                self.log_path.clone()
            } else {
                self.log_path.with_extension(format!("jsonl.{}", i - 1))
            };

            let new_path = self.log_path.with_extension(format!("jsonl.{}", i));

            if old_path.exists() {
                if i == MAX_FILES - 1 {
                    // Delete oldest
                    fs::remove_file(&old_path)?;
                } else {
                    // Rename
                    fs::rename(&old_path, &new_path)?;
                }
            }
        }

        // Create new current file
        File::create(&self.log_path)?;

        Ok(())
    }
}

/// Event counts by type
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EventCounts {
    pub errors: usize,
    pub warnings: usize,
    pub changes: usize,
    pub advice: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_event_creation() {
        let event = Event::error(500, "Test error");
        assert_eq!(event.event_type(), "error");

        let event = Event::warning("Test warning");
        assert_eq!(event.event_type(), "warning");

        let event = Event::change("key", serde_json::json!("old"), serde_json::json!("new"));
        assert_eq!(event.event_type(), "change");

        let event = Event::advice("key", "Test advice");
        assert_eq!(event.event_type(), "advice");
    }

    #[test]
    fn test_event_logger_basic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let logger = EventLogger::new(Some(temp_dir.path().to_path_buf()))?;

        logger.log(Event::error(500, "Error 1"))?;
        logger.log(Event::warning("Warning 1"))?;
        logger.log(Event::change("key", serde_json::json!(1), serde_json::json!(2)))?;

        let counts = logger.count_by_type();
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 1);
        assert_eq!(counts.changes, 1);

        Ok(())
    }

    #[test]
    fn test_event_filter() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let logger = EventLogger::new(Some(temp_dir.path().to_path_buf()))?;

        logger.log(Event::error(500, "Error 1"))?;
        logger.log(Event::warning("Warning 1"))?;
        logger.log(Event::error(501, "Error 2"))?;

        let filter = EventFilter {
            event_type: Some("error".to_string()),
            since_timestamp: None,
            limit: None,
        };

        let events = logger.get_events(&filter);
        assert_eq!(events.len(), 2);

        Ok(())
    }

    #[test]
    fn test_ring_buffer_overflow() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let logger = EventLogger::new(Some(temp_dir.path().to_path_buf()))?;

        // Log more than MAX_MEMORY_ENTRIES
        for i in 0..1100 {
            logger.log(Event::warning(format!("Warning {}", i)))?;
        }

        let all_events = logger.get_events(&EventFilter::default());
        assert_eq!(all_events.len(), MAX_MEMORY_ENTRIES);

        Ok(())
    }
}
