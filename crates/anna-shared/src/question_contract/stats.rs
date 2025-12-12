//! Intent Quality Stats (Part F) - v0.0.437.
//!
//! Track intent classification quality:
//! - total_questions
//! - clarified (user needed to clarify)
//! - misclassified (Anna got the intent wrong)
//! - corrected_by_user (user said "that's not what I asked")
//!
//! Do not hide this. This is how Anna improves.

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

/// Detector for misclassification signals.
pub struct MisclassificationDetector;

impl MisclassificationDetector {
    /// Phrases that indicate misclassification.
    const MISCLASS_PHRASES: &'static [&'static str] = &[
        "that's not what i asked",
        "not what i meant",
        "wrong question",
        "i didn't ask about",
        "i asked about",
        "i wanted to know",
        "that doesn't answer",
        "you didn't answer",
        "different question",
        "misunderstood",
    ];

    /// Phrases that indicate rephrase (potential misclassification).
    const REPHRASE_PHRASES: &'static [&'static str] = &[
        "let me rephrase",
        "what i mean is",
        "to clarify",
        "more specifically",
        "i meant",
        "what i'm asking is",
    ];

    /// Check if user response indicates misclassification.
    pub fn detect(user_response: &str) -> MisclassificationSignal {
        let lower = user_response.to_lowercase();

        // Check for explicit misclassification phrases
        for phrase in Self::MISCLASS_PHRASES {
            if lower.contains(phrase) {
                return MisclassificationSignal::Explicit {
                    phrase: phrase.to_string(),
                };
            }
        }

        // Check for rephrase phrases
        for phrase in Self::REPHRASE_PHRASES {
            if lower.contains(phrase) {
                return MisclassificationSignal::Rephrase {
                    phrase: phrase.to_string(),
                };
            }
        }

        MisclassificationSignal::None
    }

    /// Check if user is asking about a different subject than answered.
    pub fn subject_mismatch(
        answered_subject: super::intent::Subject,
        user_response: &str,
    ) -> bool {
        use super::intent::Subject;

        let lower = user_response.to_lowercase();

        let subject_keywords: &[(&[&str], Subject)] = &[
            (&["memory", "ram", "swap"], Subject::Memory),
            (&["cpu", "processor"], Subject::Cpu),
            (&["disk", "storage", "partition"], Subject::Disk),
            (&["service", "systemd", "unit"], Subject::Service),
            (&["network", "wifi", "ethernet", "ip"], Subject::Network),
            (&["gpu", "graphics", "nvidia", "driver"], Subject::Gpu),
            (&["boot", "startup"], Subject::Boot),
            (&["audio", "sound", "volume"], Subject::Audio),
        ];

        for (keywords, subject) in subject_keywords {
            if keywords.iter().any(|k| lower.contains(k)) {
                if *subject != answered_subject && answered_subject != Subject::Unknown {
                    return true;
                }
            }
        }

        false
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_outcome() {
        assert!(IntentOutcome::Correct.is_success());
        assert!(!IntentOutcome::Misclassified.is_success());
    }

    #[test]
    fn test_stats_record() {
        let mut stats = IntentQualityStats::new();

        stats.record(IntentOutcome::Correct);
        stats.record(IntentOutcome::Correct);
        stats.record(IntentOutcome::Misclassified);

        assert_eq!(stats.total_questions, 3);
        assert_eq!(stats.correct_first_try, 2);
        assert_eq!(stats.misclassified, 1);
        assert!((stats.accuracy_rate() - 0.667).abs() < 0.01);
    }

    #[test]
    fn test_stats_display() {
        let mut stats = IntentQualityStats::new();
        stats.record(IntentOutcome::Correct);
        stats.record(IntentOutcome::CorrectedByUser);

        let display = stats.display();
        assert!(display.contains("accuracy"));
        assert!(display.contains("2 total"));
    }

    #[test]
    fn test_misclassification_detector() {
        // Explicit misclassification
        let signal = MisclassificationDetector::detect("that's not what i asked for");
        assert!(signal.is_explicit());

        // Rephrase
        let signal = MisclassificationDetector::detect("let me rephrase my question");
        assert!(signal.detected());
        assert!(!signal.is_explicit());

        // No signal
        let signal = MisclassificationDetector::detect("thanks, that's helpful");
        assert!(!signal.detected());
    }

    #[test]
    fn test_subject_mismatch() {
        use super::super::intent::Subject;

        // User asks about memory, Anna answered about CPU
        let mismatch = MisclassificationDetector::subject_mismatch(
            Subject::Cpu,
            "I asked about memory usage, not CPU",
        );
        assert!(mismatch);

        // No mismatch
        let mismatch = MisclassificationDetector::subject_mismatch(
            Subject::Memory,
            "Great, that answers my question about RAM",
        );
        assert!(!mismatch);
    }

    #[test]
    fn test_conversation_tracker() {
        let mut tracker = ConversationIntentTracker::new();

        tracker.record_intent("int_001", "fact", "memory");
        tracker.check_response("That's not what I asked!");

        let stats = tracker.export_stats();
        assert_eq!(stats.corrected_by_user, 1);
    }

    #[test]
    fn test_stats_merge() {
        let mut stats1 = IntentQualityStats::new();
        stats1.record(IntentOutcome::Correct);

        let mut stats2 = IntentQualityStats::new();
        stats2.record(IntentOutcome::Misclassified);

        stats1.merge(&stats2);

        assert_eq!(stats1.total_questions, 2);
        assert_eq!(stats1.correct_first_try, 1);
        assert_eq!(stats1.misclassified, 1);
    }
}
