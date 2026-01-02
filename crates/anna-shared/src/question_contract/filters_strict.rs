//! Strict filter that rejects answers with any leakage.

use super::filters::AnswerFilter;
use super::filters_types::DetectedLeakage;
use super::intent::QuestionIntent;

/// Strict filter that rejects answers with any leakage.
pub struct StrictFilter;

impl StrictFilter {
    /// Check if answer passes strict filtering.
    pub fn passes(intent: &QuestionIntent, text: &str) -> StrictFilterResult {
        if intent.category.allows_tutorials() {
            return StrictFilterResult::Pass;
        }

        let result = AnswerFilter::filter(intent, text);

        if result.has_leakage() {
            StrictFilterResult::Reject {
                leakages: result.leakages,
            }
        } else {
            StrictFilterResult::Pass
        }
    }
}

/// Result of strict filtering.
#[derive(Debug, Clone)]
pub enum StrictFilterResult {
    /// Answer passes.
    Pass,
    /// Answer rejected due to leakage.
    Reject { leakages: Vec<DetectedLeakage> },
}

impl StrictFilterResult {
    /// Check if passed.
    pub fn passed(&self) -> bool {
        matches!(self, Self::Pass)
    }
}
