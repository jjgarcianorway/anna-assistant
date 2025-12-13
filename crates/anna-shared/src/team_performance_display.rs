//! Team Performance Display (Phase 71)
//!
//! Provides display functions for showing team performance metrics,
//! comparing teams, and tracking improvement over time.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Team identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamId {
    Desktop,
    Storage,
    Network,
    Performance,
    Services,
    Security,
    Hardware,
    General,
}

impl TeamId {
    /// Display name for the team
    pub fn display(&self) -> &'static str {
        match self {
            Self::Desktop => "Desktop",
            Self::Storage => "Storage",
            Self::Network => "Network",
            Self::Performance => "Performance",
            Self::Services => "Services",
            Self::Security => "Security",
            Self::Hardware => "Hardware",
            Self::General => "General",
        }
    }

    /// Get all teams
    pub fn all() -> Vec<TeamId> {
        vec![
            Self::Desktop,
            Self::Storage,
            Self::Network,
            Self::Performance,
            Self::Services,
            Self::Security,
            Self::Hardware,
            Self::General,
        ]
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<TeamId> {
        match s.to_lowercase().as_str() {
            "desktop" => Some(Self::Desktop),
            "storage" => Some(Self::Storage),
            "network" => Some(Self::Network),
            "performance" => Some(Self::Performance),
            "services" => Some(Self::Services),
            "security" => Some(Self::Security),
            "hardware" => Some(Self::Hardware),
            "general" => Some(Self::General),
            _ => None,
        }
    }
}

/// Performance metrics for a single team
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamMetrics {
    /// Team identifier
    pub team_id: String,
    /// Total tickets handled
    pub tickets_handled: u64,
    /// Tickets resolved successfully
    pub tickets_resolved: u64,
    /// Tickets escalated (to senior or other team)
    pub tickets_escalated: u64,
    /// Total resolution time in milliseconds
    pub total_resolution_time_ms: u64,
    /// Fastest resolution in ms
    pub fastest_resolution_ms: Option<u64>,
    /// Slowest resolution in ms
    pub slowest_resolution_ms: Option<u64>,
    /// Junior tickets handled
    pub junior_tickets: u64,
    /// Senior tickets handled
    pub senior_tickets: u64,
    /// Current active tickets
    pub active_tickets: u64,
}

impl TeamMetrics {
    /// Create new metrics for a team
    pub fn new(team_id: impl Into<String>) -> Self {
        Self {
            team_id: team_id.into(),
            ..Default::default()
        }
    }

    /// Record a ticket being handled
    pub fn record_ticket(&mut self, resolution_time_ms: u64, resolved: bool, is_senior: bool) {
        self.tickets_handled += 1;

        if resolved {
            self.tickets_resolved += 1;
        }

        if is_senior {
            self.senior_tickets += 1;
        } else {
            self.junior_tickets += 1;
        }

        self.total_resolution_time_ms += resolution_time_ms;

        // Update fastest/slowest
        if resolution_time_ms > 0 {
            match self.fastest_resolution_ms {
                Some(f) if resolution_time_ms < f => {
                    self.fastest_resolution_ms = Some(resolution_time_ms)
                }
                None => self.fastest_resolution_ms = Some(resolution_time_ms),
                _ => {}
            }

            match self.slowest_resolution_ms {
                Some(s) if resolution_time_ms > s => {
                    self.slowest_resolution_ms = Some(resolution_time_ms)
                }
                None => self.slowest_resolution_ms = Some(resolution_time_ms),
                _ => {}
            }
        }
    }

    /// Record an escalation
    pub fn record_escalation(&mut self) {
        self.tickets_escalated += 1;
    }

    /// Success rate (resolved / handled)
    pub fn success_rate(&self) -> f64 {
        if self.tickets_handled == 0 {
            return 0.0;
        }
        (self.tickets_resolved as f64 / self.tickets_handled as f64) * 100.0
    }

    /// Escalation rate (escalated / handled)
    pub fn escalation_rate(&self) -> f64 {
        if self.tickets_handled == 0 {
            return 0.0;
        }
        (self.tickets_escalated as f64 / self.tickets_handled as f64) * 100.0
    }

    /// Average resolution time in milliseconds
    pub fn avg_resolution_ms(&self) -> u64 {
        if self.tickets_resolved == 0 {
            return 0;
        }
        self.total_resolution_time_ms / self.tickets_resolved
    }

    /// Senior/Junior ratio
    pub fn senior_ratio(&self) -> f64 {
        let total = self.junior_tickets + self.senior_tickets;
        if total == 0 {
            return 0.0;
        }
        (self.senior_tickets as f64 / total as f64) * 100.0
    }
}

/// Team performance tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamPerformance {
    /// Metrics by team
    pub teams: HashMap<String, TeamMetrics>,
    /// Total tickets across all teams
    pub total_tickets: u64,
    /// Total resolved across all teams
    pub total_resolved: u64,
}

impl TeamPerformance {
    /// Create a new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create metrics for a team
    pub fn get_or_create(&mut self, team_id: &str) -> &mut TeamMetrics {
        self.teams
            .entry(team_id.to_string())
            .or_insert_with(|| TeamMetrics::new(team_id))
    }

    /// Record a ticket for a team
    pub fn record_ticket(
        &mut self,
        team_id: &str,
        resolution_time_ms: u64,
        resolved: bool,
        is_senior: bool,
    ) {
        self.total_tickets += 1;
        if resolved {
            self.total_resolved += 1;
        }

        let metrics = self.get_or_create(team_id);
        metrics.record_ticket(resolution_time_ms, resolved, is_senior);
    }

    /// Get metrics for a team
    pub fn get_team(&self, team_id: &str) -> Option<&TeamMetrics> {
        self.teams.get(team_id)
    }

    /// Get teams sorted by ticket count (most active first)
    pub fn by_activity(&self) -> Vec<(&String, &TeamMetrics)> {
        let mut teams: Vec<_> = self.teams.iter().collect();
        teams.sort_by(|a, b| b.1.tickets_handled.cmp(&a.1.tickets_handled));
        teams
    }

    /// Get teams sorted by success rate (best first)
    pub fn by_success_rate(&self) -> Vec<(&String, &TeamMetrics)> {
        let mut teams: Vec<_> = self.teams.iter().collect();
        teams.sort_by(|a, b| {
            b.1.success_rate()
                .partial_cmp(&a.1.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        teams
    }

    /// Get teams sorted by average resolution time (fastest first)
    pub fn by_speed(&self) -> Vec<(&String, &TeamMetrics)> {
        let mut teams: Vec<_> = self.teams.iter().filter(|(_, m)| m.avg_resolution_ms() > 0).collect();
        teams.sort_by(|a, b| a.1.avg_resolution_ms().cmp(&b.1.avg_resolution_ms()));
        teams
    }

    /// Get the most active team
    pub fn most_active(&self) -> Option<(&String, &TeamMetrics)> {
        self.by_activity().first().copied()
    }

    /// Get the best performing team (by success rate)
    pub fn best_performing(&self) -> Option<(&String, &TeamMetrics)> {
        self.by_success_rate()
            .into_iter()
            .find(|(_, m)| m.tickets_handled >= 3)
    }

    /// Get the fastest team (by average resolution)
    pub fn fastest(&self) -> Option<(&String, &TeamMetrics)> {
        self.by_speed().first().copied()
    }

    /// Overall success rate
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_tickets == 0 {
            return 0.0;
        }
        (self.total_resolved as f64 / self.total_tickets as f64) * 100.0
    }
}

/// Format duration in milliseconds as human-readable
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3600000 {
        format!("{:.1}m", ms as f64 / 60000.0)
    } else {
        format!("{:.1}h", ms as f64 / 3600000.0)
    }
}

/// Format team performance as full display
pub fn format_team_performance(perf: &TeamPerformance) -> String {
    let mut lines = Vec::new();

    lines.push("=== Team Performance ===".to_string());
    lines.push(String::new());

    // Overall summary
    lines.push(format!("Total Tickets: {}", perf.total_tickets));
    lines.push(format!("Total Resolved: {}", perf.total_resolved));
    lines.push(format!("Overall Success Rate: {:.1}%", perf.overall_success_rate()));
    lines.push(String::new());

    // Highlights
    if let Some((name, metrics)) = perf.most_active() {
        lines.push(format!(
            "Most Active: {} ({} tickets)",
            name, metrics.tickets_handled
        ));
    }

    if let Some((name, metrics)) = perf.best_performing() {
        lines.push(format!(
            "Best Performing: {} ({:.1}% success)",
            name,
            metrics.success_rate()
        ));
    }

    if let Some((name, metrics)) = perf.fastest() {
        lines.push(format!(
            "Fastest: {} (avg {})",
            name,
            format_duration_ms(metrics.avg_resolution_ms())
        ));
    }

    lines.push(String::new());

    // Team breakdown
    lines.push("--- Team Breakdown ---".to_string());

    let teams = perf.by_activity();
    if teams.is_empty() {
        lines.push("  No team data yet.".to_string());
    } else {
        for (name, metrics) in teams {
            lines.push(format!("\n  {} Team:", name));
            lines.push(format!("    Tickets: {} handled, {} resolved", metrics.tickets_handled, metrics.tickets_resolved));
            lines.push(format!("    Success Rate: {:.1}%", metrics.success_rate()));
            lines.push(format!("    Escalation Rate: {:.1}%", metrics.escalation_rate()));

            if metrics.avg_resolution_ms() > 0 {
                lines.push(format!("    Avg Resolution: {}", format_duration_ms(metrics.avg_resolution_ms())));
            }

            if metrics.fastest_resolution_ms.is_some() || metrics.slowest_resolution_ms.is_some() {
                let fastest = metrics.fastest_resolution_ms.map(format_duration_ms).unwrap_or_else(|| "-".to_string());
                let slowest = metrics.slowest_resolution_ms.map(format_duration_ms).unwrap_or_else(|| "-".to_string());
                lines.push(format!("    Range: {} - {}", fastest, slowest));
            }

            lines.push(format!(
                "    Junior/Senior: {}/{}",
                metrics.junior_tickets, metrics.senior_tickets
            ));
        }
    }

    lines.join("\n")
}

/// Format team performance compact (for greetings)
pub fn format_team_performance_compact(perf: &TeamPerformance) -> String {
    let teams = perf.by_activity();
    if teams.is_empty() {
        return "No team activity yet.".to_string();
    }

    let top_teams: Vec<String> = teams
        .iter()
        .take(3)
        .map(|(name, m)| format!("{}: {}t", name, m.tickets_handled))
        .collect();

    format!(
        "Top teams: {} | {:.0}% overall success",
        top_teams.join(", "),
        perf.overall_success_rate()
    )
}

/// Format team performance one-line
pub fn format_team_performance_oneline(perf: &TeamPerformance) -> String {
    format!(
        "Teams: {} active, {} tickets, {:.0}% success",
        perf.teams.len(),
        perf.total_tickets,
        perf.overall_success_rate()
    )
}

/// Generate a fun fact about team performance
pub fn team_performance_fun_fact(perf: &TeamPerformance) -> Option<String> {
    if perf.teams.is_empty() {
        return None;
    }

    let facts = vec![
        perf.total_tickets >= 100,
        perf.overall_success_rate() >= 95.0,
        perf.teams.len() >= 5,
    ];

    let messages = vec![
        format!(
            "Century of teamwork! {} tickets handled across all teams.",
            perf.total_tickets
        ),
        format!(
            "Elite performance! {:.1}% overall success rate.",
            perf.overall_success_rate()
        ),
        format!(
            "Full house! {} different teams have been called into action.",
            perf.teams.len()
        ),
    ];

    for (i, fact) in facts.iter().enumerate() {
        if *fact {
            return Some(messages[i].clone());
        }
    }

    // Default fact
    if let Some((name, _)) = perf.most_active() {
        return Some(format!(
            "The {} team is the most consulted, handling the most tickets.",
            name
        ));
    }

    None
}

/// Check if query is asking about team performance
pub fn is_team_performance_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "team performance",
        "team stats",
        "how are the teams",
        "team metrics",
        "best team",
        "fastest team",
        "most active team",
        "team breakdown",
        "department stats",
        "team activity",
        "which team",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Get performance grade for a team
pub fn team_grade(metrics: &TeamMetrics) -> &'static str {
    let success = metrics.success_rate();
    let escalation = metrics.escalation_rate();

    if metrics.tickets_handled < 3 {
        return "N/A";
    }

    if success >= 95.0 && escalation <= 10.0 {
        "A+"
    } else if success >= 90.0 && escalation <= 20.0 {
        "A"
    } else if success >= 80.0 && escalation <= 30.0 {
        "B"
    } else if success >= 70.0 {
        "C"
    } else if success >= 50.0 {
        "D"
    } else {
        "F"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_id_display() {
        assert_eq!(TeamId::Desktop.display(), "Desktop");
        assert_eq!(TeamId::Network.display(), "Network");
    }

    #[test]
    fn test_team_id_parse() {
        assert_eq!(TeamId::parse("desktop"), Some(TeamId::Desktop));
        assert_eq!(TeamId::parse("NETWORK"), Some(TeamId::Network));
        assert_eq!(TeamId::parse("unknown"), None);
    }

    #[test]
    fn test_team_id_all() {
        let all = TeamId::all();
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn test_team_metrics_new() {
        let metrics = TeamMetrics::new("Desktop");
        assert_eq!(metrics.team_id, "Desktop");
        assert_eq!(metrics.tickets_handled, 0);
    }

    #[test]
    fn test_team_metrics_record_ticket() {
        let mut metrics = TeamMetrics::new("Desktop");
        metrics.record_ticket(5000, true, false);

        assert_eq!(metrics.tickets_handled, 1);
        assert_eq!(metrics.tickets_resolved, 1);
        assert_eq!(metrics.junior_tickets, 1);
        assert_eq!(metrics.fastest_resolution_ms, Some(5000));
    }

    #[test]
    fn test_team_metrics_success_rate() {
        let mut metrics = TeamMetrics::new("Desktop");
        metrics.record_ticket(1000, true, false);
        metrics.record_ticket(1000, true, false);
        metrics.record_ticket(1000, false, false);

        // 2 resolved out of 3 = 66.67%
        assert!((metrics.success_rate() - 66.67).abs() < 1.0);
    }

    #[test]
    fn test_team_metrics_avg_resolution() {
        let mut metrics = TeamMetrics::new("Desktop");
        metrics.record_ticket(1000, true, false);
        metrics.record_ticket(3000, true, false);

        // Average of 1000 and 3000 = 2000
        assert_eq!(metrics.avg_resolution_ms(), 2000);
    }

    #[test]
    fn test_team_performance_record() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 5000, true, false);

        assert_eq!(perf.total_tickets, 1);
        assert_eq!(perf.total_resolved, 1);
        assert!(perf.teams.contains_key("Desktop"));
    }

    #[test]
    fn test_team_performance_by_activity() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 1000, true, false);
        perf.record_ticket("Desktop", 1000, true, false);
        perf.record_ticket("Network", 1000, true, false);

        let by_activity = perf.by_activity();
        assert_eq!(by_activity[0].0, "Desktop");
    }

    #[test]
    fn test_team_performance_most_active() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 1000, true, false);
        perf.record_ticket("Desktop", 1000, true, false);
        perf.record_ticket("Network", 1000, true, false);

        let (name, _) = perf.most_active().unwrap();
        assert_eq!(name, "Desktop");
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(1500), "1.5s");
        assert_eq!(format_duration_ms(90000), "1.5m");
        assert_eq!(format_duration_ms(5400000), "1.5h");
    }

    #[test]
    fn test_format_team_performance() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 5000, true, false);

        let output = format_team_performance(&perf);
        assert!(output.contains("Team Performance"));
        assert!(output.contains("Desktop"));
    }

    #[test]
    fn test_format_team_performance_compact() {
        let mut perf = TeamPerformance::new();
        perf.record_ticket("Desktop", 1000, true, false);

        let output = format_team_performance_compact(&perf);
        assert!(output.contains("Desktop"));
    }

    #[test]
    fn test_is_team_performance_query() {
        assert!(is_team_performance_query("show team performance"));
        assert!(is_team_performance_query("what are the team stats?"));
        assert!(is_team_performance_query("which team is best?"));
        assert!(!is_team_performance_query("how do I install vim?"));
    }

    #[test]
    fn test_team_grade() {
        let mut excellent = TeamMetrics::new("Test");
        for _ in 0..10 {
            excellent.record_ticket(1000, true, false);
        }
        assert_eq!(team_grade(&excellent), "A+");

        let mut poor = TeamMetrics::new("Test2");
        for _ in 0..5 {
            poor.record_ticket(1000, true, false);
        }
        for _ in 0..5 {
            poor.record_ticket(1000, false, false);
        }
        assert_eq!(team_grade(&poor), "D");
    }

    #[test]
    fn test_team_performance_fun_fact() {
        let mut perf = TeamPerformance::new();

        // Empty - no fact
        assert!(team_performance_fun_fact(&perf).is_none());

        // Add some data
        perf.record_ticket("Desktop", 1000, true, false);
        let fact = team_performance_fun_fact(&perf);
        assert!(fact.is_some());
    }
}
