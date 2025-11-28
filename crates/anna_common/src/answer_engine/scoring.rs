//! Reliability Scoring v0.65.0
//!
//! Scoring formula: overall = 0.4 * evidence + 0.3 * reasoning + 0.3 * coverage
//!
//! v0.65.0 Confidence Thresholds:
//! - >= 90%: GREEN (high confidence) - answer with confidence
//! - 70% - 89%: YELLOW (medium confidence) - answer with caveats
//! - 60% - 69%: RED (low confidence) - warn user, may be unreliable
//! - < 60%: REFUSED - hard gate, do not answer
//!
//! Display: Always show percentages, never raw floats.

use serde::{Deserialize, Serialize};

// ============================================================================
// Thresholds (v0.65.0)
// ============================================================================

/// Hard minimum confidence to provide ANY answer (below this = refuse entirely)
/// v0.65.0: This is a HARD GATE - answers below 60% are refused with a clear message
pub const MIN_CONFIDENCE_TO_ANSWER: f64 = 0.60;

/// Minimum score to accept as "reliable" (yellow zone floor)
pub const MINIMUM_ACCEPTABLE_SCORE: f64 = 0.70;

/// High confidence threshold (green zone floor)
pub const HIGH_CONFIDENCE_THRESHOLD: f64 = 0.90;

/// Maximum number of probe-audit loops before giving up
pub const MAX_LOOPS: usize = 3;

// ============================================================================
// Score Weights
// ============================================================================

/// Weight for evidence score in overall calculation
pub const EVIDENCE_WEIGHT: f64 = 0.4;

/// Weight for reasoning score in overall calculation
pub const REASONING_WEIGHT: f64 = 0.3;

/// Weight for coverage score in overall calculation
pub const COVERAGE_WEIGHT: f64 = 0.3;

// ============================================================================
// Reliability Scores
// ============================================================================

/// Reliability scores from LLM-A self-assessment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityScores {
    /// How well the answer is backed by probe evidence (0.0 - 1.0)
    #[serde(default)]
    pub evidence: f64,
    /// How logically consistent and non-contradictory (0.0 - 1.0)
    #[serde(default)]
    pub reasoning: f64,
    /// How well the answer covers the actual user question (0.0 - 1.0)
    #[serde(default)]
    pub coverage: f64,
}

impl ReliabilityScores {
    /// Create new scores
    pub fn new(evidence: f64, reasoning: f64, coverage: f64) -> Self {
        Self {
            evidence: evidence.clamp(0.0, 1.0),
            reasoning: reasoning.clamp(0.0, 1.0),
            coverage: coverage.clamp(0.0, 1.0),
        }
    }

    /// Compute weighted overall score
    pub fn overall(&self) -> f64 {
        EVIDENCE_WEIGHT * self.evidence
            + REASONING_WEIGHT * self.reasoning
            + COVERAGE_WEIGHT * self.coverage
    }

    /// Check if scores meet minimum threshold for reliable answer (70%+)
    pub fn is_acceptable(&self) -> bool {
        self.overall() >= MINIMUM_ACCEPTABLE_SCORE
    }

    /// v0.65.0: Check if confidence is above hard minimum (60%+)
    /// Answers below this threshold should be REFUSED entirely
    pub fn is_above_min_confidence(&self) -> bool {
        self.overall() >= MIN_CONFIDENCE_TO_ANSWER
    }

    /// Check if scores indicate high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.overall() >= HIGH_CONFIDENCE_THRESHOLD
    }

    /// Get confidence level with v0.65.0 thresholds
    pub fn confidence_level(&self) -> ScoreConfidence {
        let overall = self.overall();
        if overall >= HIGH_CONFIDENCE_THRESHOLD {
            ScoreConfidence::High
        } else if overall >= MINIMUM_ACCEPTABLE_SCORE {
            ScoreConfidence::Medium
        } else if overall >= MIN_CONFIDENCE_TO_ANSWER {
            ScoreConfidence::Low
        } else {
            ScoreConfidence::Refused
        }
    }

    /// v0.65.0: Format confidence as percentage string (e.g., "85%")
    pub fn as_percentage(&self) -> String {
        format!("{:.0}%", self.overall() * 100.0)
    }
}

/// Score confidence level (v0.65.0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScoreConfidence {
    /// >= 90% - GREEN (high confidence)
    High,
    /// 70% - 89% - YELLOW (medium confidence)
    Medium,
    /// 60% - 69% - RED (low confidence, warn user)
    Low,
    /// < 60% - REFUSED (hard gate, do not answer)
    Refused,
}

impl ScoreConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScoreConfidence::High => "HIGH",
            ScoreConfidence::Medium => "MEDIUM",
            ScoreConfidence::Low => "LOW",
            ScoreConfidence::Refused => "REFUSED",
        }
    }

    /// v0.65.0: Get display label with percentage ranges
    pub fn display_label(&self) -> &'static str {
        match self {
            ScoreConfidence::High => "GREEN",
            ScoreConfidence::Medium => "YELLOW",
            ScoreConfidence::Low => "RED",
            ScoreConfidence::Refused => "REFUSED",
        }
    }

    /// ANSI color code for terminal
    pub fn ansi_color(&self) -> &'static str {
        match self {
            ScoreConfidence::High => "\x1b[32m",   // Green
            ScoreConfidence::Medium => "\x1b[33m", // Yellow
            ScoreConfidence::Low => "\x1b[31m",    // Red
            ScoreConfidence::Refused => "\x1b[91m", // Bright Red
        }
    }

    /// v0.65.0: Check if this confidence level is acceptable (at least 60%)
    pub fn is_acceptable(&self) -> bool {
        !matches!(self, ScoreConfidence::Refused)
    }

    /// v0.65.0: Check if confidence requires a warning (red zone)
    pub fn requires_warning(&self) -> bool {
        matches!(self, ScoreConfidence::Low)
    }
}

// ============================================================================
// Loop State
// ============================================================================

/// State tracking for the probe-audit loop
#[derive(Debug, Clone)]
pub struct LoopState {
    /// Current loop iteration (0-indexed)
    pub iteration: usize,
    /// Scores from each iteration
    pub score_history: Vec<f64>,
    /// Did we reach acceptable score?
    pub reached_acceptable: bool,
    /// Final outcome
    pub outcome: LoopOutcome,
}

impl Default for LoopState {
    fn default() -> Self {
        Self {
            iteration: 0,
            score_history: vec![],
            reached_acceptable: false,
            outcome: LoopOutcome::Pending,
        }
    }
}

impl LoopState {
    /// Check if we can do another iteration
    pub fn can_continue(&self) -> bool {
        self.iteration < MAX_LOOPS && self.outcome == LoopOutcome::Pending
    }

    /// Record a score for this iteration
    pub fn record_score(&mut self, score: f64) {
        self.score_history.push(score);
        if score >= MINIMUM_ACCEPTABLE_SCORE {
            self.reached_acceptable = true;
        }
    }

    /// Mark as approved
    pub fn mark_approved(&mut self) {
        self.outcome = LoopOutcome::Approved;
    }

    /// Mark as refused
    pub fn mark_refused(&mut self) {
        self.outcome = LoopOutcome::Refused;
    }

    /// Mark as exhausted (max loops reached)
    pub fn mark_exhausted(&mut self) {
        self.outcome = LoopOutcome::Exhausted;
    }

    /// Advance to next iteration
    pub fn next_iteration(&mut self) {
        self.iteration += 1;
    }
}

/// Loop outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoopOutcome {
    /// Still processing
    Pending,
    /// Answer approved
    Approved,
    /// Answer refused by auditor
    Refused,
    /// Max loops exhausted without approval
    Exhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thresholds() {
        assert!((MIN_CONFIDENCE_TO_ANSWER - 0.60).abs() < 0.001);
        assert!((MINIMUM_ACCEPTABLE_SCORE - 0.70).abs() < 0.001);
        assert!((HIGH_CONFIDENCE_THRESHOLD - 0.90).abs() < 0.001);
        assert_eq!(MAX_LOOPS, 3);
    }

    #[test]
    fn test_weights_sum_to_one() {
        let sum = EVIDENCE_WEIGHT + REASONING_WEIGHT + COVERAGE_WEIGHT;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_reliability_scores_overall() {
        let scores = ReliabilityScores::new(0.95, 0.90, 0.85);
        // 0.4 * 0.95 + 0.3 * 0.90 + 0.3 * 0.85 = 0.38 + 0.27 + 0.255 = 0.905
        assert!((scores.overall() - 0.905).abs() < 0.001);
    }

    #[test]
    fn test_reliability_scores_clamp() {
        let scores = ReliabilityScores::new(1.5, -0.5, 0.5);
        assert_eq!(scores.evidence, 1.0);
        assert_eq!(scores.reasoning, 0.0);
        assert_eq!(scores.coverage, 0.5);
    }

    #[test]
    fn test_is_acceptable() {
        let high = ReliabilityScores::new(0.95, 0.95, 0.95);
        assert!(high.is_acceptable());

        let medium = ReliabilityScores::new(0.80, 0.80, 0.80);
        assert!(medium.is_acceptable());

        let low = ReliabilityScores::new(0.50, 0.50, 0.50);
        assert!(!low.is_acceptable());
    }

    #[test]
    fn test_confidence_level() {
        // >= 90% = High (GREEN)
        let high = ReliabilityScores::new(0.95, 0.95, 0.95);
        assert_eq!(high.confidence_level(), ScoreConfidence::High);

        // 70% - 89% = Medium (YELLOW)
        let medium = ReliabilityScores::new(0.80, 0.80, 0.80);
        assert_eq!(medium.confidence_level(), ScoreConfidence::Medium);

        // 60% - 69% = Low (RED)
        let low = ReliabilityScores::new(0.65, 0.65, 0.65);
        assert_eq!(low.confidence_level(), ScoreConfidence::Low);

        // < 60% = Refused (HARD GATE)
        let refused = ReliabilityScores::new(0.50, 0.50, 0.50);
        assert_eq!(refused.confidence_level(), ScoreConfidence::Refused);
    }

    #[test]
    fn test_is_above_min_confidence() {
        // 65% should be above min (60%)
        let above = ReliabilityScores::new(0.65, 0.65, 0.65);
        assert!(above.is_above_min_confidence());

        // 55% should be below min (60%)
        let below = ReliabilityScores::new(0.55, 0.55, 0.55);
        assert!(!below.is_above_min_confidence());
    }

    #[test]
    fn test_as_percentage() {
        let scores = ReliabilityScores::new(0.85, 0.85, 0.85);
        assert_eq!(scores.as_percentage(), "85%");

        let low = ReliabilityScores::new(0.654, 0.654, 0.654);
        assert_eq!(low.as_percentage(), "65%");
    }

    #[test]
    fn test_loop_state() {
        let mut state = LoopState::default();
        assert!(state.can_continue());
        assert_eq!(state.iteration, 0);

        state.record_score(0.65);
        assert!(!state.reached_acceptable);

        state.next_iteration();
        state.record_score(0.85);
        assert!(state.reached_acceptable);

        assert_eq!(state.score_history.len(), 2);
    }

    #[test]
    fn test_loop_state_max_iterations() {
        let mut state = LoopState::default();
        for _ in 0..MAX_LOOPS {
            state.next_iteration();
        }
        assert_eq!(state.iteration, MAX_LOOPS);
        // Should not be able to continue after MAX_LOOPS
        state.mark_exhausted();
        assert!(!state.can_continue());
    }

    #[test]
    fn test_score_confidence_colors() {
        assert_eq!(ScoreConfidence::High.ansi_color(), "\x1b[32m");
        assert_eq!(ScoreConfidence::Medium.ansi_color(), "\x1b[33m");
        assert_eq!(ScoreConfidence::Low.ansi_color(), "\x1b[31m");
        assert_eq!(ScoreConfidence::Refused.ansi_color(), "\x1b[91m");
    }

    #[test]
    fn test_score_confidence_acceptable() {
        assert!(ScoreConfidence::High.is_acceptable());
        assert!(ScoreConfidence::Medium.is_acceptable());
        assert!(ScoreConfidence::Low.is_acceptable());
        assert!(!ScoreConfidence::Refused.is_acceptable());
    }

    #[test]
    fn test_score_confidence_requires_warning() {
        assert!(!ScoreConfidence::High.requires_warning());
        assert!(!ScoreConfidence::Medium.requires_warning());
        assert!(ScoreConfidence::Low.requires_warning());
        assert!(!ScoreConfidence::Refused.requires_warning()); // Refused isn't a warning, it's a hard gate
    }
}
