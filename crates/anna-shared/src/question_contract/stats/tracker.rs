//! Conversation-level intent tracking.

use super::detector::MisclassificationDetector;
use super::types::{IntentOutcome, IntentQualityStats, TrackedIntent};

/// Track intent quality for a single conversation.
#[derive(Debug, Clone, Default)]
pub struct ConversationIntentTracker {
    /// Intents in this conversation.
    pub intents: Vec<TrackedIntent>,
}

impl ConversationIntentTracker {
    /// Create new tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an intent.
    pub fn record_intent(&mut self, intent_id: &str, category: &str, subject: &str) {
        self.intents.push(TrackedIntent {
            intent_id: intent_id.to_string(),
            category: category.to_string(),
            subject: subject.to_string(),
            outcome: None,
        });
    }

    /// Update outcome for latest intent.
    pub fn update_outcome(&mut self, outcome: IntentOutcome) {
        if let Some(last) = self.intents.last_mut() {
            last.outcome = Some(outcome);
        }
    }

    /// Check user response for misclassification signals.
    pub fn check_response(&mut self, user_response: &str) {
        let signal = MisclassificationDetector::detect(user_response);
        if let Some(outcome) = signal.to_outcome() {
            self.update_outcome(outcome);
        }
    }

    /// Export stats for this conversation.
    pub fn export_stats(&self) -> IntentQualityStats {
        let mut stats = IntentQualityStats::new();

        for intent in &self.intents {
            if let Some(outcome) = intent.outcome {
                stats.record(outcome);
            } else {
                // Assume correct if no outcome recorded
                stats.record(IntentOutcome::Correct);
            }
        }

        stats
    }
}
