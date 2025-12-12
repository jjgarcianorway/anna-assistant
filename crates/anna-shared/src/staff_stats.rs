//! Staff performance statistics for Service Desk Theatre.
//!
//! v0.0.107: Tracks per-staff metrics like tickets handled, success rates.
//! v0.0.315: XP penalties for poor performance, bonus/penalty in record_ticket.
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
    /// v0.0.301: Staff XP (experience points)
    #[serde(default)]
    pub xp: u64,
    /// v0.0.301: Staff level (1-6)
    #[serde(default = "default_level")]
    pub level: u8,
}

fn default_level() -> u8 {
    1
}

impl StaffMetrics {
    /// Record a ticket completion
    pub fn record_ticket(
        &mut self,
        resolved: bool,
        escalated: bool,
        reliability: u8,
        duration_ms: u64,
    ) {
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

        // v0.0.301: Compute XP and level
        self.update_xp(resolved, reliability);
    }

    /// v0.0.315: Update XP based on ticket outcome (with penalties)
    fn update_xp(&mut self, resolved: bool, reliability: u8) {
        // v0.0.315: XP can increase OR decrease based on performance
        let xp_change: i64;

        if resolved {
            // Base XP for resolved: +10
            let mut bonus: i64 = 10;

            // Bonus for high reliability: +2 per point above 60
            if reliability > 60 {
                bonus += (reliability - 60) as i64 * 2;
            }

            // Extra bonus for excellent work (90+): +15
            if reliability >= 90 {
                bonus += 15;
            }

            xp_change = bonus;
        } else {
            // PENALTY for unresolved tickets
            // Low reliability = bigger penalty (staff should improve)
            if reliability < 40 {
                xp_change = -15; // Significant penalty for poor work
            } else if reliability < 60 {
                xp_change = -5; // Minor penalty
            } else {
                xp_change = 0; // No penalty if reliability was ok but still unresolved
            }
        }

        // Apply XP change (floor at 0)
        if xp_change >= 0 {
            self.xp += xp_change as u64;
        } else {
            let penalty = (-xp_change) as u64;
            self.xp = self.xp.saturating_sub(penalty);
        }

        self.level = xp_to_level(self.xp);
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

/// v0.0.301: Convert XP to level (1-6)
/// Progression is slower - meaningful growth over time
pub fn xp_to_level(xp: u64) -> u8 {
    match xp {
        0..=99 => 1,      // Novice
        100..=299 => 2,   // Apprentice
        300..=699 => 3,   // Competent
        700..=1499 => 4,  // Expert
        1500..=2999 => 5, // Master
        _ => 6,           // Principal
    }
}

/// v0.0.301: Get title for level (for juniors and seniors)
pub fn level_title(level: u8, is_senior: bool) -> &'static str {
    if is_senior {
        match level {
            1..=3 => "Expert",
            4..=5 => "Master",
            _ => "Principal",
        }
    } else {
        match level {
            1 => "Novice",
            2 => "Apprentice",
            3 => "Competent",
            4 => "Skilled",
            5 => "Proficient",
            _ => "Expert",
        }
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
        assert_eq!(
            stats
                .by_staff
                .get("desktop_jr_sofia")
                .unwrap()
                .tickets_handled,
            2
        );
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

    #[test]
    fn test_xp_calculation() {
        let mut metrics = StaffMetrics::default();
        // v0.0.315: Base: 10xp + reliability bonus: (85-60)*2 = 50xp = 60 total
        metrics.record_ticket(true, false, 85, 1000);
        assert_eq!(metrics.xp, 60); // 10 + 50
        assert_eq!(metrics.level, 1); // < 100 = Novice
    }

    #[test]
    fn test_xp_level_progression() {
        let mut metrics = StaffMetrics::default();
        // Simulate 5 high-reliability resolved tickets
        for _ in 0..5 {
            metrics.record_ticket(true, false, 90, 1000);
        }
        // v0.0.315: Each ticket: 10 + (90-60)*2 + 15 = 85 xp, total = 425 xp
        assert_eq!(metrics.xp, 425);
        assert_eq!(metrics.level, 3); // 300-699 = Competent
    }

    #[test]
    fn test_xp_penalty() {
        let mut metrics = StaffMetrics::default();
        // First earn some XP
        metrics.record_ticket(true, false, 80, 1000); // +50 xp (10 + 40)
        assert_eq!(metrics.xp, 50);

        // v0.0.315: Unresolved with low reliability = penalty
        metrics.record_ticket(false, false, 30, 1000); // -15 xp
        assert_eq!(metrics.xp, 35);

        // Unresolved with medium reliability = smaller penalty
        metrics.record_ticket(false, false, 50, 1000); // -5 xp
        assert_eq!(metrics.xp, 30);

        // Unresolved but decent reliability = no penalty
        metrics.record_ticket(false, false, 70, 1000); // 0 xp change
        assert_eq!(metrics.xp, 30);

        // XP can't go below 0
        let mut fresh = StaffMetrics::default();
        fresh.record_ticket(false, false, 20, 1000); // -15 but floor at 0
        assert_eq!(fresh.xp, 0);
    }

    #[test]
    fn test_staff_feedback() {
        let mut stats = StaffStats::default();
        // First create a staff entry via record_ticket
        stats.record_ticket("desktop_jr", true, false, 80, 1000);
        let initial_xp = stats.by_staff.get("desktop_jr").unwrap().xp;

        // Test positive feedback (+5 XP)
        let result = stats.apply_feedback("desktop_jr", true);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.new_xp, initial_xp + 5);
        assert!(r.helpful);

        // Test negative feedback (-10 XP)
        let before = stats.by_staff.get("desktop_jr").unwrap().xp;
        let result = stats.apply_feedback("desktop_jr", false);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.new_xp, before.saturating_sub(10));
        assert!(!r.helpful);

        // Test feedback for non-existent staff returns None
        assert!(stats.apply_feedback("unknown_person", true).is_none());
    }

    #[test]
    fn test_xp_to_level() {
        assert_eq!(xp_to_level(0), 1); // Novice
        assert_eq!(xp_to_level(99), 1); // Novice
        assert_eq!(xp_to_level(100), 2); // Apprentice
        assert_eq!(xp_to_level(299), 2); // Apprentice
        assert_eq!(xp_to_level(300), 3); // Competent
        assert_eq!(xp_to_level(699), 3); // Competent
        assert_eq!(xp_to_level(700), 4); // Expert
        assert_eq!(xp_to_level(1500), 5); // Master
        assert_eq!(xp_to_level(3000), 6); // Principal
    }
}
