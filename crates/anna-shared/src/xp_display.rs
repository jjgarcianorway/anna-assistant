//! XP/Level Display for RPG-style Progression.
//!
//! Per VISION.md:
//! - "XP from 0 to 100, non-linear RPG-style progression"
//! - "Funny titles based on level"
//!
//! Anna gains XP from successful queries, learning recipes, and achievements.

use crate::event_log::AggregatedEvents;
use crate::stats::GlobalStats;

/// XP thresholds for each level (non-linear progression)
const XP_THRESHOLDS: [u64; 10] = [
    0,    // Level 1 (0 XP)
    10,   // Level 2 (10 XP)
    25,   // Level 3 (25 XP)
    50,   // Level 4 (50 XP)
    100,  // Level 5 (100 XP)
    200,  // Level 6 (200 XP)
    400,  // Level 7 (400 XP)
    700,  // Level 8 (700 XP)
    1000, // Level 9 (1000 XP)
    1500, // Level 10 (1500 XP) - MAX
];

/// Funny titles for each level
const LEVEL_TITLES: [&str; 10] = [
    "Intern",                  // Level 1
    "Help Desk Trainee",       // Level 2
    "Junior Technician",       // Level 3
    "IT Support Specialist",   // Level 4
    "Senior Technician",       // Level 5
    "Systems Administrator",   // Level 6
    "Infrastructure Wizard",   // Level 7
    "Senior Architect",        // Level 8
    "IT Director",             // Level 9
    "Chief Technology Guru",   // Level 10
];

/// Short descriptions for each level
const LEVEL_DESCRIPTIONS: [&str; 10] = [
    "Just started, still figuring out the coffee machine",
    "Can now restart a service without panicking",
    "Knows the difference between RAM and storage",
    "Has memorized most man page flags",
    "Colleagues actually ask for help now",
    "Speaks fluent systemd and cron",
    "Can debug in production without breaking sweat",
    "Has seen things... many, many logs",
    "Makes the decisions that matter",
    "Achieved IT enlightenment",
];

/// Anna's XP and level information
#[derive(Debug, Clone)]
pub struct AnnaXP {
    /// Current XP total
    pub xp: u64,
    /// Current level (1-10)
    pub level: u8,
    /// XP needed for next level
    pub xp_to_next: u64,
    /// Progress to next level (0.0 - 1.0)
    pub progress: f32,
    /// Title for current level
    pub title: &'static str,
    /// Description for current level
    pub description: &'static str,
}

impl AnnaXP {
    /// Calculate XP and level from stats
    pub fn from_stats(stats: &GlobalStats) -> Self {
        let xp = calculate_xp(stats);
        Self::from_xp(xp)
    }

    /// Calculate XP and level from aggregated events
    pub fn from_events(events: &AggregatedEvents) -> Self {
        let xp = calculate_xp_from_events(events);
        Self::from_xp(xp)
    }

    /// Create from raw XP value
    pub fn from_xp(xp: u64) -> Self {
        let level = calculate_level(xp);
        let xp_to_next = xp_to_next_level(xp, level);
        let progress = level_progress(xp, level);
        let title = LEVEL_TITLES.get(level as usize - 1).unwrap_or(&"Unknown");
        let description = LEVEL_DESCRIPTIONS.get(level as usize - 1).unwrap_or(&"");

        Self {
            xp,
            level,
            xp_to_next,
            progress,
            title,
            description,
        }
    }

    /// Check if max level
    pub fn is_max_level(&self) -> bool {
        self.level >= 10
    }
}

/// Calculate XP from GlobalStats
fn calculate_xp(stats: &GlobalStats) -> u64 {
    let mut xp: u64 = 0;

    // Base XP from queries (1 XP per query)
    xp += stats.total_requests;

    // Bonus XP from fast path hits (efficient answers)
    xp += stats.fast_path_hits / 2;

    // Bonus XP from recipe hits (learned behavior)
    xp += stats.recipe_hits * 2;

    // Bonus XP from knowledge pack hits
    xp += stats.knowledge_pack_hits;

    // Bonus XP from successful clarifications
    xp += stats.clarifications_verified * 3;

    // Bonus XP from facts learned
    xp += stats.facts_learned * 5;

    xp
}

/// Calculate XP from AggregatedEvents
fn calculate_xp_from_events(events: &AggregatedEvents) -> u64 {
    let mut xp: u64 = 0;

    // Base XP from queries
    xp += events.total_requests;

    // Bonus XP from verified queries
    xp += events.verified_count / 2;

    // Bonus XP from streaks
    xp += events.best_streak as u64 * 5;

    // Bonus XP from recipes learned
    xp += events.recipes_learned * 10;

    // Penalty for failures (small)
    xp = xp.saturating_sub(events.failed_count / 5);

    xp
}

/// Calculate level from XP
fn calculate_level(xp: u64) -> u8 {
    for (i, &threshold) in XP_THRESHOLDS.iter().enumerate().rev() {
        if xp >= threshold {
            return (i + 1) as u8;
        }
    }
    1
}

/// Calculate XP needed to reach next level
fn xp_to_next_level(xp: u64, current_level: u8) -> u64 {
    if current_level >= 10 {
        return 0; // Max level
    }
    let next_threshold = XP_THRESHOLDS.get(current_level as usize).unwrap_or(&u64::MAX);
    next_threshold.saturating_sub(xp)
}

/// Calculate progress towards next level (0.0 - 1.0)
fn level_progress(xp: u64, current_level: u8) -> f32 {
    if current_level >= 10 {
        return 1.0;
    }
    let current_threshold = XP_THRESHOLDS.get(current_level as usize - 1).unwrap_or(&0);
    let next_threshold = XP_THRESHOLDS.get(current_level as usize).unwrap_or(&u64::MAX);

    let level_xp = xp.saturating_sub(*current_threshold);
    let level_range = next_threshold.saturating_sub(*current_threshold);

    if level_range == 0 {
        1.0
    } else {
        (level_xp as f32 / level_range as f32).min(1.0)
    }
}

/// Format XP display for status output
pub fn format_xp_display(anna_xp: &AnnaXP) -> String {
    let mut lines = vec![];

    lines.push(format!("Level {} - {}", anna_xp.level, anna_xp.title));
    lines.push(format!("  {}", anna_xp.description));
    lines.push(String::new());

    // XP progress bar
    let bar_width = 20;
    let filled = (anna_xp.progress * bar_width as f32) as usize;
    let empty = bar_width - filled;
    let bar = format!("[{}{}]", "=".repeat(filled), "-".repeat(empty));

    if anna_xp.is_max_level() {
        lines.push(format!("  XP: {} (MAX LEVEL)", anna_xp.xp));
        lines.push(format!("  {}", bar));
    } else {
        lines.push(format!("  XP: {} ({} to next level)", anna_xp.xp, anna_xp.xp_to_next));
        lines.push(format!("  {} {:.0}%", bar, anna_xp.progress * 100.0));
    }

    lines.join("\n")
}

/// Format compact XP line for greetings
pub fn format_xp_compact(anna_xp: &AnnaXP) -> String {
    if anna_xp.is_max_level() {
        format!("Lv.{} {} (MAX)", anna_xp.level, anna_xp.title)
    } else {
        format!(
            "Lv.{} {} ({} XP, {}% to next)",
            anna_xp.level,
            anna_xp.title,
            anna_xp.xp,
            (anna_xp.progress * 100.0) as u8
        )
    }
}

/// Check if query is asking about XP/level
pub fn is_xp_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    matches_any(&lower, &[
        "my level", "my xp", "xp", "level", "experience",
        "anna level", "anna xp", "progression", "rank"
    ])
}

fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_calculation() {
        assert_eq!(calculate_level(0), 1);
        assert_eq!(calculate_level(9), 1);
        assert_eq!(calculate_level(10), 2);
        assert_eq!(calculate_level(25), 3);
        assert_eq!(calculate_level(100), 5);
        assert_eq!(calculate_level(1500), 10);
        assert_eq!(calculate_level(5000), 10);
    }

    #[test]
    fn test_xp_to_next() {
        assert_eq!(xp_to_next_level(0, 1), 10);
        assert_eq!(xp_to_next_level(5, 1), 5);
        assert_eq!(xp_to_next_level(1500, 10), 0);
    }

    #[test]
    fn test_level_progress() {
        assert_eq!(level_progress(0, 1), 0.0);
        assert_eq!(level_progress(5, 1), 0.5);
        assert_eq!(level_progress(10, 2), 0.0);
    }

    #[test]
    fn test_anna_xp_from_xp() {
        let anna = AnnaXP::from_xp(50);
        assert_eq!(anna.level, 4);
        assert_eq!(anna.title, "IT Support Specialist");
        assert!(!anna.is_max_level());
    }

    #[test]
    fn test_max_level() {
        let anna = AnnaXP::from_xp(2000);
        assert_eq!(anna.level, 10);
        assert!(anna.is_max_level());
        assert_eq!(anna.xp_to_next, 0);
    }

    #[test]
    fn test_format_xp_display() {
        let anna = AnnaXP::from_xp(50);
        let display = format_xp_display(&anna);
        assert!(display.contains("Level 4"));
        assert!(display.contains("IT Support Specialist"));
        assert!(display.contains("XP"));
    }

    #[test]
    fn test_format_xp_compact() {
        let anna = AnnaXP::from_xp(50);
        let compact = format_xp_compact(&anna);
        assert!(compact.contains("Lv.4"));
        assert!(compact.contains("50 XP"));
    }

    #[test]
    fn test_is_xp_query() {
        assert!(is_xp_query("what is my level"));
        assert!(is_xp_query("show xp"));
        assert!(is_xp_query("anna level"));
        assert!(!is_xp_query("disk space"));
    }
}
