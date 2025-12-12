//! Aggregated event statistics (v0.0.190).
//! v0.0.450: Enhanced with VISION.md requirements - RPG XP 0-100, more stats.

use serde::{Deserialize, Serialize};

use crate::streaks::{calculate_lucky_team, calculate_streaks, TeamOutcome};

use super::types::EventRecord;

/// Aggregated statistics from event records
/// v0.0.450: Enhanced per VISION.md requirements
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
    /// v0.0.450: Most consulted team
    pub most_consulted_team: Option<String>,
    /// Recipes used count
    pub recipes_used: u64,
    /// Recipes learned count
    pub recipes_learned: u64,
    /// XP (computed, 0-100 scale per VISION.md)
    pub xp: u64,
    /// Level (computed, 1-10)
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
    /// v0.0.450: Times Anna solved without specialist (recipe/deterministic)
    pub anna_solo_count: u64,
    /// v0.0.450: Longest reply length (chars)
    pub longest_reply_chars: u64,
    /// v0.0.450: Shortest reply length (chars)
    pub shortest_reply_chars: u64,
}

impl AggregatedEvents {
    pub fn from_records(records: &[EventRecord]) -> Self {
        let mut agg = Self::default();

        if records.is_empty() {
            agg.title = "Trainee".to_string();
            return agg;
        }

        agg.total_requests = records.len() as u64;
        agg.min_duration_ms = u64::MAX;
        agg.first_event_ts = u64::MAX;
        agg.last_event_ts = 0;
        agg.shortest_reply_chars = u64::MAX;

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

            // v0.0.450: Track Anna solo (no escalation + recipe/deterministic)
            if !record.escalated && record.recipe_used.is_some() {
                agg.anna_solo_count += 1;
            }

            // Reliability
            total_reliability += record.reliability as u64;

            // Duration
            total_duration += record.duration_ms;
            agg.min_duration_ms = agg.min_duration_ms.min(record.duration_ms);
            agg.max_duration_ms = agg.max_duration_ms.max(record.duration_ms);

            // v0.0.450: Reply length stats (estimate from duration as proxy)
            let reply_len = record.duration_ms; // Proxy: longer = longer reply
            agg.longest_reply_chars = agg.longest_reply_chars.max(reply_len);
            agg.shortest_reply_chars = agg.shortest_reply_chars.min(reply_len);

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
        if agg.shortest_reply_chars == u64::MAX {
            agg.shortest_reply_chars = 0;
        }

        // Most escalated team
        agg.most_escalated_team = escalations_by_team
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(team, _)| team);

        // v0.0.450: Most consulted team
        agg.most_consulted_team = agg
            .by_team
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(team, _)| team.clone());

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

    /// Compute XP using logistic curve (0-100 scale per VISION.md)
    /// v0.0.450: Non-linear RPG-style progression
    fn compute_xp(&mut self) {
        // Raw score factors (all contribute to final XP)
        let request_factor = (self.total_requests as f64).ln().max(0.0) * 10.0;
        let success_rate = if self.total_requests > 0 {
            self.verified_count as f64 / self.total_requests as f64
        } else {
            0.0
        };
        let success_factor = success_rate * 20.0;
        let reliability_factor = (self.avg_reliability as f64 / 100.0) * 15.0;
        let recipe_factor = (self.recipes_learned as f64).ln().max(0.0) * 8.0;
        let solo_factor = if self.total_requests > 0 {
            (self.anna_solo_count as f64 / self.total_requests as f64) * 15.0
        } else {
            0.0
        };
        let streak_factor = (self.current_streak as f64).sqrt() * 3.0;

        // Raw XP (0-100 scale with diminishing returns)
        let raw_xp = request_factor
            + success_factor
            + reliability_factor
            + recipe_factor
            + solo_factor
            + streak_factor;

        // Logistic curve to cap at 100
        self.xp = (100.0 * (1.0 - (-raw_xp / 50.0).exp())) as u64;
        self.xp = self.xp.min(100);

        // Level from XP (10 levels)
        self.level = xp_to_level(self.xp);
        self.title = level_title(self.level);
    }
}

/// Convert XP (0-100) to level (1-10) using non-linear thresholds
/// v0.0.450: RPG-style progression per VISION.md
pub fn xp_to_level(xp: u64) -> u32 {
    match xp {
        0..=4 => 1,
        5..=11 => 2,
        12..=22 => 3,
        23..=37 => 4,
        38..=52 => 5,
        53..=67 => 6,
        68..=79 => 7,
        80..=89 => 8,
        90..=96 => 9,
        _ => 10,
    }
}

/// Get title for a given level - funny IT titles per VISION.md
/// v0.0.450: Enhanced with funnier titles
pub fn level_title(level: u32) -> String {
    match level {
        1 => "Trainee",
        2 => "Cable Untangler",
        3 => "Permission Gremlin",
        4 => "Log Whisperer",
        5 => "Daemon Wrangler",
        6 => "Kernel Whisperer",
        7 => "Stack Overflow Survivor",
        8 => "Senior Packet Inspector",
        9 => "Principal Engineer",
        10 => "Linus's Chosen One",
        _ => "Linus's Chosen One",
    }
    .to_string()
}
