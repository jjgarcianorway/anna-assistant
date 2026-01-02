//! Intent Quality Stats (Part F) - v0.0.437.
//!
//! Track intent classification quality:
//! - total_questions
//! - clarified (user needed to clarify)
//! - misclassified (Anna got the intent wrong)
//! - corrected_by_user (user said "that's not what I asked")
//!
//! Do not hide this. This is how Anna improves.

mod detector;
mod tracker;
mod types;

pub use detector::MisclassificationDetector;
pub use tracker::ConversationIntentTracker;
pub use types::{IntentOutcome, IntentQualityStats, MisclassificationSignal, TrackedIntent};

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
        use crate::question_contract::intent::Subject;

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
