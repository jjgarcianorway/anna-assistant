//! Level System v0.40.1
//!
//! RPG-style levels 0-99 with quadratic XP curve and title bands.
//!
//! ## XP Curve
//!
//! XP required for level L: base_xp * L^growth_factor
//! - base_xp = 100 (XP to reach level 1)
//! - growth_factor = 1.8 (nearly quadratic growth)
//!
//! This means:
//! - Level 1: 100 XP
//! - Level 10: ~6,310 XP total
//! - Level 50: ~158,000 XP total
//! - Level 99: ~600,000 XP total

use serde::{Deserialize, Serialize};

/// Title bands mapping level ranges to progression titles
pub const TITLE_BANDS: &[(u8, u8, &str)] = &[
    (0, 4, "Intern"),
    (5, 14, "Junior Sysadmin"),
    (15, 29, "Journeyman Troubleshooter"),
    (30, 49, "Senior Systems Whisperer"),
    (50, 69, "Operations Archmage"),
    (70, 89, "Infrastructure Oracle"),
    (90, 99, "God of Knowledge"),
];

/// Anna's current level (0-99)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Level(pub u8);

impl Level {
    /// Create a new level (clamped to 0-99)
    pub fn new(level: u8) -> Self {
        Self(level.min(99))
    }

    /// Get the raw level number
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Get the title for this level
    pub fn title(&self) -> Title {
        Title::from_level(self.0)
    }

    /// Calculate XP required to reach this level from 0
    ///
    /// Formula: sum of (base_xp * i^growth) for i in 1..=level
    /// Simplified: base_xp * sum(i^growth for i in 1..=level)
    pub fn xp_required(&self) -> u64 {
        xp_for_level(self.0)
    }

    /// Calculate XP needed to reach the next level
    pub fn xp_to_next(&self) -> u64 {
        if self.0 >= 99 {
            return 0; // Max level
        }
        xp_for_level(self.0 + 1) - xp_for_level(self.0)
    }

    /// Calculate level from total XP
    pub fn from_xp(total_xp: u64) -> Self {
        for level in (0..=99u8).rev() {
            if total_xp >= xp_for_level(level) {
                return Self(level);
            }
        }
        Self(0)
    }

    /// Progress to next level as percentage (0.0 - 1.0)
    pub fn progress_to_next(&self, current_xp: u64) -> f64 {
        if self.0 >= 99 {
            return 1.0; // Max level
        }

        let current_level_xp = xp_for_level(self.0);
        let next_level_xp = xp_for_level(self.0 + 1);
        let level_range = next_level_xp - current_level_xp;

        if level_range == 0 {
            return 1.0;
        }

        let progress = current_xp.saturating_sub(current_level_xp) as f64 / level_range as f64;
        progress.clamp(0.0, 1.0)
    }
}

impl Default for Level {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Title corresponding to a level band
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Title(String);

impl Title {
    /// Get title from level
    pub fn from_level(level: u8) -> Self {
        for &(min, max, title) in TITLE_BANDS {
            if level >= min && level <= max {
                return Self(title.to_string());
            }
        }
        Self("Unknown".to_string())
    }

    /// Get the title string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// XP configuration constants
const BASE_XP: f64 = 100.0;
const GROWTH_FACTOR: f64 = 1.8;

/// Calculate total XP required to reach a level
///
/// Uses cumulative sum: sum(base * i^growth for i in 1..=level)
fn xp_for_level(level: u8) -> u64 {
    if level == 0 {
        return 0;
    }

    let mut total = 0.0;
    for i in 1..=level as u32 {
        total += BASE_XP * (i as f64).powf(GROWTH_FACTOR);
    }
    total as u64
}

/// Anna's progression state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaProgression {
    /// Total XP accumulated
    pub total_xp: u64,
    /// Current level (derived from XP)
    pub level: Level,
    /// Current title (derived from level)
    pub title: Title,
}

impl AnnaProgression {
    /// Create new progression state
    pub fn new() -> Self {
        Self {
            total_xp: 0,
            level: Level::new(0),
            title: Title::from_level(0),
        }
    }

    /// Create from existing XP
    pub fn from_xp(total_xp: u64) -> Self {
        let level = Level::from_xp(total_xp);
        let title = level.title();
        Self {
            total_xp,
            level,
            title,
        }
    }

    /// Add XP and update level/title
    pub fn add_xp(&mut self, xp: u64) {
        self.total_xp = self.total_xp.saturating_add(xp);
        self.level = Level::from_xp(self.total_xp);
        self.title = self.level.title();
    }

    /// Get XP progress to next level
    pub fn xp_to_next_level(&self) -> u64 {
        if self.level.0 >= 99 {
            return 0;
        }
        let next_level_xp = xp_for_level(self.level.0 + 1);
        next_level_xp.saturating_sub(self.total_xp)
    }

    /// Get progress percentage to next level (0-100)
    pub fn progress_percent(&self) -> u8 {
        (self.level.progress_to_next(self.total_xp) * 100.0) as u8
    }
}

impl Default for AnnaProgression {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_creation() {
        assert_eq!(Level::new(0).value(), 0);
        assert_eq!(Level::new(50).value(), 50);
        assert_eq!(Level::new(99).value(), 99);
        assert_eq!(Level::new(100).value(), 99); // Clamped
        assert_eq!(Level::new(255).value(), 99); // Clamped
    }

    #[test]
    fn test_title_bands() {
        assert_eq!(Title::from_level(0).as_str(), "Intern");
        assert_eq!(Title::from_level(4).as_str(), "Intern");
        assert_eq!(Title::from_level(5).as_str(), "Junior Sysadmin");
        assert_eq!(Title::from_level(14).as_str(), "Junior Sysadmin");
        assert_eq!(Title::from_level(15).as_str(), "Journeyman Troubleshooter");
        assert_eq!(Title::from_level(30).as_str(), "Senior Systems Whisperer");
        assert_eq!(Title::from_level(50).as_str(), "Operations Archmage");
        assert_eq!(Title::from_level(70).as_str(), "Infrastructure Oracle");
        assert_eq!(Title::from_level(90).as_str(), "God of Knowledge");
        assert_eq!(Title::from_level(99).as_str(), "God of Knowledge");
    }

    #[test]
    fn test_xp_curve_progression() {
        // Level 0 requires 0 XP
        assert_eq!(xp_for_level(0), 0);

        // Level 1 requires base XP
        let lvl1_xp = xp_for_level(1);
        assert_eq!(lvl1_xp, 100);

        // XP increases with level
        let lvl5_xp = xp_for_level(5);
        let lvl10_xp = xp_for_level(10);
        assert!(lvl5_xp > lvl1_xp);
        assert!(lvl10_xp > lvl5_xp);

        // Higher levels need more XP per level
        let lvl49_xp = xp_for_level(49);
        let lvl50_xp = xp_for_level(50);
        let lvl51_xp = xp_for_level(51);

        let jump_to_50 = lvl50_xp - lvl49_xp;
        let jump_to_51 = lvl51_xp - lvl50_xp;
        assert!(jump_to_51 > jump_to_50);
    }

    #[test]
    fn test_level_from_xp() {
        assert_eq!(Level::from_xp(0).value(), 0);
        assert_eq!(Level::from_xp(99).value(), 0);
        assert_eq!(Level::from_xp(100).value(), 1);
        assert_eq!(Level::from_xp(101).value(), 1);

        // At exactly level 10's XP threshold
        let lvl10_xp = xp_for_level(10);
        assert_eq!(Level::from_xp(lvl10_xp).value(), 10);
        assert_eq!(Level::from_xp(lvl10_xp - 1).value(), 9);
    }

    #[test]
    fn test_progression_add_xp() {
        let mut prog = AnnaProgression::new();
        assert_eq!(prog.level.value(), 0);
        assert_eq!(prog.title.as_str(), "Intern");

        // Add enough XP to reach level 1
        prog.add_xp(100);
        assert_eq!(prog.level.value(), 1);
        assert_eq!(prog.title.as_str(), "Intern");

        // Add more XP to reach level 5 (Junior Sysadmin)
        let lvl5_xp = xp_for_level(5);
        prog.add_xp(lvl5_xp);
        assert!(prog.level.value() >= 5);
        assert_eq!(prog.title.as_str(), "Junior Sysadmin");
    }

    #[test]
    fn test_progress_to_next() {
        let lvl5_xp = xp_for_level(5);
        let lvl6_xp = xp_for_level(6);
        let midpoint = (lvl5_xp + lvl6_xp) / 2;

        let prog = AnnaProgression::from_xp(midpoint);
        let pct = prog.progress_percent();

        // Should be roughly 50%
        assert!(pct >= 40 && pct <= 60, "Progress was {}%", pct);
    }

    #[test]
    fn test_max_level() {
        let lvl99_xp = xp_for_level(99);
        let prog = AnnaProgression::from_xp(lvl99_xp);

        assert_eq!(prog.level.value(), 99);
        assert_eq!(prog.title.as_str(), "God of Knowledge");
        assert_eq!(prog.xp_to_next_level(), 0);
        assert_eq!(prog.progress_percent(), 100);
    }

    #[test]
    fn test_xp_thresholds_reasonable() {
        // Verify XP thresholds are reasonable for gameplay
        let lvl1 = xp_for_level(1);
        let lvl10 = xp_for_level(10);
        let lvl50 = xp_for_level(50);
        let lvl99 = xp_for_level(99);

        // Early levels should be achievable quickly
        assert!(lvl1 <= 200, "Level 1 XP: {}", lvl1);
        assert!(lvl10 <= 30_000, "Level 10 XP: {}", lvl10);

        // Mid levels require significant work
        assert!(lvl50 >= 50_000, "Level 50 XP: {}", lvl50);

        // Max level requires substantial effort
        assert!(lvl99 >= 200_000, "Level 99 XP: {}", lvl99);

        // Print for debugging
        println!("XP Thresholds:");
        println!("  Level 1:  {:>10}", lvl1);
        println!("  Level 10: {:>10}", lvl10);
        println!("  Level 50: {:>10}", lvl50);
        println!("  Level 99: {:>10}", lvl99);
    }
}
