//! Tests for response length tracking.

#[cfg(test)]
mod tests {
    use super::super::formatting::*;
    use super::super::tracker::ResponseLengthTracker;
    use super::super::types::RecordedResponse;
    use super::super::utils::*;

    #[test]
    fn test_recorded_response_from_text() {
        let response = RecordedResponse::from_text("Hello world", 1000);
        assert_eq!(response.char_count, 11);
        assert_eq!(response.word_count, 2);
        assert_eq!(response.line_count, 1);
    }

    #[test]
    fn test_excerpt_truncation() {
        let long_text = "This is a very long response that should be truncated to fifty characters plus ellipsis";
        let response = RecordedResponse::from_text(long_text, 1000);
        assert!(response.excerpt.len() <= 50);
        assert!(response.excerpt.ends_with("..."));
    }

    #[test]
    fn test_tracker_record() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Short", 1000);
        tracker.record("This is a longer response with more words", 2000);
        tracker.record("Medium length", 3000);

        assert_eq!(tracker.total_responses, 3);
        assert!(tracker.longest.is_some());
        assert!(tracker.shortest.is_some());
    }

    #[test]
    fn test_longest_shortest() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Short", 1000);
        tracker.record("This is significantly longer than the other one", 2000);

        assert_eq!(tracker.shortest.as_ref().unwrap().char_count, 5);
        assert_eq!(tracker.longest.as_ref().unwrap().char_count, 47);
    }

    #[test]
    fn test_averages() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("One two three", 1000); // 13 chars, 3 words
        tracker.record("Four five six seven", 2000); // 19 chars, 4 words

        assert_eq!(tracker.average_chars(), 16.0);
        assert_eq!(tracker.average_words(), 3.5);
    }

    #[test]
    fn test_empty_response_ignored() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("", 1000);
        tracker.record("Valid", 2000);

        assert_eq!(tracker.total_responses, 1);
    }

    #[test]
    fn test_recent_limit() {
        let mut tracker = ResponseLengthTracker::new();

        for i in 0..15 {
            tracker.record(&format!("Response {}", i), i as u64 * 1000);
        }

        assert_eq!(tracker.recent.len(), 10);
    }

    #[test]
    fn test_summary() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Short", 1000);
        tracker.record("Much longer response here", 2000);

        let summary = tracker.summary();
        assert_eq!(summary.total_responses, 2);
        assert!(summary.longest_chars > summary.shortest_chars);
    }

    #[test]
    fn test_format_compact() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Hello", 1000);
        tracker.record("World", 2000);

        let output = format_response_lengths_compact(&tracker);
        assert!(output.contains("2 responses"));
    }

    #[test]
    fn test_response_length_fun_fact() {
        let mut tracker = ResponseLengthTracker::new();

        for i in 0..10 {
            tracker.record(&format!("Response number {} with some content", i), i as u64 * 1000);
        }

        let fact = response_length_fun_fact(&tracker);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_response_length_query() {
        assert!(is_response_length_query("what was the longest reply"));
        assert!(is_response_length_query("show response length stats"));
        assert!(is_response_length_query("shortest response"));

        assert!(!is_response_length_query("install vim"));
        assert!(!is_response_length_query("status"));
    }

    #[test]
    fn test_with_category() {
        let response = RecordedResponse::from_text("Test", 1000).with_category("system");
        assert_eq!(response.category, Some("system".to_string()));
    }

    #[test]
    fn test_record_with_category() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record_with_category("System response", 1000, "system");

        assert!(tracker.longest.as_ref().unwrap().category.is_some());
    }

    #[test]
    fn test_length_range() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Short", 1000);
        tracker.record("Much much longer", 2000);

        let range = tracker.length_range();
        assert!(range.is_some());
        let (min, max) = range.unwrap();
        assert!(min < max);
    }
}
