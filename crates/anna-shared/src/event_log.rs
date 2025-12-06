//! Event log store for stats/RPG system (v0.0.75).
//!
//! Append-only JSONL store for request events with rotation.

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Single event record for the log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    /// Unique request ID
    pub request_id: String,
    /// Unix timestamp (seconds)
    pub timestamp: u64,
    /// Query class (e.g., "memory_usage", "configure_editor")
    pub query_class: String,
    /// Outcome: "verified", "failed", "timeout", "clarification"
    pub outcome: String,
    /// Reliability score (0-100)
    pub reliability: u8,
    /// Team that handled the request
    pub team: String,
    /// Whether escalation occurred
    pub escalated: bool,
    /// Escalation tier reached (0=none, 1=junior, 2=senior, 3=manager)
    pub escalation_tier: u8,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Number of interactions (clarifications, retries)
    pub interactions: u32,
    /// Recipe ID if one was used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_used: Option<String>,
    /// Recipe ID if one was learned
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_learned: Option<String>,
}

impl EventRecord {
    pub fn new(request_id: &str, query_class: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            query_class: query_class.to_string(),
            outcome: "pending".to_string(),
            reliability: 0,
            team: "unknown".to_string(),
            escalated: false,
            escalation_tier: 0,
            duration_ms: 0,
            interactions: 0,
            recipe_used: None,
            recipe_learned: None,
        }
    }

    /// Mark as verified
    pub fn verified(mut self, reliability: u8) -> Self {
        self.outcome = "verified".to_string();
        self.reliability = reliability;
        self
    }

    /// Mark as failed
    pub fn failed(mut self) -> Self {
        self.outcome = "failed".to_string();
        self
    }

    /// Mark as timeout
    pub fn timeout(mut self) -> Self {
        self.outcome = "timeout".to_string();
        self
    }

    /// Set team
    pub fn with_team(mut self, team: &str) -> Self {
        self.team = team.to_string();
        self
    }

    /// Set escalation info
    pub fn with_escalation(mut self, tier: u8) -> Self {
        self.escalated = tier > 0;
        self.escalation_tier = tier;
        self
    }

    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

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
    pub fn default_path() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/lib/anna/events.jsonl")
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
}

/// Aggregated statistics from event records
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedEvents {
    /// Total requests
    pub total_requests: u64,
    /// First event timestamp (installation date proxy)
    pub first_event_ts: u64,
    /// Last event timestamp
    pub last_event_ts: u64,
    /// Successful (verified) requests
    pub verified_count: u64,
    /// Failed requests
    pub failed_count: u64,
    /// Timeout requests
    pub timeout_count: u64,
    /// Clarification requests
    pub clarification_count: u64,
    /// Total escalations
    pub escalation_count: u64,
    /// Average reliability score
    pub avg_reliability: f32,
    /// Average duration (ms)
    pub avg_duration_ms: f64,
    /// Min duration (ms)
    pub min_duration_ms: u64,
    /// Max duration (ms)
    pub max_duration_ms: u64,
    /// Requests by team
    pub by_team: std::collections::HashMap<String, u64>,
    /// Most escalated team
    pub most_escalated_team: Option<String>,
    /// Recipes used count
    pub recipes_used: u64,
    /// Recipes learned count
    pub recipes_learned: u64,
    /// XP (computed)
    pub xp: u64,
    /// Level (computed)
    pub level: u32,
    /// Title (computed)
    pub title: String,
}

impl AggregatedEvents {
    pub fn from_records(records: &[EventRecord]) -> Self {
        let mut agg = Self::default();

        if records.is_empty() {
            agg.title = "Apprentice Troubleshooter".to_string();
            return agg;
        }

        agg.total_requests = records.len() as u64;
        agg.min_duration_ms = u64::MAX;
        agg.first_event_ts = u64::MAX;
        agg.last_event_ts = 0;

        let mut total_reliability: u64 = 0;
        let mut total_duration: u64 = 0;
        let mut escalations_by_team: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();

        for record in records {
            // Track first and last timestamps
            agg.first_event_ts = agg.first_event_ts.min(record.timestamp);
            agg.last_event_ts = agg.last_event_ts.max(record.timestamp);
            // Count outcomes
            match record.outcome.as_str() {
                "verified" => agg.verified_count += 1,
                "failed" => agg.failed_count += 1,
                "timeout" => agg.timeout_count += 1,
                "clarification" => agg.clarification_count += 1,
                _ => {}
            }

            // Escalations
            if record.escalated {
                agg.escalation_count += 1;
                *escalations_by_team.entry(record.team.clone()).or_insert(0) += 1;
            }

            // Reliability
            total_reliability += record.reliability as u64;

            // Duration
            total_duration += record.duration_ms;
            agg.min_duration_ms = agg.min_duration_ms.min(record.duration_ms);
            agg.max_duration_ms = agg.max_duration_ms.max(record.duration_ms);

            // By team
            *agg.by_team.entry(record.team.clone()).or_insert(0) += 1;

            // Recipes
            if record.recipe_used.is_some() {
                agg.recipes_used += 1;
            }
            if record.recipe_learned.is_some() {
                agg.recipes_learned += 1;
            }
        }

        // Averages
        agg.avg_reliability = total_reliability as f32 / agg.total_requests as f32;
        agg.avg_duration_ms = total_duration as f64 / agg.total_requests as f64;

        // Fix min values if needed
        if agg.min_duration_ms == u64::MAX {
            agg.min_duration_ms = 0;
        }
        if agg.first_event_ts == u64::MAX {
            agg.first_event_ts = 0;
        }

        // Most escalated team
        agg.most_escalated_team = escalations_by_team
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(team, _)| team);

        // Compute XP and level
        agg.compute_xp();

        agg
    }

    /// Compute XP using logistic curve
    fn compute_xp(&mut self) {
        // Base XP from requests
        let request_xp = self.total_requests * 10;

        // Bonus for success rate
        let success_rate = if self.total_requests > 0 {
            self.verified_count as f32 / self.total_requests as f32
        } else {
            0.0
        };
        let success_bonus = (success_rate * 100.0 * self.total_requests as f32) as u64;

        // Bonus for reliability
        let reliability_bonus = (self.avg_reliability * self.total_requests as f32) as u64;

        // Recipe bonuses
        let recipe_bonus = self.recipes_learned * 50 + self.recipes_used * 10;

        self.xp = request_xp + success_bonus + reliability_bonus + recipe_bonus;

        // Level from XP (logistic curve)
        self.level = self.xp_to_level(self.xp);
        self.title = Self::level_title(self.level);
    }

    fn xp_to_level(&self, xp: u64) -> u32 {
        // Logistic-style progression
        match xp {
            0..=99 => 1,
            100..=299 => 2,
            300..=599 => 3,
            600..=999 => 4,
            1000..=1999 => 5,
            2000..=3999 => 6,
            4000..=7999 => 7,
            8000..=15999 => 8,
            16000..=31999 => 9,
            32000..=63999 => 10,
            _ => 11,
        }
    }

    fn level_title(level: u32) -> String {
        match level {
            1 => "Apprentice Troubleshooter",
            2 => "Help Desk Hero",
            3 => "System Sleuth",
            4 => "Diagnostic Detective",
            5 => "Performance Prophet",
            6 => "Infrastructure Sage",
            7 => "Uptime Guardian",
            8 => "Reliability Wizard",
            9 => "System Architect",
            10 => "IT Grandmaster",
            _ => "Grandmaster of Uptime",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_record_builder() {
        let record = EventRecord::new("test-123", "memory_usage")
            .verified(85)
            .with_team("Performance")
            .with_duration(1500);

        assert_eq!(record.outcome, "verified");
        assert_eq!(record.reliability, 85);
        assert_eq!(record.team, "Performance");
        assert_eq!(record.duration_ms, 1500);
    }

    #[test]
    fn test_aggregated_events_empty() {
        let agg = AggregatedEvents::from_records(&[]);
        assert_eq!(agg.total_requests, 0);
        assert_eq!(agg.level, 0); // Will be 1 after compute
        assert_eq!(agg.title, "Apprentice Troubleshooter");
    }

    #[test]
    fn test_aggregated_events_xp_calculation() {
        let records = vec![
            EventRecord::new("1", "memory").verified(90).with_team("Performance"),
            EventRecord::new("2", "disk").verified(85).with_team("Storage"),
            EventRecord::new("3", "network").failed().with_team("Network"),
        ];

        let agg = AggregatedEvents::from_records(&records);
        assert_eq!(agg.total_requests, 3);
        assert_eq!(agg.verified_count, 2);
        assert_eq!(agg.failed_count, 1);
        assert!(agg.xp > 0);
        assert!(agg.level >= 1);
    }

    #[test]
    fn test_xp_to_level_progression() {
        let agg = AggregatedEvents::default();
        assert_eq!(agg.xp_to_level(0), 1);
        assert_eq!(agg.xp_to_level(100), 2);
        assert_eq!(agg.xp_to_level(1000), 5);
        assert_eq!(agg.xp_to_level(100000), 11);
    }
}
