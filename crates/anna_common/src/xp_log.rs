//! XP Log Module v0.84.0
//!
//! Persistent storage for XP events with 24-hour window metrics.
//! Stores events as JSONL format for efficient append-only writes.

use crate::xp_events::{XpEvent, XpEventType};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Default XP log directory
pub const XP_LOG_DIR: &str = "/var/lib/anna/knowledge/stats";

/// XP log file name
pub const XP_LOG_FILE: &str = "xp_events.jsonl";

/// Stored XP event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredXpEvent {
    /// ISO timestamp
    pub timestamp: String,
    /// Event type name
    pub event_type: String,
    /// XP change (positive or negative)
    pub xp_change: i32,
    /// Question that triggered this event
    pub question: String,
    /// Command involved (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl From<&XpEvent> for StoredXpEvent {
    fn from(event: &XpEvent) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            event_type: format!("{:?}", event.event_type),
            xp_change: event.xp_change,
            question: event.question.clone(),
            command: event.command.clone(),
            context: event.context.clone(),
        }
    }
}

/// 24-hour metrics summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics24h {
    /// Total XP gained (positive events)
    pub xp_gained: i32,
    /// Total XP lost (negative events)
    pub xp_lost: i32,
    /// Net XP change
    pub net_xp: i32,
    /// Number of positive events
    pub positive_events: u32,
    /// Number of negative events
    pub negative_events: u32,
    /// Total events
    pub total_events: u32,
    /// Most common positive event type
    pub top_positive: Option<String>,
    /// Most common negative event type
    pub top_negative: Option<String>,
    /// Questions answered
    pub questions_answered: u32,
}

/// XP Log manager
pub struct XpLog {
    /// Path to log file
    path: PathBuf,
}

impl Default for XpLog {
    fn default() -> Self {
        Self::new()
    }
}

impl XpLog {
    /// Create with default path
    pub fn new() -> Self {
        Self {
            path: PathBuf::from(XP_LOG_DIR).join(XP_LOG_FILE),
        }
    }

    /// Create with custom path
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Append an XP event to the log
    pub fn append(&self, event: &XpEvent) -> std::io::Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let stored = StoredXpEvent::from(event);
        let mut line = serde_json::to_string(&stored)?;
        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        file.write_all(line.as_bytes())
    }

    /// Read recent events (up to limit)
    pub fn read_recent(&self, limit: usize) -> Vec<StoredXpEvent> {
        if !self.path.exists() {
            return Vec::new();
        }

        // Read all events and take last N
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        let mut events: Vec<StoredXpEvent> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str(&line).ok())
            .collect();

        // Take last N events (most recent)
        if events.len() > limit {
            events = events.split_off(events.len() - limit);
        }

        // Reverse so most recent first
        events.reverse();
        events
    }

    /// Get 24-hour metrics
    pub fn metrics_24h(&self) -> Metrics24h {
        if !self.path.exists() {
            return Metrics24h::default();
        }

        let cutoff = Utc::now() - Duration::hours(24);

        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(_) => return Metrics24h::default(),
        };

        let reader = BufReader::new(file);
        let mut metrics = Metrics24h::default();
        let mut positive_counts: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        let mut negative_counts: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        let mut seen_questions: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for line in reader.lines().flatten() {
            if let Ok(event) = serde_json::from_str::<StoredXpEvent>(&line) {
                // Parse timestamp and check if within 24h
                if let Ok(ts) = DateTime::parse_from_rfc3339(&event.timestamp) {
                    if ts.with_timezone(&Utc) >= cutoff {
                        metrics.total_events += 1;
                        seen_questions.insert(event.question.clone());

                        if event.xp_change >= 0 {
                            metrics.xp_gained += event.xp_change;
                            metrics.positive_events += 1;
                            *positive_counts.entry(event.event_type.clone()).or_insert(0) += 1;
                        } else {
                            metrics.xp_lost += event.xp_change.abs();
                            metrics.negative_events += 1;
                            *negative_counts.entry(event.event_type.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        metrics.net_xp = metrics.xp_gained - metrics.xp_lost;
        metrics.questions_answered = seen_questions.len() as u32;

        // Find most common events
        metrics.top_positive = positive_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name);

        metrics.top_negative = negative_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name);

        metrics
    }

    /// Clean old events (older than 7 days)
    pub fn cleanup_old(&self) -> std::io::Result<u32> {
        if !self.path.exists() {
            return Ok(0);
        }

        let cutoff = Utc::now() - Duration::days(7);
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        let mut kept: Vec<String> = Vec::new();
        let mut removed: u32 = 0;

        for line in reader.lines().flatten() {
            if let Ok(event) = serde_json::from_str::<StoredXpEvent>(&line) {
                if let Ok(ts) = DateTime::parse_from_rfc3339(&event.timestamp) {
                    if ts.with_timezone(&Utc) >= cutoff {
                        kept.push(line);
                    } else {
                        removed += 1;
                    }
                }
            }
        }

        // Rewrite file with kept events
        let mut file = File::create(&self.path)?;
        for line in kept {
            writeln!(file, "{}", line)?;
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_stored_xp_event_from() {
        let event = XpEvent::new(XpEventType::BrainSelfSolve, "test question")
            .with_command("lscpu");

        let stored = StoredXpEvent::from(&event);
        assert_eq!(stored.xp_change, 15);
        assert_eq!(stored.question, "test question");
        assert_eq!(stored.command, Some("lscpu".to_string()));
    }

    #[test]
    fn test_xp_log_append_and_read() {
        let temp_file = NamedTempFile::new().unwrap();
        let log = XpLog::with_path(temp_file.path().to_path_buf());

        let event1 = XpEvent::new(XpEventType::BrainSelfSolve, "q1");
        let event2 = XpEvent::new(XpEventType::JuniorBadCommand, "q2");

        log.append(&event1).unwrap();
        log.append(&event2).unwrap();

        let events = log.read_recent(10);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].question, "q2"); // Most recent first
        assert_eq!(events[1].question, "q1");
    }

    #[test]
    fn test_metrics_24h() {
        let temp_file = NamedTempFile::new().unwrap();
        let log = XpLog::with_path(temp_file.path().to_path_buf());

        // Add some events
        log.append(&XpEvent::new(XpEventType::BrainSelfSolve, "q1")).unwrap();
        log.append(&XpEvent::new(XpEventType::SeniorGreenApproval, "q2")).unwrap();
        log.append(&XpEvent::new(XpEventType::JuniorBadCommand, "q3")).unwrap();

        let metrics = log.metrics_24h();
        assert_eq!(metrics.positive_events, 2);
        assert_eq!(metrics.negative_events, 1);
        assert_eq!(metrics.xp_gained, 15 + 12); // BrainSelfSolve + SeniorGreenApproval
        assert_eq!(metrics.xp_lost, 8); // JuniorBadCommand
        assert_eq!(metrics.questions_answered, 3);
    }

    #[test]
    fn test_read_recent_limit() {
        let temp_file = NamedTempFile::new().unwrap();
        let log = XpLog::with_path(temp_file.path().to_path_buf());

        for i in 0..10 {
            log.append(&XpEvent::new(XpEventType::BrainSelfSolve, &format!("q{}", i)))
                .unwrap();
        }

        let events = log.read_recent(3);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].question, "q9"); // Most recent
        assert_eq!(events[2].question, "q7");
    }
}
