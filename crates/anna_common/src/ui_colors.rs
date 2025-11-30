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

// Note: Below 50% = REFUSED (hard gate, do not answer)
// This is not a threshold but a policy: anything below THRESHOLD_RED is refused

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
// Percentage Formatting - CANONICAL (v3.6.0)
// ============================================================================
//
// GLOBAL RULE: Any value conceptually between 0 and 1 shown to the user
// must be converted to a percentage between 0 and 100 with a percent sign.
//
// Examples:
//   Reliability 0.83 -> "83%"
//   Trust 0.50 -> "50%"
//   Success rate 0.921 -> "92%"
//
// All display code MUST use these helpers instead of inline formatting.

/// Format a 0-1 float value as a percentage string.
/// Uses standard rounding (0.5 rounds up).
///
/// # Arguments
/// * `value` - A float between 0.0 and 1.0
///
/// # Returns
/// A string like "83%" with no decimal places (default)
///
/// # Examples
/// ```
/// use anna_common::ui_colors::format_percentage;
/// assert_eq!(format_percentage(0.0), "0%");
/// assert_eq!(format_percentage(0.5), "50%");
/// assert_eq!(format_percentage(0.923), "92%");
/// assert_eq!(format_percentage(1.0), "100%");
/// ```
pub fn format_percentage(value: f64) -> String {
    format!("{:.0}%", value * 100.0)
}

/// Format a 0-1 float as percentage with specified decimal places.
///
/// # Arguments
/// * `value` - A float between 0.0 and 1.0
/// * `decimals` - Number of decimal places (0, 1, or 2)
///
/// # Examples
/// ```
/// use anna_common::ui_colors::format_percentage_decimals;
/// assert_eq!(format_percentage_decimals(0.923, 0), "92%");
/// assert_eq!(format_percentage_decimals(0.923, 1), "92.3%");
/// assert_eq!(format_percentage_decimals(0.9234, 2), "92.34%");
/// ```
pub fn format_percentage_decimals(value: f64, decimals: usize) -> String {
    match decimals {
        0 => format!("{:.0}%", value * 100.0),
        1 => format!("{:.1}%", value * 100.0),
        2 => format!("{:.2}%", value * 100.0),
        _ => format!("{:.0}%", value * 100.0), // Default to 0 decimals
    }
}

/// Format a 0-1 f32 value as a percentage string.
/// Convenience wrapper for f32 inputs.
pub fn format_percentage_f32(value: f32) -> String {
    format_percentage(value as f64)
}

/// Format a percentage with color based on reliability thresholds.
/// Uses the canonical reliability colors (Green/Yellow/Red).
pub fn format_percentage_colored(value: f64) -> String {
    let level = ReliabilityLevel::from_score(value);
    let (r, g, b) = level.color();
    let pct = format_percentage(value);
    pct.truecolor(r, g, b).bold().to_string()
}

/// Format a percentage with color, accepting f32.
pub fn format_percentage_colored_f32(value: f32) -> String {
    format_percentage_colored(value as f64)
}

// ============================================================================
// Formatting Helpers (Legacy - now use format_percentage internally)
// ============================================================================

/// Format a score with color using owo_colors
/// Note: Now uses format_percentage internally for consistency.
pub fn format_score_colored(score: f64) -> String {
    let level = ReliabilityLevel::from_score(score);
    let (r, g, b) = level.color();
    let score_str = format_percentage(score);
    score_str.truecolor(r, g, b).bold().to_string()
}

/// Format a score with color and label
/// Note: Now uses format_percentage internally for consistency.
pub fn format_score_with_label(score: f64) -> String {
    let level = ReliabilityLevel::from_score(score);
    let (r, g, b) = level.color();
    let score_str = format_percentage(score);
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

    // ========================================================================
    // Percentage Formatting Tests (v3.6.0)
    // ========================================================================

    #[test]
    fn test_format_percentage_boundary_values() {
        // Exact boundaries
        assert_eq!(format_percentage(0.0), "0%");
        assert_eq!(format_percentage(1.0), "100%");
        assert_eq!(format_percentage(0.5), "50%");
    }

    #[test]
    fn test_format_percentage_typical_values() {
        // Common reliability scores
        assert_eq!(format_percentage(0.83), "83%");
        assert_eq!(format_percentage(0.923), "92%"); // 92.3 rounds to 92
        assert_eq!(format_percentage(0.926), "93%"); // 92.6 rounds to 93
        assert_eq!(format_percentage(0.70), "70%");
        assert_eq!(format_percentage(0.90), "90%");
    }

    #[test]
    fn test_format_percentage_small_values() {
        // Near-zero values - Rust uses "round half to even" (banker's rounding)
        assert_eq!(format_percentage(0.001), "0%"); // 0.1 rounds to 0
        assert_eq!(format_percentage(0.006), "1%"); // 0.6 rounds to 1
        assert_eq!(format_percentage(0.009), "1%"); // 0.9 rounds to 1
        assert_eq!(format_percentage(0.01), "1%");  // exactly 1
    }

    #[test]
    fn test_format_percentage_near_100() {
        // Near-100 values
        assert_eq!(format_percentage(0.99), "99%");
        assert_eq!(format_percentage(0.995), "100%"); // Rounds up
        assert_eq!(format_percentage(0.999), "100%");
    }

    #[test]
    fn test_format_percentage_decimals() {
        // 0 decimal places
        assert_eq!(format_percentage_decimals(0.923, 0), "92%");

        // 1 decimal place
        assert_eq!(format_percentage_decimals(0.923, 1), "92.3%");
        assert_eq!(format_percentage_decimals(0.9236, 1), "92.4%"); // 92.36 rounds to 92.4

        // 2 decimal places
        assert_eq!(format_percentage_decimals(0.9234, 2), "92.34%");
        assert_eq!(format_percentage_decimals(0.92346, 2), "92.35%"); // 92.346 rounds to 92.35

        // Invalid decimals (defaults to 0)
        assert_eq!(format_percentage_decimals(0.923, 5), "92%");
    }

    #[test]
    fn test_format_percentage_f32() {
        // f32 convenience wrapper
        assert_eq!(format_percentage_f32(0.5_f32), "50%");
        assert_eq!(format_percentage_f32(0.83_f32), "83%");
        assert_eq!(format_percentage_f32(1.0_f32), "100%");
    }

    #[test]
    fn test_format_percentage_colored_contains_percent() {
        // Colored output should still contain the percentage
        let colored = format_percentage_colored(0.95);
        assert!(colored.contains("95%"), "Colored output should contain '95%'");

        let colored = format_percentage_colored(0.50);
        assert!(colored.contains("50%"), "Colored output should contain '50%'");
    }

    #[test]
    fn test_no_raw_floats_in_formatted_output() {
        // Ensure no 0.XX format appears in output (common bug)
        let values = [0.0, 0.1, 0.5, 0.83, 0.923, 1.0];
        for v in values {
            let result = format_percentage(v);
            assert!(!result.contains("0."), "Output '{}' contains raw float for value {}", result, v);
            assert!(result.ends_with('%'), "Output '{}' should end with %", result);
        }
    }
}
