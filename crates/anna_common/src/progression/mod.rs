//! Progression Module v0.42.0
//!
//! DEPRECATED: See `xp_events.rs` and `xp_track.rs` for canonical XP system.
//!
//! RPG-style leveling system for Anna with XP, levels, titles, and statistics.
//!
//! ## Architecture Note (v1.0.0)
//!
//! This module is DEPRECATED. The canonical XP system uses:
//! - `xp_events.rs`: XP event types and base values (CANONICAL)
//! - `xp_track.rs`: XpStore, XpTrack for tracking agent progress
//! - `rpg_display.rs`: Display formatting, title colors, progress bars
//!
//! This module has different title bands and XP curves that may cause
//! inconsistency. New code should NOT use this module.
//!
//! ## Level System (Legacy)
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
//! v1.0.0: Deprecated in favor of xp_events.rs and xp_track.rs.

pub mod levels;
pub mod stats;
pub mod xp;

pub use levels::{AnnaProgression, Level, Title, TITLE_BANDS};
pub use stats::{
    GlobalStats, PatternStats, PerformanceSnapshot, QuestionPattern, StatsEngine, STATS_DIR,
};
pub use xp::{XpCalculator, XpGain, XpInput, XpPenalty, XP_CONFIG};
