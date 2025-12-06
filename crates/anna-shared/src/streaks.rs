//! Streak calculation module for stats/RPG system (v0.0.86).
//!
//! Calculates usage streaks and lucky team statistics.

use std::collections::HashMap;

/// Streak statistics
#[derive(Debug, Clone, Default)]
pub struct StreakStats {
    /// Current streak (consecutive days with activity)
    pub current_streak: u32,
    /// Best streak ever
    pub best_streak: u32,
    /// Unique days with activity
    pub active_days: u32,
}

/// Lucky team statistics
#[derive(Debug, Clone, Default)]
pub struct LuckyTeamStats {
    /// Team with highest success rate
    pub team: Option<String>,
    /// Success rate (0.0 - 1.0)
    pub rate: f32,
}

/// Calculate streak statistics from timestamps
pub fn calculate_streaks(timestamps: &[u64]) -> StreakStats {
    if timestamps.is_empty() {
        return StreakStats::default();
    }

    // Get unique days (as day number since epoch)
    let mut days: Vec<u64> = timestamps.iter().map(|ts| ts / 86400).collect();
    days.sort();
    days.dedup();

    let active_days = days.len() as u32;

    if days.is_empty() {
        return StreakStats {
            active_days,
            ..Default::default()
        };
    }

    // Calculate best streak
    let mut best_streak = 1u32;
    let mut streak = 1u32;

    for window in days.windows(2) {
        if window[1] == window[0] + 1 {
            streak += 1;
            best_streak = best_streak.max(streak);
        } else {
            streak = 1;
        }
    }

    // Check if streak is current (includes today or yesterday)
    let today = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);

    let last_day = *days.last().unwrap_or(&0);
    let current_streak = if last_day == today || last_day == today.saturating_sub(1) {
        // Count backwards from last_day
        let mut current = 1u32;
        for i in (0..days.len().saturating_sub(1)).rev() {
            if days[i] + 1 == days[i + 1] {
                current += 1;
            } else {
                break;
            }
        }
        current
    } else {
        0 // Streak broken
    };

    StreakStats {
        current_streak,
        best_streak,
        active_days,
    }
}

/// Team outcome record for lucky team calculation
pub struct TeamOutcome {
    pub team: String,
    pub success: bool,
}

/// Calculate lucky team (highest success rate with ≥3 cases)
pub fn calculate_lucky_team(outcomes: &[TeamOutcome]) -> LuckyTeamStats {
    let mut team_stats: HashMap<String, (u64, u64)> = HashMap::new();

    for outcome in outcomes {
        let entry = team_stats.entry(outcome.team.clone()).or_insert((0, 0));
        entry.0 += 1; // Total
        if outcome.success {
            entry.1 += 1; // Success
        }
    }

    // Find team with highest success rate (min 3 cases)
    let lucky = team_stats
        .iter()
        .filter(|(_, (total, _))| *total >= 3)
        .map(|(team, (total, success))| {
            let rate = *success as f32 / *total as f32;
            (team.clone(), rate)
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    match lucky {
        Some((team, rate)) => LuckyTeamStats {
            team: Some(team),
            rate,
        },
        None => LuckyTeamStats::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_streaks() {
        let stats = calculate_streaks(&[]);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.best_streak, 0);
        assert_eq!(stats.active_days, 0);
    }

    #[test]
    fn test_single_day_streak() {
        // Single day
        let today = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let stats = calculate_streaks(&[today]);
        assert_eq!(stats.active_days, 1);
        assert_eq!(stats.best_streak, 1);
    }

    #[test]
    fn test_consecutive_days() {
        let today = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let yesterday = today - 86400;
        let day_before = today - 86400 * 2;

        let stats = calculate_streaks(&[day_before, yesterday, today]);
        assert_eq!(stats.active_days, 3);
        assert_eq!(stats.best_streak, 3);
        assert_eq!(stats.current_streak, 3);
    }

    #[test]
    fn test_broken_streak() {
        // Old timestamps, streak should be 0
        let old = 1000000000u64; // ~2001
        let stats = calculate_streaks(&[old, old + 86400, old + 86400 * 2]);
        assert_eq!(stats.active_days, 3);
        assert_eq!(stats.best_streak, 3);
        assert_eq!(stats.current_streak, 0); // Streak is broken
    }

    #[test]
    fn test_lucky_team_empty() {
        let stats = calculate_lucky_team(&[]);
        assert!(stats.team.is_none());
    }

    #[test]
    fn test_lucky_team_needs_minimum() {
        // Only 2 cases, should not qualify
        let outcomes = vec![
            TeamOutcome { team: "A".to_string(), success: true },
            TeamOutcome { team: "A".to_string(), success: true },
        ];
        let stats = calculate_lucky_team(&outcomes);
        assert!(stats.team.is_none());
    }

    #[test]
    fn test_lucky_team_selection() {
        let outcomes = vec![
            TeamOutcome { team: "A".to_string(), success: true },
            TeamOutcome { team: "A".to_string(), success: true },
            TeamOutcome { team: "A".to_string(), success: true },
            TeamOutcome { team: "B".to_string(), success: true },
            TeamOutcome { team: "B".to_string(), success: false },
            TeamOutcome { team: "B".to_string(), success: false },
        ];
        let stats = calculate_lucky_team(&outcomes);
        assert_eq!(stats.team, Some("A".to_string()));
        assert_eq!(stats.rate, 1.0);
    }
}
