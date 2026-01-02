//! Core types for fun statistics (v0.0.479).

use crate::event_log::AggregatedEvents;

/// Fun statistics derived from aggregated events
#[derive(Debug, Clone, Default)]
pub struct FunStats {
    /// Most consulted team
    pub most_consulted_team: Option<String>,
    /// Most consulted team count
    pub most_consulted_count: u64,
    /// Topic most asked about
    pub top_topic: Option<String>,
    /// Top topic count
    pub top_topic_count: u64,
    /// Number of repeated questions
    pub repeated_questions: u64,
    /// Longest reply duration (ms, proxy for length)
    pub longest_reply_ms: u64,
    /// Shortest reply duration (ms, proxy for length)
    pub shortest_reply_ms: u64,
    /// Installation date (first event timestamp)
    pub installation_date: u64,
    /// Days since installation
    pub days_active: u64,
    /// Anna solo successes (no escalation)
    pub anna_solo_count: u64,
    /// Solo percentage
    pub anna_solo_pct: f32,
    /// Lucky team (highest success rate)
    pub lucky_team: Option<String>,
    /// Lucky team rate
    pub lucky_team_rate: f32,
    /// Current streak (days)
    pub current_streak: u32,
    /// Best streak ever
    pub best_streak: u32,
    /// Total requests
    pub total_requests: u64,
    /// Recipes learned
    pub recipes_learned: u64,
}

impl FunStats {
    /// Create fun stats from aggregated events
    pub fn from_aggregated(agg: &AggregatedEvents) -> Self {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let days_active = if agg.first_event_ts > 0 {
            (now_secs - agg.first_event_ts) / 86400
        } else {
            0
        };

        let solo_pct = if agg.total_requests > 0 {
            (agg.anna_solo_count as f32 / agg.total_requests as f32) * 100.0
        } else {
            0.0
        };

        // Find most consulted team from by_team
        let (most_team, most_count) = agg
            .by_team
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(team, count)| (Some(team.clone()), *count))
            .unwrap_or((None, 0));

        Self {
            most_consulted_team: most_team,
            most_consulted_count: most_count,
            top_topic: agg.most_consulted_team.clone(), // reuse as topic proxy
            top_topic_count: most_count,
            repeated_questions: 0, // TODO: track in aggregation
            longest_reply_ms: agg.longest_reply_chars,
            shortest_reply_ms: agg.shortest_reply_chars,
            installation_date: agg.first_event_ts,
            days_active,
            anna_solo_count: agg.anna_solo_count,
            anna_solo_pct: solo_pct,
            lucky_team: agg.lucky_team.clone(),
            lucky_team_rate: agg.lucky_team_rate,
            current_streak: agg.current_streak,
            best_streak: agg.best_streak,
            total_requests: agg.total_requests,
            recipes_learned: agg.recipes_learned,
        }
    }
}
