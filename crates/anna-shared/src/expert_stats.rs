//! Expert Ticket Statistics (v0.0.489).
//!
//! Tracks tickets closed per expert (junior and senior).
//! Provides detailed statistics on expert performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expert level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpertLevel {
    /// Junior specialist
    Junior,
    /// Senior specialist
    Senior,
}

impl ExpertLevel {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Junior => "Junior",
            Self::Senior => "Senior",
        }
    }

    /// Get short name
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Junior => "Jr",
            Self::Senior => "Sr",
        }
    }
}

/// An expert in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expert {
    /// Unique ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Department/team
    pub department: String,
    /// Level (junior/senior)
    pub level: ExpertLevel,
}

impl Expert {
    /// Create new expert
    pub fn new(id: &str, name: &str, department: &str, level: ExpertLevel) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            department: department.to_string(),
            level,
        }
    }

    /// Get full title
    pub fn title(&self) -> String {
        format!("{} {} ({})", self.level.display_name(), self.department, self.name)
    }
}

/// Statistics for a single expert
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpertStatistics {
    /// Tickets closed
    pub tickets_closed: u64,
    /// Tickets escalated (for juniors)
    pub tickets_escalated: u64,
    /// Average confidence on resolutions
    pub avg_confidence: f64,
    /// Total confidence (for averaging)
    total_confidence: f64,
    /// High confidence resolutions (>90%)
    pub high_confidence_count: u64,
    /// Resolution count (for averaging)
    resolution_count: u64,
    /// Average response time (ms)
    pub avg_response_ms: f64,
    /// Total response time (for averaging)
    total_response_ms: u64,
    response_count: u64,
}

impl ExpertStatistics {
    /// Record a closed ticket
    pub fn record_closed(&mut self, confidence: f64, response_ms: Option<u64>) {
        self.tickets_closed += 1;
        self.resolution_count += 1;
        self.total_confidence += confidence;
        self.avg_confidence = self.total_confidence / self.resolution_count as f64;

        if confidence >= 0.9 {
            self.high_confidence_count += 1;
        }

        if let Some(ms) = response_ms {
            self.total_response_ms += ms;
            self.response_count += 1;
            self.avg_response_ms = self.total_response_ms as f64 / self.response_count as f64;
        }
    }

    /// Record an escalation
    pub fn record_escalation(&mut self) {
        self.tickets_escalated += 1;
    }

    /// Get escalation rate
    pub fn escalation_rate(&self) -> f64 {
        let total = self.tickets_closed + self.tickets_escalated;
        if total == 0 {
            0.0
        } else {
            self.tickets_escalated as f64 / total as f64 * 100.0
        }
    }

    /// Get high confidence rate
    pub fn high_confidence_rate(&self) -> f64 {
        if self.tickets_closed == 0 {
            0.0
        } else {
            self.high_confidence_count as f64 / self.tickets_closed as f64 * 100.0
        }
    }
}

/// Expert statistics tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpertStatsTracker {
    /// Registered experts
    pub experts: HashMap<String, Expert>,
    /// Statistics per expert
    pub stats: HashMap<String, ExpertStatistics>,
    /// Total junior tickets
    pub junior_total: u64,
    /// Total senior tickets
    pub senior_total: u64,
    /// Anna solo tickets
    pub anna_solo: u64,
}

impl ExpertStatsTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an expert
    pub fn register_expert(&mut self, expert: Expert) {
        self.experts.insert(expert.id.clone(), expert);
    }

    /// Record a closed ticket
    pub fn record_closed(
        &mut self,
        expert_id: &str,
        confidence: f64,
        response_ms: Option<u64>,
    ) {
        let stats = self.stats.entry(expert_id.to_string()).or_default();
        stats.record_closed(confidence, response_ms);

        // Update level totals
        if let Some(expert) = self.experts.get(expert_id) {
            match expert.level {
                ExpertLevel::Junior => self.junior_total += 1,
                ExpertLevel::Senior => self.senior_total += 1,
            }
        }
    }

    /// Record an escalation from junior to senior
    pub fn record_escalation(&mut self, junior_id: &str) {
        let stats = self.stats.entry(junior_id.to_string()).or_default();
        stats.record_escalation();
    }

    /// Record Anna solo resolution
    pub fn record_anna_solo(&mut self) {
        self.anna_solo += 1;
    }

    /// Get total tickets handled
    pub fn total_tickets(&self) -> u64 {
        self.junior_total + self.senior_total + self.anna_solo
    }

    /// Get Anna's share
    pub fn anna_share(&self) -> f64 {
        let total = self.total_tickets();
        if total == 0 {
            0.0
        } else {
            self.anna_solo as f64 / total as f64 * 100.0
        }
    }

    /// Get junior share
    pub fn junior_share(&self) -> f64 {
        let total = self.total_tickets();
        if total == 0 {
            0.0
        } else {
            self.junior_total as f64 / total as f64 * 100.0
        }
    }

    /// Get senior share
    pub fn senior_share(&self) -> f64 {
        let total = self.total_tickets();
        if total == 0 {
            0.0
        } else {
            self.senior_total as f64 / total as f64 * 100.0
        }
    }

    /// Get top performers by tickets closed
    pub fn top_performers(&self, limit: usize) -> Vec<(&str, &ExpertStatistics)> {
        let mut sorted: Vec<_> = self.stats.iter().collect();
        sorted.sort_by(|a, b| b.1.tickets_closed.cmp(&a.1.tickets_closed));
        sorted
            .into_iter()
            .take(limit)
            .map(|(id, stats)| (id.as_str(), stats))
            .collect()
    }

    /// Get experts by department
    pub fn by_department(&self, department: &str) -> Vec<(&Expert, &ExpertStatistics)> {
        self.experts
            .values()
            .filter(|e| e.department == department)
            .filter_map(|e| self.stats.get(&e.id).map(|s| (e, s)))
            .collect()
    }

    /// Get experts by level
    pub fn by_level(&self, level: ExpertLevel) -> Vec<(&Expert, &ExpertStatistics)> {
        self.experts
            .values()
            .filter(|e| e.level == level)
            .filter_map(|e| self.stats.get(&e.id).map(|s| (e, s)))
            .collect()
    }

    /// Get most reliable expert (highest confidence)
    pub fn most_reliable(&self) -> Option<(&str, f64)> {
        self.stats
            .iter()
            .filter(|(_, s)| s.tickets_closed >= 5)
            .max_by(|a, b| {
                a.1.avg_confidence
                    .partial_cmp(&b.1.avg_confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, s)| (id.as_str(), s.avg_confidence))
    }

    /// Get fastest responder
    pub fn fastest_responder(&self) -> Option<(&str, f64)> {
        self.stats
            .iter()
            .filter(|(_, s)| s.response_count > 0)
            .min_by(|a, b| {
                a.1.avg_response_ms
                    .partial_cmp(&b.1.avg_response_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, s)| (id.as_str(), s.avg_response_ms))
    }
}

/// Expert stats summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertStatsSummary {
    /// Total tickets
    pub total_tickets: u64,
    /// Anna solo count
    pub anna_solo: u64,
    /// Junior total
    pub junior_total: u64,
    /// Senior total
    pub senior_total: u64,
    /// Expert count
    pub expert_count: usize,
    /// Top performer name
    pub top_performer: Option<String>,
}

impl ExpertStatsTracker {
    /// Generate summary
    pub fn summary(&self) -> ExpertStatsSummary {
        let top = self.top_performers(1).first().map(|(id, _)| {
            self.experts
                .get(*id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| id.to_string())
        });

        ExpertStatsSummary {
            total_tickets: self.total_tickets(),
            anna_solo: self.anna_solo,
            junior_total: self.junior_total,
            senior_total: self.senior_total,
            expert_count: self.experts.len(),
            top_performer: top,
        }
    }
}

/// Format expert stats for display
pub fn format_expert_stats(tracker: &ExpertStatsTracker) -> String {
    let mut output = String::new();

    output.push_str("Expert Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    if tracker.total_tickets() == 0 {
        output.push_str("No tickets recorded yet.\n");
        return output;
    }

    output.push_str(&format!(
        "Total Tickets: {}\n",
        tracker.total_tickets()
    ));
    output.push_str(&format!(
        "  Anna Solo: {} ({:.1}%)\n",
        tracker.anna_solo,
        tracker.anna_share()
    ));
    output.push_str(&format!(
        "  Junior:    {} ({:.1}%)\n",
        tracker.junior_total,
        tracker.junior_share()
    ));
    output.push_str(&format!(
        "  Senior:    {} ({:.1}%)\n\n",
        tracker.senior_total,
        tracker.senior_share()
    ));

    let top = tracker.top_performers(5);
    if !top.is_empty() {
        output.push_str("Top Performers:\n");
        for (id, stats) in top {
            let name = tracker
                .experts
                .get(id)
                .map(|e| format!("{} ({})", e.name, e.level.short_name()))
                .unwrap_or_else(|| id.to_string());

            output.push_str(&format!(
                "  {} - {} closed, {:.0}% confidence\n",
                name,
                stats.tickets_closed,
                stats.avg_confidence * 100.0
            ));
        }
    }

    output
}

/// Format compact expert stats
pub fn format_expert_stats_compact(tracker: &ExpertStatsTracker) -> String {
    if tracker.total_tickets() == 0 {
        return "No tickets yet".to_string();
    }

    format!(
        "{} tickets: Anna {:.0}%, Jr {:.0}%, Sr {:.0}%",
        tracker.total_tickets(),
        tracker.anna_share(),
        tracker.junior_share(),
        tracker.senior_share()
    )
}

/// Generate fun fact about expert stats
pub fn expert_stats_fun_fact(tracker: &ExpertStatsTracker) -> Option<String> {
    if tracker.total_tickets() < 5 {
        return None;
    }

    let facts = vec![
        format!(
            "Anna handles {:.0}% of requests independently - {}!",
            tracker.anna_share(),
            if tracker.anna_share() > 50.0 {
                "she's becoming an expert"
            } else {
                "teamwork at its finest"
            }
        ),
        format!(
            "Juniors resolved {} tickets, seniors handled {} - great balance!",
            tracker.junior_total, tracker.senior_total
        ),
        tracker
            .most_reliable()
            .map(|(id, conf)| {
                let name = tracker
                    .experts
                    .get(id)
                    .map(|e| e.name.clone())
                    .unwrap_or_else(|| id.to_string());
                format!("{} has {:.0}% confidence - most reliable expert!", name, conf * 100.0)
            })
            .unwrap_or_else(|| "The team maintains high standards!".to_string()),
        format!(
            "{} experts working together across {} tickets!",
            tracker.experts.len(),
            tracker.total_tickets()
        ),
    ];

    let index = (tracker.total_tickets() as usize) % facts.len();
    Some(facts[index].clone())
}

/// Check if query is asking about expert stats
pub fn is_expert_stats_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "expert stats",
        "who closed",
        "tickets per expert",
        "top performer",
        "junior stats",
        "senior stats",
        "expert performance",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expert_level_display() {
        assert_eq!(ExpertLevel::Junior.display_name(), "Junior");
        assert_eq!(ExpertLevel::Senior.display_name(), "Senior");
        assert_eq!(ExpertLevel::Junior.short_name(), "Jr");
    }

    #[test]
    fn test_expert_new() {
        let expert = Expert::new("desktop-jr-1", "Alex", "Desktop", ExpertLevel::Junior);
        assert_eq!(expert.id, "desktop-jr-1");
        assert_eq!(expert.department, "Desktop");
    }

    #[test]
    fn test_expert_title() {
        let expert = Expert::new("net-sr-1", "Jordan", "Network", ExpertLevel::Senior);
        assert!(expert.title().contains("Senior"));
        assert!(expert.title().contains("Network"));
    }

    #[test]
    fn test_expert_stats_record_closed() {
        let mut stats = ExpertStatistics::default();

        stats.record_closed(0.95, Some(500));
        stats.record_closed(0.85, Some(600));

        assert_eq!(stats.tickets_closed, 2);
        assert_eq!(stats.high_confidence_count, 1);
        assert_eq!(stats.avg_response_ms, 550.0);
    }

    #[test]
    fn test_escalation_rate() {
        let mut stats = ExpertStatistics::default();

        stats.record_closed(0.9, None);
        stats.record_closed(0.9, None);
        stats.record_escalation();

        // 1 escalation / 3 total = 33.33%
        assert!(stats.escalation_rate() > 30.0);
        assert!(stats.escalation_rate() < 35.0);
    }

    #[test]
    fn test_tracker_register() {
        let mut tracker = ExpertStatsTracker::new();

        let expert = Expert::new("test-1", "Test", "Test Dept", ExpertLevel::Junior);
        tracker.register_expert(expert);

        assert_eq!(tracker.experts.len(), 1);
    }

    #[test]
    fn test_tracker_record_closed() {
        let mut tracker = ExpertStatsTracker::new();

        let expert = Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior);
        tracker.register_expert(expert);

        tracker.record_closed("jr-1", 0.9, Some(500));

        assert_eq!(tracker.junior_total, 1);
        assert_eq!(tracker.total_tickets(), 1);
    }

    #[test]
    fn test_anna_share() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.record_anna_solo();
        tracker.record_anna_solo();

        let expert = Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior);
        tracker.register_expert(expert);
        tracker.record_closed("jr-1", 0.9, None);

        // 2 anna / 3 total = 66.67%
        assert!(tracker.anna_share() > 60.0);
    }

    #[test]
    fn test_top_performers() {
        let mut tracker = ExpertStatsTracker::new();

        let expert1 = Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior);
        let expert2 = Expert::new("jr-2", "Sam", "Network", ExpertLevel::Junior);
        tracker.register_expert(expert1);
        tracker.register_expert(expert2);

        // Alex: 3 tickets
        for _ in 0..3 {
            tracker.record_closed("jr-1", 0.9, None);
        }
        // Sam: 1 ticket
        tracker.record_closed("jr-2", 0.9, None);

        let top = tracker.top_performers(2);
        assert_eq!(top[0].0, "jr-1");
    }

    #[test]
    fn test_by_level() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.register_expert(Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior));
        tracker.register_expert(Expert::new("sr-1", "Jordan", "Desktop", ExpertLevel::Senior));
        tracker.record_closed("jr-1", 0.9, None);
        tracker.record_closed("sr-1", 0.95, None);

        let juniors = tracker.by_level(ExpertLevel::Junior);
        assert_eq!(juniors.len(), 1);
    }

    #[test]
    fn test_summary() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.register_expert(Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior));
        tracker.record_closed("jr-1", 0.9, None);
        tracker.record_anna_solo();

        let summary = tracker.summary();
        assert_eq!(summary.total_tickets, 2);
        assert_eq!(summary.anna_solo, 1);
    }

    #[test]
    fn test_format_compact() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.register_expert(Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior));
        tracker.record_closed("jr-1", 0.9, None);
        tracker.record_anna_solo();

        let output = format_expert_stats_compact(&tracker);
        assert!(output.contains("2 tickets"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = ExpertStatsTracker::new();

        tracker.register_expert(Expert::new("jr-1", "Alex", "Desktop", ExpertLevel::Junior));
        for _ in 0..10 {
            tracker.record_closed("jr-1", 0.9, None);
        }

        let fact = expert_stats_fun_fact(&tracker);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_expert_stats_query() {
        assert!(is_expert_stats_query("show expert stats"));
        assert!(is_expert_stats_query("who closed the most tickets"));
        assert!(is_expert_stats_query("top performer"));

        assert!(!is_expert_stats_query("install vim"));
        assert!(!is_expert_stats_query("status"));
    }
}
