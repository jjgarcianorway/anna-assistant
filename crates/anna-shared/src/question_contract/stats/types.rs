//! Core types for intent quality statistics.

use serde::{Deserialize, Serialize};

/// Outcome of intent classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentOutcome {
    /// Intent was correctly understood.
    Correct,
    /// User needed to clarify the question.
    Clarified,
    /// Anna misclassified the intent.
    Misclassified,
    /// User explicitly corrected Anna.
    CorrectedByUser,
}

impl IntentOutcome {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Correct => "correct",
            Self::Clarified => "clarified",
            Self::Misclassified => "misclassified",
            Self::CorrectedByUser => "corrected_by_user",
        }
    }

    /// Whether this counts as a success.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Correct)
    }
}

/// Intent quality statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentQualityStats {
    /// Total questions processed.
    pub total_questions: u64,
    /// Questions where user clarified.
    pub clarified: u64,
    /// Questions where intent was misclassified.
    pub misclassified: u64,
    /// Questions where user explicitly corrected.
    pub corrected_by_user: u64,
    /// Questions answered correctly on first try.
    pub correct_first_try: u64,
}

impl IntentQualityStats {
    /// Create new empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an intent outcome.
    pub fn record(&mut self, outcome: IntentOutcome) {
        self.total_questions += 1;

        match outcome {
            IntentOutcome::Correct => self.correct_first_try += 1,
            IntentOutcome::Clarified => self.clarified += 1,
            IntentOutcome::Misclassified => self.misclassified += 1,
            IntentOutcome::CorrectedByUser => self.corrected_by_user += 1,
        }
    }

    /// Get accuracy rate (0.0 to 1.0).
    pub fn accuracy_rate(&self) -> f64 {
        if self.total_questions == 0 {
            return 1.0; // No data = assume good
        }
        self.correct_first_try as f64 / self.total_questions as f64
    }

    /// Get clarification rate (0.0 to 1.0).
    pub fn clarification_rate(&self) -> f64 {
        if self.total_questions == 0 {
            return 0.0;
        }
        self.clarified as f64 / self.total_questions as f64
    }

    /// Get misclassification rate (0.0 to 1.0).
    pub fn misclassification_rate(&self) -> f64 {
        if self.total_questions == 0 {
            return 0.0;
        }
        self.misclassified as f64 / self.total_questions as f64
    }

    /// Get correction rate (0.0 to 1.0).
    pub fn correction_rate(&self) -> f64 {
        if self.total_questions == 0 {
            return 0.0;
        }
        self.corrected_by_user as f64 / self.total_questions as f64
    }

    /// Merge another stats object.
    pub fn merge(&mut self, other: &IntentQualityStats) {
        self.total_questions += other.total_questions;
        self.clarified += other.clarified;
        self.misclassified += other.misclassified;
        self.corrected_by_user += other.corrected_by_user;
        self.correct_first_try += other.correct_first_try;
    }

    /// Format as display string.
    pub fn display(&self) -> String {
        format!(
            "Intent Quality: {:.1}% accuracy | {} clarified | {} misclassified | {} corrected ({} total)",
            self.accuracy_rate() * 100.0,
            self.clarified,
            self.misclassified,
            self.corrected_by_user,
            self.total_questions
        )
    }
}

/// Signal of potential misclassification.
#[derive(Debug, Clone)]
pub enum MisclassificationSignal {
    /// No signal detected.
    None,
    /// User explicitly said wrong answer.
    Explicit { phrase: String },
    /// User rephrased (might indicate misunderstanding).
    Rephrase { phrase: String },
}

impl MisclassificationSignal {
    /// Check if any signal detected.
    pub fn detected(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if explicit misclassification.
    pub fn is_explicit(&self) -> bool {
        matches!(self, Self::Explicit { .. })
    }

    /// Convert to outcome.
    pub fn to_outcome(&self) -> Option<IntentOutcome> {
        match self {
            Self::None => None,
            Self::Explicit { .. } => Some(IntentOutcome::CorrectedByUser),
            Self::Rephrase { .. } => Some(IntentOutcome::Misclassified),
        }
    }
}

/// A tracked intent in a conversation.
#[derive(Debug, Clone)]
pub struct TrackedIntent {
    /// Intent ID.
    pub intent_id: String,
    /// Category detected.
    pub category: String,
    /// Subject detected.
    pub subject: String,
    /// Outcome (if determined).
    pub outcome: Option<IntentOutcome>,
}
