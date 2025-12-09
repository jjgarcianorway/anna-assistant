//! Aggregated event statistics (v0.0.190).

use serde::{Deserialize, Serialize};

use crate::streaks::{calculate_lucky_team, calculate_streaks, TeamOutcome};

use super::types::EventRecord;

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
    /// Current streak (consecutive days with activity)
    pub current_streak: u32,
    /// Best streak ever
    pub best_streak: u32,
    /// Unique days with activity
    pub active_days: u32,
    /// Team with highest success rate (lucky team)
    pub lucky_team: Option<String>,
    /// Lucky team success rate
    pub lucky_team_rate: f32,
    /// Total interactions across tickets
    pub total_interactions: u64,
    /// Average interactions per ticket
    pub avg_interactions: f32,
    /// Maximum interactions on a ticket
    pub max_interactions: u32,
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

            // Interactions
            agg.total_interactions += record.interactions as u64;
            agg.max_interactions = agg.max_interactions.max(record.interactions);

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
        agg.avg_interactions = agg.total_interactions as f32 / agg.total_requests as f32;

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

        // Compute streaks and lucky team
        agg.compute_streaks(records);
        agg.compute_lucky_team(records);

        agg
    }

    /// Compute usage streaks from records (delegates to streaks module)
    fn compute_streaks(&mut self, records: &[EventRecord]) {
        let timestamps: Vec<u64> = records.iter().map(|r| r.timestamp).collect();
        let stats = calculate_streaks(&timestamps);
        self.current_streak = stats.current_streak;
        self.best_streak = stats.best_streak;
        self.active_days = stats.active_days;
    }

    /// Find the lucky team (delegates to streaks module)
    fn compute_lucky_team(&mut self, records: &[EventRecord]) {
        let outcomes: Vec<TeamOutcome> = records
            .iter()
            .map(|r| TeamOutcome {
                team: r.team.clone(),
                success: r.outcome == "verified",
            })
            .collect();
        let stats = calculate_lucky_team(&outcomes);
        self.lucky_team = stats.team;
        self.lucky_team_rate = stats.rate;
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
        self.level = xp_to_level(self.xp);
        self.title = level_title(self.level);
    }
}

/// Convert XP to level using logistic-style progression
pub fn xp_to_level(xp: u64) -> u32 {
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

/// Get title for a given level
pub fn level_title(level: u32) -> String {
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
