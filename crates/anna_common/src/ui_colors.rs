//! UI Colors Module v1.0.0
//!
//! CANONICAL source for all UI colors, thresholds, and styling constants.
//!
//! ## Architecture Note (v1.0.0)
//!
//! This module centralizes all color definitions to eliminate duplication:
//! - Actor colors (Anna, Junior, Senior, System)
//! - Reliability colors (Green, Yellow, Red, Refused)
//! - Reliability thresholds (canonical values from docs/architecture.md)
//! - Debug event colors
//!
//! Other modules should import from here rather than defining their own colors.

use owo_colors::OwoColorize;

// ============================================================================
// Reliability Thresholds - CANONICAL (from docs/architecture.md Section 4)
// ============================================================================

/// Green reliability threshold (>= 90% confidence)
pub const THRESHOLD_GREEN: f64 = 0.90;

/// Yellow reliability threshold (>= 70% confidence)
pub const THRESHOLD_YELLOW: f64 = 0.70;

/// Red reliability threshold (>= 50% confidence) - warn user
pub const THRESHOLD_RED: f64 = 0.50;

/// Below 50% = REFUSED (hard gate, do not answer)
/// This is not a threshold but a policy: anything below THRESHOLD_RED is refused

// ============================================================================
// Actor Colors (RGB tuples for true color terminals)
// ============================================================================

/// Anna (orchestrator) - Medium purple
pub const COLOR_ANNA: (u8, u8, u8) = (147, 112, 219);

/// Junior (planner) - Cornflower blue
pub const COLOR_JUNIOR: (u8, u8, u8) = (100, 149, 237);

/// Senior (reviewer) - Orange
pub const COLOR_SENIOR: (u8, u8, u8) = (255, 165, 0);

/// System (infrastructure) - Gray
pub const COLOR_SYSTEM: (u8, u8, u8) = (128, 128, 128);

// ============================================================================
// Reliability Colors (RGB tuples)
// ============================================================================

/// Green - High confidence (>= 90%)
pub const COLOR_GREEN: (u8, u8, u8) = (50, 205, 50);

/// Yellow - Medium confidence (70-89%)
pub const COLOR_YELLOW: (u8, u8, u8) = (255, 215, 0);

/// Red - Low confidence (50-69%)
pub const COLOR_RED: (u8, u8, u8) = (255, 69, 0);

/// Refused - Hard gate (< 50%)
pub const COLOR_REFUSED: (u8, u8, u8) = (220, 20, 60);

// ============================================================================
// Status Colors
// ============================================================================

/// Success/OK - Bright green
pub const COLOR_OK: (u8, u8, u8) = (0, 255, 0);

/// Error - Bright red
pub const COLOR_ERROR: (u8, u8, u8) = (255, 0, 0);

/// Warning - Amber
pub const COLOR_WARNING: (u8, u8, u8) = (255, 191, 0);

/// Muted/dimmed - Dark gray
pub const COLOR_MUTED: (u8, u8, u8) = (100, 100, 100);

// ============================================================================
// Reliability Classification
// ============================================================================

/// Reliability level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReliabilityLevel {
    /// >= 90% - High confidence, full trust
    Green,
    /// 70-89% - Medium confidence, proceed with caution
    Yellow,
    /// 50-69% - Low confidence, warn user
    Red,
    /// < 50% - Hard gate, refuse to answer
    Refused,
}

impl ReliabilityLevel {
    /// Classify a reliability score into a level
    pub fn from_score(score: f64) -> Self {
        if score >= THRESHOLD_GREEN {
            ReliabilityLevel::Green
        } else if score >= THRESHOLD_YELLOW {
            ReliabilityLevel::Yellow
        } else if score >= THRESHOLD_RED {
            ReliabilityLevel::Red
        } else {
            ReliabilityLevel::Refused
        }
    }

    /// Same as from_score but accepts f32 for compatibility
    pub fn from_score_f32(score: f32) -> Self {
        Self::from_score(score as f64)
    }

    /// Get the RGB color tuple for this level
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            ReliabilityLevel::Green => COLOR_GREEN,
            ReliabilityLevel::Yellow => COLOR_YELLOW,
            ReliabilityLevel::Red => COLOR_RED,
            ReliabilityLevel::Refused => COLOR_REFUSED,
        }
    }

    /// Get the display label for this level
    pub fn label(&self) -> &'static str {
        match self {
            ReliabilityLevel::Green => "GREEN",
            ReliabilityLevel::Yellow => "YELLOW",
            ReliabilityLevel::Red => "RED",
            ReliabilityLevel::Refused => "REFUSED",
        }
    }

    /// Get lowercase label (for backward compatibility)
    pub fn label_lower(&self) -> &'static str {
        match self {
            ReliabilityLevel::Green => "green",
            ReliabilityLevel::Yellow => "yellow",
            ReliabilityLevel::Red => "red",
            ReliabilityLevel::Refused => "refused",
        }
    }

    /// Is this level acceptable for answering?
    pub fn is_acceptable(&self) -> bool {
        !matches!(self, ReliabilityLevel::Refused)
    }

    /// Is this level high confidence?
    pub fn is_high_confidence(&self) -> bool {
        matches!(self, ReliabilityLevel::Green)
    }
}

// ============================================================================
// Formatting Helpers
// ============================================================================

/// Format a score with color using owo_colors
pub fn format_score_colored(score: f64) -> String {
    let level = ReliabilityLevel::from_score(score);
    let (r, g, b) = level.color();
    let score_str = format!("{:.0}%", score * 100.0);
    score_str.truecolor(r, g, b).bold().to_string()
}

/// Format a score with color and label
pub fn format_score_with_label(score: f64) -> String {
    let level = ReliabilityLevel::from_score(score);
    let (r, g, b) = level.color();
    let score_str = format!("{:.0}%", score * 100.0);
    format!(
        "{} ({})",
        score_str.truecolor(r, g, b).bold(),
        level.label().truecolor(r, g, b)
    )
}

/// Get color and label tuple for a score (for compatibility)
pub fn reliability_display(score: f64) -> ((u8, u8, u8), &'static str) {
    let level = ReliabilityLevel::from_score(score);
    (level.color(), level.label())
}

/// Same as reliability_display but accepts f32
pub fn reliability_display_f32(score: f32) -> ((u8, u8, u8), &'static str) {
    reliability_display(score as f64)
}

// ============================================================================
// Actor Helpers
// ============================================================================

/// Actor type for color selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Anna,
    Junior,
    Senior,
    System,
}

impl Actor {
    /// Get the RGB color for this actor
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Actor::Anna => COLOR_ANNA,
            Actor::Junior => COLOR_JUNIOR,
            Actor::Senior => COLOR_SENIOR,
            Actor::System => COLOR_SYSTEM,
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
    fn test_threshold_ordering() {
        assert!(THRESHOLD_GREEN > THRESHOLD_YELLOW);
        assert!(THRESHOLD_YELLOW > THRESHOLD_RED);
        assert!(THRESHOLD_RED > 0.0);
        assert!(THRESHOLD_GREEN <= 1.0);
    }

    #[test]
    fn test_reliability_classification() {
        assert_eq!(ReliabilityLevel::from_score(0.95), ReliabilityLevel::Green);
        assert_eq!(ReliabilityLevel::from_score(0.90), ReliabilityLevel::Green);
        assert_eq!(ReliabilityLevel::from_score(0.89), ReliabilityLevel::Yellow);
        assert_eq!(ReliabilityLevel::from_score(0.70), ReliabilityLevel::Yellow);
        assert_eq!(ReliabilityLevel::from_score(0.69), ReliabilityLevel::Red);
        assert_eq!(ReliabilityLevel::from_score(0.50), ReliabilityLevel::Red);
        assert_eq!(ReliabilityLevel::from_score(0.49), ReliabilityLevel::Refused);
        assert_eq!(ReliabilityLevel::from_score(0.0), ReliabilityLevel::Refused);
    }

    #[test]
    fn test_reliability_colors() {
        assert_eq!(ReliabilityLevel::Green.color(), COLOR_GREEN);
        assert_eq!(ReliabilityLevel::Yellow.color(), COLOR_YELLOW);
        assert_eq!(ReliabilityLevel::Red.color(), COLOR_RED);
        assert_eq!(ReliabilityLevel::Refused.color(), COLOR_REFUSED);
    }

    #[test]
    fn test_actor_colors() {
        assert_eq!(Actor::Anna.color(), COLOR_ANNA);
        assert_eq!(Actor::Junior.color(), COLOR_JUNIOR);
        assert_eq!(Actor::Senior.color(), COLOR_SENIOR);
        assert_eq!(Actor::System.color(), COLOR_SYSTEM);
    }

    #[test]
    fn test_is_acceptable() {
        assert!(ReliabilityLevel::Green.is_acceptable());
        assert!(ReliabilityLevel::Yellow.is_acceptable());
        assert!(ReliabilityLevel::Red.is_acceptable());
        assert!(!ReliabilityLevel::Refused.is_acceptable());
    }

    #[test]
    fn test_reliability_display() {
        let (color, label) = reliability_display(0.95);
        assert_eq!(color, COLOR_GREEN);
        assert_eq!(label, "GREEN");

        let (color, label) = reliability_display(0.75);
        assert_eq!(color, COLOR_YELLOW);
        assert_eq!(label, "YELLOW");

        let (color, label) = reliability_display(0.55);
        assert_eq!(color, COLOR_RED);
        assert_eq!(label, "RED");

        let (color, label) = reliability_display(0.30);
        assert_eq!(color, COLOR_REFUSED);
        assert_eq!(label, "REFUSED");
    }
}
