//! Achievement badges for Anna stats/RPG system (v0.0.90).
//!
//! Tracks user milestones and unlocks achievement badges.
//! Achievements add personality and gamification to the stats display.

use crate::event_log::AggregatedEvents;
use serde::{Deserialize, Serialize};

/// Achievement badge with emoji and description
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Achievement {
    /// Unique identifier
    pub id: &'static str,
    /// Display emoji
    pub emoji: &'static str,
    /// Short name
    pub name: &'static str,
    /// Description of how to earn it
    pub description: &'static str,
    /// Whether it's been unlocked
    pub unlocked: bool,
}

impl Achievement {
    const fn new(id: &'static str, emoji: &'static str, name: &'static str, description: &'static str) -> Self {
        Self { id, emoji, name, description, unlocked: false }
    }

    fn unlock(mut self) -> Self {
        self.unlocked = true;
        self
    }
}

/// All available achievements
pub fn all_achievements() -> Vec<Achievement> {
    vec![
        // Milestone achievements
        Achievement::new("first_query", "🎯", "First Contact", "Complete your first query"),
        Achievement::new("ten_queries", "🔟", "Getting Started", "Complete 10 queries"),
        Achievement::new("fifty_queries", "⭐", "Regular User", "Complete 50 queries"),
        Achievement::new("hundred_queries", "💯", "Power User", "Complete 100 queries"),
        Achievement::new("five_hundred", "🏆", "Anna Expert", "Complete 500 queries"),

        // Streak achievements
        Achievement::new("streak_3", "🔥", "On Fire", "Maintain a 3-day streak"),
        Achievement::new("streak_7", "📅", "Week Warrior", "Maintain a 7-day streak"),
        Achievement::new("streak_30", "🗓️", "Monthly Master", "Maintain a 30-day streak"),

        // Quality achievements
        Achievement::new("perfect_10", "💎", "Perfect 10", "Get 10 queries with 90%+ reliability"),
        Achievement::new("no_failures", "✅", "Flawless", "Complete 20+ queries with no failures"),
        Achievement::new("speed_demon", "⚡", "Speed Demon", "Get an answer in under 500ms"),

        // Team achievements
        Achievement::new("all_teams", "🌐", "Well-Rounded", "Consult all 8 teams at least once"),
        Achievement::new("storage_fan", "💾", "Storage Savvy", "Ask 20+ storage questions"),
        Achievement::new("network_guru", "🌐", "Network Guru", "Ask 20+ network questions"),
        Achievement::new("perf_junkie", "📊", "Performance Junkie", "Ask 20+ performance questions"),

        // Special achievements
        Achievement::new("night_owl", "🦉", "Night Owl", "Use Anna after midnight"),
        Achievement::new("early_bird", "🐦", "Early Bird", "Use Anna before 6 AM"),
        Achievement::new("recipe_master", "📖", "Recipe Master", "Learn 5+ recipes"),
        Achievement::new("escalation_free", "🎖️", "Solo Artist", "Complete 10+ queries without escalation"),

        // Tenure achievements
        Achievement::new("week_old", "📆", "One Week In", "Use Anna for a week"),
        Achievement::new("month_old", "🗓️", "Month Veteran", "Use Anna for a month"),
    ]
}

/// Check which achievements are unlocked based on aggregated stats
pub fn check_achievements(agg: &AggregatedEvents) -> Vec<Achievement> {
    let mut achievements = all_achievements();

    for ach in &mut achievements {
        ach.unlocked = is_unlocked(ach.id, agg);
    }

    achievements
}

/// Get only unlocked achievements
pub fn unlocked_achievements(agg: &AggregatedEvents) -> Vec<Achievement> {
    check_achievements(agg).into_iter().filter(|a| a.unlocked).collect()
}

/// Get newly unlockable achievements (for notifications)
pub fn newly_unlocked(old: &AggregatedEvents, new: &AggregatedEvents) -> Vec<Achievement> {
    let old_unlocked: Vec<_> = unlocked_achievements(old).iter().map(|a| a.id).collect();
    unlocked_achievements(new)
        .into_iter()
        .filter(|a| !old_unlocked.contains(&a.id))
        .collect()
}

/// Check if a specific achievement is unlocked
fn is_unlocked(id: &str, agg: &AggregatedEvents) -> bool {
    match id {
        // Milestone achievements
        "first_query" => agg.total_requests >= 1,
        "ten_queries" => agg.total_requests >= 10,
        "fifty_queries" => agg.total_requests >= 50,
        "hundred_queries" => agg.total_requests >= 100,
        "five_hundred" => agg.total_requests >= 500,

        // Streak achievements
        "streak_3" => agg.best_streak >= 3,
        "streak_7" => agg.best_streak >= 7,
        "streak_30" => agg.best_streak >= 30,

        // Quality achievements
        "perfect_10" => count_high_reliability(agg) >= 10,
        "no_failures" => agg.total_requests >= 20 && agg.failed_count == 0,
        "speed_demon" => agg.min_duration_ms > 0 && agg.min_duration_ms < 500,

        // Team achievements
        "all_teams" => agg.by_team.len() >= 8,
        "storage_fan" => team_count(agg, "storage") >= 20,
        "network_guru" => team_count(agg, "network") >= 20,
        "perf_junkie" => team_count(agg, "performance") >= 20,

        // Special achievements
        "night_owl" => check_hour_range(agg, 0, 4),
        "early_bird" => check_hour_range(agg, 4, 6),
        "recipe_master" => agg.recipes_learned >= 5,
        "escalation_free" => agg.total_requests >= 10 && agg.escalation_count == 0,

        // Tenure achievements
        "week_old" => tenure_days(agg) >= 7,
        "month_old" => tenure_days(agg) >= 30,

        _ => false,
    }
}

/// Count queries with 90%+ reliability (estimate from avg)
fn count_high_reliability(agg: &AggregatedEvents) -> u64 {
    // Estimate: if avg is 90+, assume good portion are high reliability
    if agg.avg_reliability >= 90.0 {
        agg.verified_count
    } else if agg.avg_reliability >= 80.0 {
        agg.verified_count / 2
    } else {
        agg.verified_count / 4
    }
}

/// Get count for a specific team
fn team_count(agg: &AggregatedEvents, team: &str) -> u64 {
    agg.by_team.get(team).copied().unwrap_or(0)
}

/// Check if any query was made in a specific hour range
fn check_hour_range(agg: &AggregatedEvents, start_hour: u8, end_hour: u8) -> bool {
    // We don't have per-event hour data in aggregated stats
    // Check if last event was in range (rough approximation)
    if agg.last_event_ts == 0 {
        return false;
    }
    let hour = ((agg.last_event_ts / 3600) % 24) as u8;
    hour >= start_hour && hour < end_hour
}

/// Calculate tenure in days
fn tenure_days(agg: &AggregatedEvents) -> u64 {
    if agg.first_event_ts == 0 || agg.last_event_ts == 0 {
        return 0;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (now.saturating_sub(agg.first_event_ts)) / 86400
}

/// Format achievements for display
pub fn format_achievements(achievements: &[Achievement], max_display: usize) -> String {
    let unlocked: Vec<_> = achievements.iter().filter(|a| a.unlocked).collect();

    if unlocked.is_empty() {
        return String::new();
    }

    let display: Vec<_> = unlocked.iter().take(max_display).collect();
    let emojis: String = display.iter().map(|a| a.emoji).collect::<Vec<_>>().join(" ");

    if unlocked.len() > max_display {
        format!("{} +{} more", emojis, unlocked.len() - max_display)
    } else {
        emojis
    }
}

/// Format a single achievement for notification
pub fn format_achievement_unlock(ach: &Achievement) -> String {
    format!("{} Achievement unlocked: {} - {}", ach.emoji, ach.name, ach.description)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_agg(requests: u64, verified: u64, streak: u32) -> AggregatedEvents {
        AggregatedEvents {
            total_requests: requests,
            verified_count: verified,
            failed_count: requests - verified,
            best_streak: streak,
            current_streak: streak,
            avg_reliability: if requests > 0 { 85.0 } else { 0.0 },
            min_duration_ms: 300,
            max_duration_ms: 5000,
            first_event_ts: 1700000000,
            last_event_ts: 1700100000,
            by_team: HashMap::new(),
            ..Default::default()
        }
    }

    #[test]
    fn test_first_query_achievement() {
        let agg = mock_agg(1, 1, 1);
        let achievements = check_achievements(&agg);
        let first = achievements.iter().find(|a| a.id == "first_query").unwrap();
        assert!(first.unlocked);
    }

    #[test]
    fn test_streak_achievement() {
        let agg = mock_agg(10, 10, 7);
        let achievements = check_achievements(&agg);

        let streak_3 = achievements.iter().find(|a| a.id == "streak_3").unwrap();
        let streak_7 = achievements.iter().find(|a| a.id == "streak_7").unwrap();
        let streak_30 = achievements.iter().find(|a| a.id == "streak_30").unwrap();

        assert!(streak_3.unlocked);
        assert!(streak_7.unlocked);
        assert!(!streak_30.unlocked);
    }

    #[test]
    fn test_speed_demon() {
        let mut agg = mock_agg(5, 5, 1);
        agg.min_duration_ms = 400;

        let achievements = check_achievements(&agg);
        let speed = achievements.iter().find(|a| a.id == "speed_demon").unwrap();
        assert!(speed.unlocked);
    }

    #[test]
    fn test_format_achievements() {
        let achievements = vec![
            Achievement::new("a", "🎯", "Test", "Test").unlock(),
            Achievement::new("b", "⭐", "Test2", "Test2").unlock(),
        ];

        let formatted = format_achievements(&achievements, 5);
        assert!(formatted.contains("🎯"));
        assert!(formatted.contains("⭐"));
    }

    #[test]
    fn test_unlocked_only() {
        let agg = mock_agg(100, 100, 5);
        let unlocked = unlocked_achievements(&agg);

        // Should have first_query, ten_queries, fifty_queries, hundred_queries, streak_3
        assert!(unlocked.len() >= 4);
        assert!(unlocked.iter().any(|a| a.id == "hundred_queries"));
    }
}
