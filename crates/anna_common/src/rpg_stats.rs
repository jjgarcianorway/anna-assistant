//! RPG Stats System v0.0.75
//!
//! Implements gamification stats for Anna:
//! - XP and Level with non-linear progression
//! - Titles from Intern to Grandmaster
//! - Request counters (total, successes, failures)
//! - Reliability metrics (average, rolling last-50)
//! - Escalation percentages (junior, doctor, recipe-solved)
//! - By-domain breakdown
//! - Latency metrics (median, p95)
//!
//! Storage: /var/lib/anna/internal/stats.json (or telemetry.db)

use crate::learning::{XpState, XP_GAIN_RECIPE_CREATED, XP_GAIN_SUCCESS_85, XP_GAIN_SUCCESS_90};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Stats storage file
pub const STATS_FILE: &str = "/var/lib/anna/internal/stats.json";

/// Rolling window size for recent reliability
pub const ROLLING_WINDOW_SIZE: usize = 50;

/// Level titles (nerdy, old-school, stable - no emojis)
const LEVEL_TITLES: &[(u8, &str)] = &[
    (0, "Intern"),
    (3, "Apprentice"),
    (6, "Technician"),
    (9, "Analyst"),
    (12, "Engineer"),
    (15, "Senior Engineer"),
    (18, "Architect"),
    (21, "Wizard"),
    (25, "Sage"),
    (30, "Grandmaster"),
];

/// Get title for a level
pub fn title_for_level(level: u8) -> &'static str {
    for (min_level, title) in LEVEL_TITLES.iter().rev() {
        if level >= *min_level {
            return title;
        }
    }
    "Intern"
}

/// Complete RPG stats for Anna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpgStats {
    /// XP and leveling
    pub xp: XpData,
    /// Request counters
    pub requests: RequestCounters,
    /// Reliability metrics
    pub reliability: ReliabilityMetrics,
    /// Escalation tracking
    pub escalations: EscalationMetrics,
    /// By-domain stats
    pub domains: HashMap<String, DomainStats>,
    /// Latency metrics
    pub latency: LatencyMetrics,
    /// Recipe coverage
    pub recipe_coverage: f64,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// Schema version
    pub schema_version: u32,
}

/// XP and level data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XpData {
    /// Current level (0-100)
    pub level: u8,
    /// Current title
    pub title: String,
    /// Total XP
    pub total_xp: u64,
    /// XP required for next level
    pub next_level_xp: u64,
    /// Progress to next level (0.0 to 1.0)
    pub progress: f64,
}

/// Request counters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestCounters {
    /// Total requests handled
    pub total: u64,
    /// Successful requests
    pub successes: u64,
    /// Failed requests
    pub failures: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// Reliability metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    /// Average reliability score (all time)
    pub average: f64,
    /// Rolling last-50 reliability
    pub rolling_50: f64,
    /// Recent reliability scores for rolling calculation
    #[serde(default)]
    pub recent_scores: Vec<u8>,
}

/// Escalation tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EscalationMetrics {
    /// % that needed Junior verification
    pub junior_percent: f64,
    /// % that used a Doctor
    pub doctor_percent: f64,
    /// % that were solved purely from recipes
    pub recipe_solved_percent: f64,
    /// Counters for calculation
    pub junior_count: u64,
    pub doctor_count: u64,
    pub recipe_solved_count: u64,
}

/// Per-domain statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainStats {
    /// Number of requests in this domain
    pub count: u64,
    /// Success rate in this domain
    pub success_rate: f64,
    /// Average reliability in this domain
    pub avg_reliability: f64,
    /// Successes for rate calculation
    pub successes: u64,
    /// Total reliability for avg calculation
    pub total_reliability: u64,
}

/// Latency metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyMetrics {
    /// Median latency (ms) - total request
    pub median_total_ms: u64,
    /// P95 latency (ms) - total request
    pub p95_total_ms: u64,
    /// Recent latencies for percentile calculation
    #[serde(default)]
    pub recent_latencies: Vec<u64>,
    /// By-component latencies
    pub translator_median_ms: Option<u64>,
    pub tools_median_ms: Option<u64>,
    pub junior_median_ms: Option<u64>,
}

impl Default for RpgStats {
    fn default() -> Self {
        Self {
            xp: XpData::default(),
            requests: RequestCounters::default(),
            reliability: ReliabilityMetrics::default(),
            escalations: EscalationMetrics::default(),
            domains: HashMap::new(),
            latency: LatencyMetrics::default(),
            recipe_coverage: 0.0,
            updated_at: Utc::now(),
            schema_version: 1,
        }
    }
}

impl RpgStats {
    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(STATS_FILE) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(STATS_FILE).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(STATS_FILE, json)
    }

    /// Sync XP data from XpState
    pub fn sync_from_xp_state(&mut self) {
        let xp_state = XpState::load();
        let level = xp_state.level();
        let next_xp = xp_state.xp_for_next_level();

        // Calculate progress to next level
        let current_level_xp = if level == 0 {
            0
        } else {
            // Approximate previous level threshold
            next_xp.saturating_sub(if level < 20 { 100 * level as u64 } else { 1000 })
        };
        let progress = if next_xp > current_level_xp {
            (xp_state.total_xp.saturating_sub(current_level_xp)) as f64
                / (next_xp - current_level_xp) as f64
        } else {
            0.0
        };

        self.xp = XpData {
            level,
            title: title_for_level(level).to_string(),
            total_xp: xp_state.total_xp,
            next_level_xp: next_xp,
            progress: progress.clamp(0.0, 1.0),
        };
    }

    /// Record a completed request
    pub fn record_request(
        &mut self,
        success: bool,
        reliability_score: u8,
        domain: &str,
        used_junior: bool,
        used_doctor: bool,
        used_recipe: bool,
        total_latency_ms: u64,
    ) {
        // Update request counters
        self.requests.total += 1;
        if success {
            self.requests.successes += 1;
        } else {
            self.requests.failures += 1;
        }
        self.requests.success_rate = self.requests.successes as f64 / self.requests.total as f64;

        // Update reliability
        self.reliability.recent_scores.push(reliability_score);
        if self.reliability.recent_scores.len() > ROLLING_WINDOW_SIZE {
            self.reliability.recent_scores.remove(0);
        }
        self.reliability.rolling_50 = if self.reliability.recent_scores.is_empty() {
            0.0
        } else {
            self.reliability
                .recent_scores
                .iter()
                .map(|&s| s as f64)
                .sum::<f64>()
                / self.reliability.recent_scores.len() as f64
        };
        // Update average (running average)
        let n = self.requests.total as f64;
        self.reliability.average =
            ((self.reliability.average * (n - 1.0)) + reliability_score as f64) / n;

        // Update escalations
        if used_junior {
            self.escalations.junior_count += 1;
        }
        if used_doctor {
            self.escalations.doctor_count += 1;
        }
        if used_recipe {
            self.escalations.recipe_solved_count += 1;
        }
        self.escalations.junior_percent =
            self.escalations.junior_count as f64 / self.requests.total as f64 * 100.0;
        self.escalations.doctor_percent =
            self.escalations.doctor_count as f64 / self.requests.total as f64 * 100.0;
        self.escalations.recipe_solved_percent =
            self.escalations.recipe_solved_count as f64 / self.requests.total as f64 * 100.0;

        // Update domain stats
        let domain_entry = self.domains.entry(domain.to_string()).or_default();
        domain_entry.count += 1;
        if success {
            domain_entry.successes += 1;
        }
        domain_entry.total_reliability += reliability_score as u64;
        domain_entry.success_rate = domain_entry.successes as f64 / domain_entry.count as f64;
        domain_entry.avg_reliability =
            domain_entry.total_reliability as f64 / domain_entry.count as f64;

        // Update latency
        self.latency.recent_latencies.push(total_latency_ms);
        if self.latency.recent_latencies.len() > ROLLING_WINDOW_SIZE {
            self.latency.recent_latencies.remove(0);
        }
        self.update_latency_percentiles();

        self.updated_at = Utc::now();
    }

    /// Update latency percentiles
    fn update_latency_percentiles(&mut self) {
        if self.latency.recent_latencies.is_empty() {
            return;
        }

        let mut sorted = self.latency.recent_latencies.clone();
        sorted.sort();

        // Median (50th percentile)
        let mid = sorted.len() / 2;
        self.latency.median_total_ms = if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2
        } else {
            sorted[mid]
        };

        // P95 (95th percentile)
        let p95_idx = (sorted.len() as f64 * 0.95) as usize;
        self.latency.p95_total_ms = sorted
            .get(p95_idx.min(sorted.len() - 1))
            .copied()
            .unwrap_or(0);
    }

    /// Format XP progress bar for display
    pub fn format_xp_bar(&self, width: usize) -> String {
        let filled = (self.xp.progress * width as f64) as usize;
        let empty = width.saturating_sub(filled);
        format!(
            "[{}{}] {:>3}%",
            "=".repeat(filled),
            " ".repeat(empty),
            (self.xp.progress * 100.0) as u8
        )
    }

    /// Format success rate bar for display
    pub fn format_success_bar(&self, width: usize) -> String {
        let filled = (self.requests.success_rate * width as f64) as usize;
        let empty = width.saturating_sub(filled);
        format!(
            "[{}{}] {:>5.1}%",
            "#".repeat(filled),
            " ".repeat(empty),
            self.requests.success_rate * 100.0
        )
    }

    /// Format recipe coverage bar for display
    pub fn format_coverage_bar(&self, width: usize) -> String {
        let filled = (self.recipe_coverage / 100.0 * width as f64) as usize;
        let empty = width.saturating_sub(filled);
        format!(
            "[{}{}] {:>5.1}%",
            "*".repeat(filled),
            " ".repeat(empty),
            self.recipe_coverage
        )
    }

    /// Format complete stats block for annactl status
    pub fn format_status_block(&self) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push("  [RPG STATS]".to_string());

        // Level and XP
        lines.push(format!(
            "  Level {} {} | XP: {} / {}",
            self.xp.level, self.xp.title, self.xp.total_xp, self.xp.next_level_xp
        ));
        lines.push(format!("  XP Progress: {}", self.format_xp_bar(20)));

        // Request counters
        lines.push(format!(
            "  Requests: {} total, {} success, {} failed",
            self.requests.total, self.requests.successes, self.requests.failures
        ));
        lines.push(format!("  Success Rate: {}", self.format_success_bar(20)));

        // Reliability
        lines.push(format!(
            "  Reliability: {:.1}% avg, {:.1}% last-50",
            self.reliability.average, self.reliability.rolling_50
        ));

        // Escalations
        lines.push(format!(
            "  Escalations: {:.1}% junior, {:.1}% doctor, {:.1}% recipe-solved",
            self.escalations.junior_percent,
            self.escalations.doctor_percent,
            self.escalations.recipe_solved_percent
        ));

        // Recipe coverage
        lines.push(format!(
            "  Recipe Coverage: {}",
            self.format_coverage_bar(20)
        ));

        // Latency
        if self.requests.total > 0 {
            lines.push(format!(
                "  Latency: {}ms median, {}ms p95",
                self.latency.median_total_ms, self.latency.p95_total_ms
            ));
        } else {
            lines.push("  Latency: not available (no requests yet)".to_string());
        }

        // Domain breakdown (top 5 by count)
        if !self.domains.is_empty() {
            lines.push("  By Domain:".to_string());
            let mut domain_list: Vec<_> = self.domains.iter().collect();
            domain_list.sort_by(|a, b| b.1.count.cmp(&a.1.count));
            for (domain, stats) in domain_list.iter().take(5) {
                lines.push(format!(
                    "    {}: {} requests, {:.0}% success, {:.0}% reliability",
                    domain,
                    stats.count,
                    stats.success_rate * 100.0,
                    stats.avg_reliability
                ));
            }
        }

        lines.join("\n")
    }

    /// Check if stats are available (have data)
    pub fn is_available(&self) -> bool {
        self.requests.total > 0 || self.xp.total_xp > 0
    }
}

/// Manager for RPG stats
pub struct RpgStatsManager;

impl RpgStatsManager {
    /// Get current stats (with XP sync)
    pub fn get_stats() -> RpgStats {
        let mut stats = RpgStats::load();
        stats.sync_from_xp_state();
        stats
    }

    /// Record a completed case
    pub fn record_case(
        success: bool,
        reliability_score: u8,
        domain: &str,
        used_junior: bool,
        used_doctor: bool,
        used_recipe: bool,
        total_latency_ms: u64,
    ) -> std::io::Result<()> {
        let mut stats = RpgStats::load();
        stats.sync_from_xp_state();
        stats.record_request(
            success,
            reliability_score,
            domain,
            used_junior,
            used_doctor,
            used_recipe,
            total_latency_ms,
        );
        stats.save()
    }

    /// Update recipe coverage from recipe engine
    pub fn update_recipe_coverage(coverage_percent: f64) -> std::io::Result<()> {
        let mut stats = RpgStats::load();
        stats.recipe_coverage = coverage_percent;
        stats.updated_at = Utc::now();
        stats.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_for_level() {
        assert_eq!(title_for_level(0), "Intern");
        assert_eq!(title_for_level(2), "Intern");
        assert_eq!(title_for_level(3), "Apprentice");
        assert_eq!(title_for_level(12), "Engineer");
        assert_eq!(title_for_level(30), "Grandmaster");
        assert_eq!(title_for_level(100), "Grandmaster");
    }

    #[test]
    fn test_stats_default() {
        let stats = RpgStats::default();
        assert_eq!(stats.xp.level, 0);
        assert_eq!(stats.requests.total, 0);
        assert_eq!(stats.reliability.average, 0.0);
    }

    #[test]
    fn test_record_request() {
        let mut stats = RpgStats::default();
        stats.record_request(true, 85, "network", true, false, false, 150);

        assert_eq!(stats.requests.total, 1);
        assert_eq!(stats.requests.successes, 1);
        assert_eq!(stats.requests.success_rate, 1.0);
        assert_eq!(stats.reliability.average, 85.0);
        assert_eq!(stats.escalations.junior_count, 1);
        assert_eq!(stats.domains.get("network").unwrap().count, 1);
    }

    #[test]
    fn test_rolling_reliability() {
        let mut stats = RpgStats::default();

        // Add 10 scores
        for i in 0..10 {
            stats.record_request(true, 80 + i as u8, "test", false, false, false, 100);
        }

        assert_eq!(stats.reliability.recent_scores.len(), 10);
        // Average of 80..90 = 84.5
        assert!((stats.reliability.rolling_50 - 84.5).abs() < 0.1);
    }

    #[test]
    fn test_latency_percentiles() {
        let mut stats = RpgStats::default();

        // Add various latencies
        for ms in [100, 150, 200, 250, 300, 350, 400, 450, 500, 1000] {
            stats.record_request(true, 90, "test", false, false, false, ms);
        }

        assert!(stats.latency.median_total_ms > 0);
        assert!(stats.latency.p95_total_ms >= stats.latency.median_total_ms);
    }

    #[test]
    fn test_format_bars() {
        let mut stats = RpgStats::default();
        stats.xp.progress = 0.5;
        stats.requests.success_rate = 0.75;
        stats.recipe_coverage = 25.0;

        let xp_bar = stats.format_xp_bar(10);
        assert!(xp_bar.contains("====="));

        let success_bar = stats.format_success_bar(10);
        assert!(success_bar.contains("#######"));

        let coverage_bar = stats.format_coverage_bar(10);
        assert!(coverage_bar.contains("**"));
    }
}
