//! v0.0.67: Stats store for RPG system.
//!
//! Tracks per-request metrics for the annactl stats command.
//! Uses JSONL for simplicity and robustness.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// A single request record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRecord {
    pub timestamp: DateTime<Utc>,
    pub query_class: String,
    pub route_used: String,
    pub probes_count: usize,
    pub specialist_used: Option<String>,
    pub reliability: u8,
    pub duration_ms: u64,
    pub evidence_kinds: Vec<String>,
    pub team: String,
    pub success: bool,
}

/// Aggregated stats for display
#[derive(Debug, Clone, Default)]
pub struct AggregatedStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub average_reliability: f64,
    pub escalated_count: u64,
    pub recipes_learned: u64,
    pub most_consulted_team: Option<String>,
    pub longest_resolution_ms: u64,
    pub shortest_resolution_ms: u64,
    pub team_counts: Vec<(String, u64)>,
}

impl AggregatedStats {
    /// Calculate XP on a 0-100 scale using logistic curve
    pub fn calculate_xp(&self) -> u8 {
        // XP based on: requests, success rate, avg reliability
        // Use logistic curve: 100 / (1 + e^(-k*(x-x0)))
        let base_score = self.total_requests as f64 * 0.1
            + self.success_rate() * 50.0
            + self.average_reliability * 0.3;

        // Logistic transformation: maps ~0-100 to 0-100 with slow start/end
        let k = 0.1;
        let x0 = 50.0;
        let logistic = 100.0 / (1.0 + (-k * (base_score - x0)).exp());

        logistic.min(100.0).max(0.0) as u8
    }

    /// Get title based on XP bucket
    pub fn xp_title(&self) -> &'static str {
        let xp = self.calculate_xp();
        match xp {
            0..=10 => "Apprentice Troubleshooter",
            11..=25 => "Junior IT Support",
            26..=40 => "System Whisperer",
            41..=55 => "Kernel Confidant",
            56..=70 => "Senior Systems Analyst",
            71..=85 => "Infrastructure Sage",
            86..=95 => "Principal Architect",
            96..=100 => "Grandmaster of Uptime",
            _ => "Unknown",
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }
}

/// Stats store backed by JSONL file
pub struct StatsStore {
    path: PathBuf,
}

impl StatsStore {
    /// Create or open stats store
    pub fn new(data_dir: &str) -> Self {
        let path = PathBuf::from(data_dir).join("stats.jsonl");
        Self { path }
    }

    /// Create store at default location
    pub fn default_location() -> Self {
        Self::new("/var/lib/anna")
    }

    /// Record a request
    pub fn record(&self, record: &RequestRecord) -> std::io::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Append to file with fsync
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let line = serde_json::to_string(record)?;
        writeln!(file, "{}", line)?;
        file.sync_all()?;

        Ok(())
    }

    /// Read all records
    pub fn read_all(&self) -> std::io::Result<Vec<RequestRecord>> {
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
            match serde_json::from_str(&line) {
                Ok(record) => records.push(record),
                Err(e) => {
                    // Skip malformed lines (forward compatibility)
                    eprintln!("Warning: skipping malformed stats line: {}", e);
                }
            }
        }

        Ok(records)
    }

    /// Aggregate all stats
    pub fn aggregate(&self) -> std::io::Result<AggregatedStats> {
        let records = self.read_all()?;

        if records.is_empty() {
            return Ok(AggregatedStats::default());
        }

        let total = records.len() as u64;
        let successful = records.iter().filter(|r| r.success).count() as u64;
        let avg_rel = records.iter().map(|r| r.reliability as f64).sum::<f64>() / total as f64;
        let escalated = records.iter().filter(|r| r.specialist_used.is_some()).count() as u64;

        let longest = records.iter().map(|r| r.duration_ms).max().unwrap_or(0);
        let shortest = records.iter().map(|r| r.duration_ms).min().unwrap_or(0);

        // Count teams
        let mut team_map = std::collections::HashMap::new();
        for r in &records {
            *team_map.entry(r.team.clone()).or_insert(0u64) += 1;
        }
        let mut team_counts: Vec<_> = team_map.into_iter().collect();
        team_counts.sort_by(|a, b| b.1.cmp(&a.1));

        let most_consulted = team_counts.first().map(|(t, _)| t.clone());

        Ok(AggregatedStats {
            total_requests: total,
            successful_requests: successful,
            average_reliability: avg_rel,
            escalated_count: escalated,
            recipes_learned: 0, // Placeholder until recipe system
            most_consulted_team: most_consulted,
            longest_resolution_ms: longest,
            shortest_resolution_ms: shortest,
            team_counts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_stats_store_roundtrip() {
        let dir = tempdir().unwrap();
        let store = StatsStore::new(dir.path().to_str().unwrap());

        let record = RequestRecord {
            timestamp: Utc::now(),
            query_class: "memory_usage".to_string(),
            route_used: "deterministic".to_string(),
            probes_count: 1,
            specialist_used: None,
            reliability: 85,
            duration_ms: 150,
            evidence_kinds: vec!["memory".to_string()],
            team: "system".to_string(),
            success: true,
        };

        store.record(&record).unwrap();
        let records = store.read_all().unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].query_class, "memory_usage");
        assert_eq!(records[0].reliability, 85);
    }

    #[test]
    fn test_aggregate_stats() {
        let dir = tempdir().unwrap();
        let store = StatsStore::new(dir.path().to_str().unwrap());

        // Add some records
        for i in 0..5 {
            let record = RequestRecord {
                timestamp: Utc::now(),
                query_class: "test".to_string(),
                route_used: "deterministic".to_string(),
                probes_count: 1,
                specialist_used: if i % 2 == 0 { Some("system".to_string()) } else { None },
                reliability: 80 + i * 2,
                duration_ms: 100 + i as u64 * 50,
                evidence_kinds: vec![],
                team: "system".to_string(),
                success: i < 4,
            };
            store.record(&record).unwrap();
        }

        let agg = store.aggregate().unwrap();
        assert_eq!(agg.total_requests, 5);
        assert_eq!(agg.successful_requests, 4);
        assert_eq!(agg.escalated_count, 3);
        assert_eq!(agg.most_consulted_team, Some("system".to_string()));
    }

    #[test]
    fn test_xp_calculation() {
        let mut stats = AggregatedStats::default();
        assert_eq!(stats.calculate_xp(), 0);

        stats.total_requests = 100;
        stats.successful_requests = 90;
        stats.average_reliability = 85.0;
        let xp = stats.calculate_xp();
        assert!(xp > 50, "XP should be > 50 for good stats, got {}", xp);
    }

    #[test]
    fn test_xp_titles() {
        let mut stats = AggregatedStats::default();
        assert_eq!(stats.xp_title(), "Apprentice Troubleshooter");

        stats.total_requests = 1000;
        stats.successful_requests = 950;
        stats.average_reliability = 90.0;
        // Should give high XP
        assert!(stats.calculate_xp() > 70);
    }
}
