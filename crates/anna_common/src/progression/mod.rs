//! Progression Module v0.42.0
//!
//! RPG-style leveling system for Anna with XP, levels, titles, and statistics.
//!
//! ## Level System
//!
//! - Levels 0-99 with quadratic XP requirements
//! - Titles based on level bands
//! - XP earned from real work (reliability scores, complexity)
//!
//! ## Statistics
//!
//! - Global stats: questions answered, success rate, latency, iterations
//! - Per-question pattern tracking for improvement detection
//! - Skill usage stats integrated with SkillStore
//!
//! v0.42.0: Added XP penalties, pattern difficulty, strike tracking.

pub mod levels;
pub mod stats;
pub mod xp;

pub use levels::{AnnaProgression, Level, Title, TITLE_BANDS};
pub use stats::{
    GlobalStats, PatternStats, PerformanceSnapshot, QuestionPattern, StatsEngine, STATS_DIR,
};
pub use xp::{XpCalculator, XpGain, XpInput, XpPenalty, XP_CONFIG};
