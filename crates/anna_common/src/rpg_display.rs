//! RPG Display Module v1.0.0
//!
//! Handles RPG-style presentation of Anna, Junior, and Senior stats.
//! Provides mood text, progress bars, and encouraging but honest messages.
//!
//! ## Architecture Note (v1.0.0)
//!
//! This module provides DISPLAY and FORMATTING for the XP system:
//! - `get_rpg_title()`: Title from level
//! - `get_title_color()`: Color code for title
//! - `TrustLevel`: Trust classification and colors
//! - `ReliabilityScale`: Scaling factors for XP/trust based on reliability
//! - `progress_bar()`: Progress bar generation
//!
//! For XP EVENT TYPES and BASE VALUES, see `xp_events.rs` (canonical source).
//! For XP TRACKING/STATE, see `xp_track.rs`.

use serde::{Deserialize, Serialize};

// ============================================================================
// Title Bands - Expanded RPG Titles
// ============================================================================

/// RPG title bands by level
/// Level 1-4: Intern
/// Level 5-9: Junior Specialist
/// Level 10-19: Specialist
/// Level 20-34: Senior Specialist
/// Level 35-49: Lead
/// Level 50-69: Principal
/// Level 70-89: Archon
/// Level 90-99: Mythic
pub const RPG_TITLE_BANDS: &[(u8, u8, &str)] = &[
    (1, 4, "Intern"),
    (5, 9, "Junior Specialist"),
    (10, 19, "Specialist"),
    (20, 34, "Senior Specialist"),
    (35, 49, "Lead"),
    (50, 69, "Principal"),
    (70, 89, "Archon"),
    (90, 99, "Mythic"),
];

/// Get RPG title for a level
pub fn get_rpg_title(level: u8) -> &'static str {
    for (min, max, title) in RPG_TITLE_BANDS {
        if level >= *min && level <= *max {
            return title;
        }
    }
    "Unknown"
}

/// Get title color code based on level (for display)
/// Returns: "dim", "cyan", "bright_cyan", "green", "yellow", "magenta", "red"
pub fn get_title_color(level: u8) -> &'static str {
    if level < 5 {
        "dim"
    } else if level < 10 {
        "cyan"
    } else if level < 20 {
        "bright_cyan"
    } else if level < 35 {
        "green"
    } else if level < 50 {
        "yellow"
    } else if level < 70 {
        "bright_yellow"
    } else if level < 90 {
        "magenta"
    } else {
        "red"
    }
}

// ============================================================================
// Trust Labels
// ============================================================================

/// Trust level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    Low,
    Normal,
    High,
}

impl TrustLevel {
    /// Classify trust value into level
    pub fn from_trust(trust: f32) -> Self {
        if trust < 0.4 {
            TrustLevel::Low
        } else if trust > 0.7 {
            TrustLevel::High
        } else {
            TrustLevel::Normal
        }
    }

    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            TrustLevel::Low => "low",
            TrustLevel::Normal => "normal",
            TrustLevel::High => "high",
        }
    }

    /// Get color for display
    pub fn color(&self) -> &'static str {
        match self {
            TrustLevel::Low => "red",
            TrustLevel::Normal => "yellow",
            TrustLevel::High => "green",
        }
    }
}

// ============================================================================
// Mood Text - Honest but Encouraging
// ============================================================================

/// Get mood text based on success rate
/// This provides honest context without fake optimism
pub fn get_mood_text(success_rate: f64) -> &'static str {
    if success_rate < 30.0 {
        "Anna is still learning. Most attempts need more work."
    } else if success_rate < 60.0 {
        "Anna is improving. Many answers are already reliable."
    } else if success_rate < 80.0 {
        "Anna is performing well. Most answers are solid."
    } else {
        "Anna is in excellent form. Answers are consistently reliable."
    }
}

/// Get streak text for display
pub fn get_streak_text(streak_good: u32, streak_bad: u32) -> String {
    if streak_good > 0 {
        if streak_good == 1 {
            "1 successful answer".to_string()
        } else {
            format!("{} successful answers in a row", streak_good)
        }
    } else if streak_bad > 0 {
        if streak_bad == 1 {
            "1 recent failure".to_string()
        } else {
            format!("{} failures to recover from", streak_bad)
        }
    } else {
        "starting fresh".to_string()
    }
}

// ============================================================================
// XP Progress Bar Generation
// ============================================================================

/// Generate a progress bar string
/// width: total characters for the bar (excluding brackets)
/// progress: percentage (0-100)
pub fn progress_bar(progress: usize, width: usize) -> String {
    let filled = (progress * width) / 100;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "=".repeat(filled), "-".repeat(empty))
}

/// Generate a progress bar with percentage text
pub fn progress_bar_with_text(progress: usize, width: usize) -> String {
    let bar = progress_bar(progress, width);
    format!("{} {}%", bar, progress)
}

// ============================================================================
// Reliability-Based XP Scaling
// ============================================================================

/// Scale factor based on answer reliability
#[derive(Debug, Clone, Copy)]
pub struct ReliabilityScale {
    /// XP multiplier (0.0 - 1.0)
    pub xp_multiplier: f32,
    /// Trust delta multiplier
    pub trust_multiplier: f32,
    /// Quality label
    pub quality: &'static str,
}

impl ReliabilityScale {
    /// Get scaling factors based on reliability score
    pub fn from_reliability(reliability: f64) -> Self {
        if reliability >= 0.9 {
            // Green - full rewards
            ReliabilityScale {
                xp_multiplier: 1.0,
                trust_multiplier: 1.0,
                quality: "Green",
            }
        } else if reliability >= 0.7 {
            // Yellow - reduced rewards
            ReliabilityScale {
                xp_multiplier: 0.75,
                trust_multiplier: 0.5,
                quality: "Yellow",
            }
        } else if reliability >= 0.5 {
            // Orange - minimal rewards
            ReliabilityScale {
                xp_multiplier: 0.5,
                trust_multiplier: 0.25,
                quality: "Orange",
            }
        } else if reliability > 0.0 {
            // Red but honest - tiny XP for honesty
            ReliabilityScale {
                xp_multiplier: 0.25,
                trust_multiplier: 0.0,
                quality: "Red",
            }
        } else {
            // No answer or failure
            ReliabilityScale {
                xp_multiplier: 0.0,
                trust_multiplier: -0.5, // Trust penalty
                quality: "None",
            }
        }
    }

    /// Apply scaling to base XP
    pub fn scale_xp(&self, base_xp: u64) -> u64 {
        ((base_xp as f32) * self.xp_multiplier).round() as u64
    }

    /// Apply scaling to trust delta
    pub fn scale_trust(&self, base_trust_delta: f32) -> f32 {
        if self.trust_multiplier < 0.0 {
            // Trust penalty (negative multiplier means penalty)
            self.trust_multiplier.abs() * base_trust_delta
        } else {
            base_trust_delta * self.trust_multiplier
        }
    }
}

// ============================================================================
// XP Event Types for All Agents
// ============================================================================

/// Anna XP event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnaXpEvent {
    /// Self-solved via Brain (no LLM)
    SelfSolveOk,
    /// LLM orchestration succeeded
    LlmOrchestrationOk,
    /// Partial answer with low reliability but honest
    PartialAnswer,
    /// Timeout or pipeline failure
    TimeoutOrFailure,
    /// Correct refusal (no evidence available)
    RefusalCorrect,
}

impl AnnaXpEvent {
    /// Base XP for event (before reliability scaling)
    pub fn base_xp(&self) -> u64 {
        match self {
            AnnaXpEvent::SelfSolveOk => 5,
            AnnaXpEvent::LlmOrchestrationOk => 4,
            AnnaXpEvent::PartialAnswer => 1,
            AnnaXpEvent::TimeoutOrFailure => 0,
            AnnaXpEvent::RefusalCorrect => 2,
        }
    }

    /// Base trust delta
    pub fn base_trust(&self) -> f32 {
        match self {
            AnnaXpEvent::SelfSolveOk => 0.02,
            AnnaXpEvent::LlmOrchestrationOk => 0.015,
            AnnaXpEvent::PartialAnswer => 0.005,
            AnnaXpEvent::TimeoutOrFailure => -0.05,
            AnnaXpEvent::RefusalCorrect => 0.01,
        }
    }

    /// Is this a positive event?
    pub fn is_positive(&self) -> bool {
        !matches!(self, AnnaXpEvent::TimeoutOrFailure)
    }

    /// Event label for XP log
    pub fn label(&self) -> &'static str {
        match self {
            AnnaXpEvent::SelfSolveOk => "self_solve_ok",
            AnnaXpEvent::LlmOrchestrationOk => "llm_orchestration_ok",
            AnnaXpEvent::PartialAnswer => "partial_answer",
            AnnaXpEvent::TimeoutOrFailure => "timeout_or_failure",
            AnnaXpEvent::RefusalCorrect => "refusal_correct",
        }
    }
}

/// Junior XP event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JuniorXpEvent {
    /// Plan led to successful answer
    PlanGood,
    /// Plan was patched by Senior
    PlanPatchedBySenior,
    /// Plan was clearly bad/unusable
    PlanBad,
    /// Timeout during planning
    Timeout,
}

impl JuniorXpEvent {
    pub fn base_xp(&self) -> u64 {
        match self {
            JuniorXpEvent::PlanGood => 3,
            JuniorXpEvent::PlanPatchedBySenior => 1,
            JuniorXpEvent::PlanBad => 0,
            JuniorXpEvent::Timeout => 0,
        }
    }

    pub fn base_trust(&self) -> f32 {
        match self {
            JuniorXpEvent::PlanGood => 0.02,
            JuniorXpEvent::PlanPatchedBySenior => -0.01,
            JuniorXpEvent::PlanBad => -0.05,
            JuniorXpEvent::Timeout => -0.03,
        }
    }

    pub fn is_positive(&self) -> bool {
        matches!(self, JuniorXpEvent::PlanGood)
    }

    pub fn label(&self) -> &'static str {
        match self {
            JuniorXpEvent::PlanGood => "plan_good",
            JuniorXpEvent::PlanPatchedBySenior => "plan_patched_by_senior",
            JuniorXpEvent::PlanBad => "plan_bad",
            JuniorXpEvent::Timeout => "timeout",
        }
    }
}

/// Senior XP event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeniorXpEvent {
    /// Approved a correct draft
    ApproveOk,
    /// Fixed a flawed draft correctly
    FixAndAcceptOk,
    /// Rubber-stamp blocked (refused weak evidence)
    RubberStampBlocked,
    /// Timeout during review
    Timeout,
}

impl SeniorXpEvent {
    pub fn base_xp(&self) -> u64 {
        match self {
            SeniorXpEvent::ApproveOk => 3,
            SeniorXpEvent::FixAndAcceptOk => 4,
            SeniorXpEvent::RubberStampBlocked => 2, // Good to catch issues
            SeniorXpEvent::Timeout => 0,
        }
    }

    pub fn base_trust(&self) -> f32 {
        match self {
            SeniorXpEvent::ApproveOk => 0.02,
            SeniorXpEvent::FixAndAcceptOk => 0.01,
            SeniorXpEvent::RubberStampBlocked => 0.01, // Careful is good
            SeniorXpEvent::Timeout => -0.03,
        }
    }

    pub fn is_positive(&self) -> bool {
        !matches!(self, SeniorXpEvent::Timeout)
    }

    pub fn label(&self) -> &'static str {
        match self {
            SeniorXpEvent::ApproveOk => "approve_ok",
            SeniorXpEvent::FixAndAcceptOk => "fix_and_accept_ok",
            SeniorXpEvent::RubberStampBlocked => "rubber_stamp_blocked",
            SeniorXpEvent::Timeout => "timeout",
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpg_title_bands() {
        assert_eq!(get_rpg_title(1), "Intern");
        assert_eq!(get_rpg_title(4), "Intern");
        assert_eq!(get_rpg_title(5), "Junior Specialist");
        assert_eq!(get_rpg_title(10), "Specialist");
        assert_eq!(get_rpg_title(20), "Senior Specialist");
        assert_eq!(get_rpg_title(35), "Lead");
        assert_eq!(get_rpg_title(50), "Principal");
        assert_eq!(get_rpg_title(70), "Archon");
        assert_eq!(get_rpg_title(90), "Mythic");
        assert_eq!(get_rpg_title(99), "Mythic");
    }

    #[test]
    fn test_trust_level() {
        assert_eq!(TrustLevel::from_trust(0.2), TrustLevel::Low);
        assert_eq!(TrustLevel::from_trust(0.5), TrustLevel::Normal);
        assert_eq!(TrustLevel::from_trust(0.8), TrustLevel::High);
    }

    #[test]
    fn test_mood_text() {
        assert!(get_mood_text(20.0).contains("learning"));
        assert!(get_mood_text(50.0).contains("improving"));
        assert!(get_mood_text(70.0).contains("performing well"));
        assert!(get_mood_text(90.0).contains("excellent"));
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(0, 10), "[----------]");
        assert_eq!(progress_bar(50, 10), "[=====-----]");
        assert_eq!(progress_bar(100, 10), "[==========]");
    }

    #[test]
    fn test_reliability_scaling() {
        // Green (90%+)
        let scale = ReliabilityScale::from_reliability(0.95);
        assert_eq!(scale.scale_xp(10), 10);
        assert_eq!(scale.quality, "Green");

        // Yellow (70-90%)
        let scale = ReliabilityScale::from_reliability(0.75);
        assert_eq!(scale.scale_xp(10), 8); // 10 * 0.75 = 7.5 -> 8
        assert_eq!(scale.quality, "Yellow");

        // Orange (50-70%)
        let scale = ReliabilityScale::from_reliability(0.55);
        assert_eq!(scale.scale_xp(10), 5);
        assert_eq!(scale.quality, "Orange");

        // Red (0-50%)
        let scale = ReliabilityScale::from_reliability(0.3);
        assert_eq!(scale.scale_xp(10), 3); // 10 * 0.25 = 2.5 -> 3
        assert_eq!(scale.quality, "Red");
    }

    #[test]
    fn test_streak_text() {
        assert_eq!(get_streak_text(1, 0), "1 successful answer");
        assert_eq!(get_streak_text(5, 0), "5 successful answers in a row");
        assert_eq!(get_streak_text(0, 1), "1 recent failure");
        assert_eq!(get_streak_text(0, 3), "3 failures to recover from");
        assert_eq!(get_streak_text(0, 0), "starting fresh");
    }
}
