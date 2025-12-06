//! Staff performance statistics for Service Desk Theatre.
//!
//! v0.0.107: Tracks per-staff metrics like tickets handled, success rates.
//!
//! Storage: /etc/anna/staff_stats.json (system-wide)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Per-staff performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StaffMetrics {
    /// Total tickets handled
    pub tickets_handled: u32,
    /// Successfully resolved tickets
    pub tickets_resolved: u32,
    /// Escalated tickets (for juniors)
    pub tickets_escalated: u32,
    /// Average reliability score
    pub avg_reliability: f32,
    /// Total handling time (ms)
    pub total_time_ms: u64,
    /// Last activity timestamp
    pub last_active: i64,
}

impl StaffMetrics {
    /// Record a ticket completion
    pub fn record_ticket(&mut self, resolved: bool, escalated: bool, reliability: u8, duration_ms: u64) {
        let old_total = self.tickets_handled;
        self.tickets_handled += 1;

        if resolved {
            self.tickets_resolved += 1;
        }
        if escalated {
            self.tickets_escalated += 1;
        }

        // Update rolling average
        if old_total == 0 {
            self.avg_reliability = reliability as f32;
        } else {
            self.avg_reliability = (self.avg_reliability * old_total as f32 + reliability as f32)
                / self.tickets_handled as f32;
        }

        self.total_time_ms += duration_ms;
        self.last_active = chrono::Utc::now().timestamp();
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f32 {
        if self.tickets_handled == 0 {
            0.0
        } else {
            (self.tickets_resolved as f32 / self.tickets_handled as f32) * 100.0
        }
    }

    /// Get average handling time in ms
    pub fn avg_time_ms(&self) -> u64 {
        if self.tickets_handled == 0 {
            0
        } else {
            self.total_time_ms / self.tickets_handled as u64
        }
    }
}

/// System-wide staff statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StaffStats {
    /// Per-staff metrics (keyed by person_id like "desktop_jr_sofia")
    pub by_staff: HashMap<String, StaffMetrics>,
    /// Last updated timestamp
    pub updated_at: i64,
}

impl StaffStats {
    /// Get stats path
    fn stats_path() -> PathBuf {
        let etc_anna = PathBuf::from("/etc/anna");
        if etc_anna.exists() && etc_anna.is_dir() {
            etc_anna.join("staff_stats.json")
        } else {
            // Fallback to home dir
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".anna").join("staff_stats.json")
        }
    }

    /// Load stats from disk
    pub fn load() -> Self {
        let path = Self::stats_path();
        if path.exists() {
            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(stats) = serde_json::from_str(&json) {
                    return stats;
                }
            }
        }
        Self::default()
    }

    /// Save stats to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::stats_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    /// Record a ticket for a staff member
    pub fn record_ticket(
        &mut self,
        person_id: &str,
        resolved: bool,
        escalated: bool,
        reliability: u8,
        duration_ms: u64,
    ) {
        let metrics = self.by_staff.entry(person_id.to_string()).or_default();
        metrics.record_ticket(resolved, escalated, reliability, duration_ms);
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Get top performers by tickets resolved
    pub fn top_performers(&self, limit: usize) -> Vec<(&String, &StaffMetrics)> {
        let mut sorted: Vec<_> = self.by_staff.iter().collect();
        sorted.sort_by(|a, b| b.1.tickets_resolved.cmp(&a.1.tickets_resolved));
        sorted.into_iter().take(limit).collect()
    }

    /// Get total tickets handled across all staff
    pub fn total_tickets(&self) -> u32 {
        self.by_staff.values().map(|m| m.tickets_handled).sum()
    }

    /// v0.0.110: Get metrics for a specific staff member
    pub fn get(&self, person_id: &str) -> Option<&StaffMetrics> {
        self.by_staff.get(person_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staff_metrics_record() {
        let mut metrics = StaffMetrics::default();
        metrics.record_ticket(true, false, 85, 1000);
        metrics.record_ticket(true, false, 75, 2000);

        assert_eq!(metrics.tickets_handled, 2);
        assert_eq!(metrics.tickets_resolved, 2);
        assert_eq!(metrics.avg_reliability, 80.0);
        assert_eq!(metrics.avg_time_ms(), 1500);
    }

    #[test]
    fn test_staff_metrics_success_rate() {
        let mut metrics = StaffMetrics::default();
        metrics.record_ticket(true, false, 80, 1000);
        metrics.record_ticket(false, true, 50, 5000);

        assert_eq!(metrics.success_rate(), 50.0);
    }

    #[test]
    fn test_staff_stats_record() {
        let mut stats = StaffStats::default();
        stats.record_ticket("desktop_jr_sofia", true, false, 90, 500);
        stats.record_ticket("desktop_jr_sofia", true, false, 80, 600);
        stats.record_ticket("network_jr_michael", true, false, 85, 700);

        assert_eq!(stats.by_staff.len(), 2);
        assert_eq!(stats.by_staff.get("desktop_jr_sofia").unwrap().tickets_handled, 2);
    }

    #[test]
    fn test_top_performers() {
        let mut stats = StaffStats::default();
        stats.record_ticket("a", true, false, 80, 100);
        stats.record_ticket("b", true, false, 80, 100);
        stats.record_ticket("b", true, false, 80, 100);
        stats.record_ticket("c", true, false, 80, 100);
        stats.record_ticket("c", true, false, 80, 100);
        stats.record_ticket("c", true, false, 80, 100);

        let top = stats.top_performers(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "c");
        assert_eq!(top[1].0, "b");
    }
}
