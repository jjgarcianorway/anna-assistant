//! Repeated Questions Detection (v0.0.485).
//!
//! Tracks and detects repeated or similar questions from users.
//! Helps identify patterns and opportunities for recipe learning.

mod category;
mod formatting;
mod normalization;
mod types;

// Re-export public API
pub use category::detect_category;
pub use formatting::{
    format_repeated_compact, format_repeated_questions, is_repeated_questions_query,
};
pub use normalization::{calculate_similarity, normalize_question};
pub use types::{
    QuestionHistory, RecordedQuestion, RepeatedQuestionsSummary, MIN_REPEAT_COUNT,
    SIMILARITY_THRESHOLD,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_question() {
        let mut history = QuestionHistory::new();

        history.record("How do I install vim?", 1000);
        history.record("How do I install vim?", 2000);
        history.record("how can i install vim", 3000);

        // All three should be grouped (same normalized or similar)
        assert!(history.questions.len() <= 2);

        let repeated = history.get_repeated();
        assert!(!repeated.is_empty());
        // At least 2 occurrences of similar questions
        assert!(repeated.iter().any(|q| q.count >= 2));
    }

    #[test]
    fn test_similar_questions_grouped() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("please install vim", 2000);
        history.record("can you install vim", 3000);

        // Should be grouped as similar
        assert!(history.questions.len() <= 2);
    }

    #[test]
    fn test_top_repeated() {
        let mut history = QuestionHistory::new();

        // Question asked 5 times
        for i in 0..5 {
            history.record("install vim", 1000 + i * 100);
        }

        // Question asked 3 times
        for i in 0..3 {
            history.record("restart nginx", 2000 + i * 100);
        }

        // Question asked once
        history.record("disk usage", 3000);

        let top = history.top_repeated(10);
        assert!(top.len() >= 2);
        assert!(top[0].count >= top[1].count);
    }

    #[test]
    fn test_by_category() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("install htop", 1100);
        history.record("restart docker", 2000);

        let packages = history.by_category("package");
        assert!(packages.len() >= 1);
    }

    #[test]
    fn test_mark_resolved() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("install vim", 2000);

        history.mark_resolved("install vim");

        let unresolved = history.unresolved_repeated();
        assert!(unresolved.is_empty());
    }

    #[test]
    fn test_summary() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("install vim", 2000);
        history.record("restart nginx", 3000);

        let summary = history.summary();
        assert_eq!(summary.total_unique, 2);
        assert_eq!(summary.repeated_count, 1);
    }

    #[test]
    fn test_recorded_question_days() {
        let q = RecordedQuestion::new("test", 0);
        assert_eq!(q.days_since_first(86400), 1);
        assert_eq!(q.days_since_first(86400 * 7), 7);
    }
}
