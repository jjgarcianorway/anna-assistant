//! System-wide staff statistics
//!
//! Manages aggregate statistics across all staff members,
//! including persistence to disk.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::levels::xp_to_level;
use super::metrics::StaffMetrics;

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

    /// v0.0.306: Clear all staff stats (for reset)
    pub fn clear() -> std::io::Result<()> {
        let path = Self::stats_path();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
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

    /// v0.0.301: Get staff grouped by department
    pub fn by_department(&self) -> HashMap<String, Vec<(&String, &StaffMetrics)>> {
        let mut departments: HashMap<String, Vec<(&String, &StaffMetrics)>> = HashMap::new();

        for (person_id, metrics) in &self.by_staff {
            let dept = person_id.split('_').next().unwrap_or("unknown").to_string();
            departments
                .entry(dept)
                .or_default()
                .push((person_id, metrics));
        }

        // Sort each department by tickets handled (descending)
        for staff in departments.values_mut() {
            staff.sort_by(|a, b| b.1.tickets_handled.cmp(&a.1.tickets_handled));
        }

        departments
    }

    /// v0.0.301: Get total resolved tickets across all staff
    pub fn total_resolved(&self) -> u32 {
        self.by_staff.values().map(|m| m.tickets_resolved).sum()
    }

    /// v0.0.301: Get total escalated tickets across all staff
    pub fn total_escalated(&self) -> u32 {
        self.by_staff.values().map(|m| m.tickets_escalated).sum()
    }

    /// v0.0.317: Apply user feedback to staff XP
    /// Helpful feedback = +5 XP bonus, NotHelpful = -10 XP penalty
    pub fn apply_feedback(&mut self, person_id: &str, helpful: bool) -> Option<FeedbackResult> {
        let metrics = self.by_staff.get_mut(person_id)?;
        let old_xp = metrics.xp;
        let old_level = metrics.level;

        if helpful {
            // User liked the answer - bonus XP
            metrics.xp += 5;
        } else {
            // User didn't like - penalty
            metrics.xp = metrics.xp.saturating_sub(10);
        }
        metrics.level = xp_to_level(metrics.xp);

        self.updated_at = chrono::Utc::now().timestamp();

        Some(FeedbackResult {
            person_id: person_id.to_string(),
            old_xp,
            new_xp: metrics.xp,
            old_level,
            new_level: metrics.level,
            helpful,
        })
    }
}

/// v0.0.317: Result of applying user feedback to staff
#[derive(Debug, Clone)]
pub struct FeedbackResult {
    pub person_id: String,
    pub old_xp: u64,
    pub new_xp: u64,
    pub old_level: u8,
    pub new_level: u8,
    pub helpful: bool,
}
