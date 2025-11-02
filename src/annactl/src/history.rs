//! History tracking for Anna v0.13.2 "Orion II"
//!
//! Store radar snapshots over time and compute trends

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY_ENTRIES: usize = 90;

/// Historical radar entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: u64,
    pub hardware_score: u8,
    pub software_score: u8,
    pub user_score: u8,
    pub overall_score: u8,
    pub top_recommendations: Vec<String>,
}

/// Trend summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSummary {
    pub direction: String,      // "improving", "stable", "declining"
    pub change_7d: i8,           // Score delta over 7 days (-10 to +10)
    pub change_30d: i8,          // Score delta over 30 days
    pub oldest_date: Option<u64>,
}

/// History manager
pub struct HistoryManager {
    history_path: PathBuf,
}

impl HistoryManager {
    /// Create new history manager
    pub fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)?;

        let history_path = state_dir.join("history.jsonl");

        Ok(Self { history_path })
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }

    /// Record new radar snapshot
    pub fn record(&self, entry: HistoryEntry) -> Result<()> {
        // Read existing entries
        let mut entries = self.load_all()?;

        // Append new entry
        entries.push(entry);

        // Trim to MAX_HISTORY_ENTRIES
        if entries.len() > MAX_HISTORY_ENTRIES {
            entries.drain(0..(entries.len() - MAX_HISTORY_ENTRIES));
        }

        // Write back
        self.save_all(&entries)?;

        Ok(())
    }

    /// Load all history entries
    pub fn load_all(&self) -> Result<Vec<HistoryEntry>> {
        if !self.history_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.history_path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<HistoryEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse history entry: {}", e);
                    continue;
                }
            }
        }

        Ok(entries)
    }

    /// Save all history entries
    fn save_all(&self, entries: &[HistoryEntry]) -> Result<()> {
        let mut lines = Vec::new();

        for entry in entries {
            let json = serde_json::to_string(entry)?;
            lines.push(json);
        }

        let content = lines.join("\n") + "\n";
        fs::write(&self.history_path, content)?;

        Ok(())
    }

    /// Compute trends from history
    pub fn compute_trends(&self) -> Result<Option<TrendSummary>> {
        let entries = self.load_all()?;

        if entries.len() < 2 {
            return Ok(None);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Current score
        let current = entries.last().unwrap();
        let current_score = current.overall_score as i16;

        // 7-day average
        let seven_days_ago = now.saturating_sub(7 * 86400);
        let entries_7d: Vec<&HistoryEntry> = entries
            .iter()
            .filter(|e| e.timestamp >= seven_days_ago)
            .collect();

        let change_7d = if entries_7d.len() >= 2 {
            let oldest_7d = entries_7d.first().unwrap().overall_score as i16;
            (current_score - oldest_7d) as i8
        } else {
            0
        };

        // 30-day average
        let thirty_days_ago = now.saturating_sub(30 * 86400);
        let entries_30d: Vec<&HistoryEntry> = entries
            .iter()
            .filter(|e| e.timestamp >= thirty_days_ago)
            .collect();

        let change_30d = if entries_30d.len() >= 2 {
            let oldest_30d = entries_30d.first().unwrap().overall_score as i16;
            (current_score - oldest_30d) as i8
        } else {
            0
        };

        // Determine direction
        let direction = if change_7d >= 2 || change_30d >= 3 {
            "improving".to_string()
        } else if change_7d <= -2 || change_30d <= -3 {
            "declining".to_string()
        } else {
            "stable".to_string()
        };

        Ok(Some(TrendSummary {
            direction,
            change_7d,
            change_30d,
            oldest_date: entries.first().map(|e| e.timestamp),
        }))
    }

    /// Get recent entries (last N)
    pub fn get_recent(&self, count: usize) -> Result<Vec<HistoryEntry>> {
        let mut entries = self.load_all()?;

        if entries.len() > count {
            entries.drain(0..(entries.len() - count));
        }

        Ok(entries)
    }

    /// Clear all history
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        if self.history_path.exists() {
            fs::remove_file(&self.history_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry_serialization() {
        let entry = HistoryEntry {
            timestamp: 1699000000,
            hardware_score: 8,
            software_score: 7,
            user_score: 6,
            overall_score: 7,
            top_recommendations: vec![
                "Update packages".to_string(),
                "Clean cache".to_string(),
            ],
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: HistoryEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.timestamp, entry.timestamp);
        assert_eq!(parsed.hardware_score, entry.hardware_score);
        assert_eq!(parsed.top_recommendations.len(), 2);
    }

    #[test]
    fn test_trend_direction_logic() {
        // Test improving
        assert_eq!(trend_direction(3, 5), "improving");

        // Test declining
        assert_eq!(trend_direction(-3, -5), "declining");

        // Test stable
        assert_eq!(trend_direction(1, 1), "stable");
        assert_eq!(trend_direction(0, 0), "stable");
    }

    fn trend_direction(change_7d: i8, change_30d: i8) -> &'static str {
        if change_7d >= 2 || change_30d >= 3 {
            "improving"
        } else if change_7d <= -2 || change_30d <= -3 {
            "declining"
        } else {
            "stable"
        }
    }

    #[test]
    fn test_rolling_window() {
        let mut entries: Vec<u8> = (1..=100).collect();

        // Simulate rolling window
        if entries.len() > 90 {
            entries.drain(0..(entries.len() - 90));
        }

        assert_eq!(entries.len(), 90);
        assert_eq!(entries[0], 11); // First entry is now 11
        assert_eq!(entries[89], 100); // Last entry is 100
    }

    #[test]
    fn test_time_window_filtering() {
        let now = 1700000000u64;
        let seven_days = 7 * 86400;

        let timestamps = vec![
            now - 10 * 86400, // 10 days ago
            now - 5 * 86400,  // 5 days ago
            now - 1 * 86400,  // 1 day ago
            now,              // now
        ];

        let recent: Vec<u64> = timestamps
            .iter()
            .copied()
            .filter(|&t| t >= now - seven_days)
            .collect();

        assert_eq!(recent.len(), 3); // Last 3 entries within 7 days
    }
}
