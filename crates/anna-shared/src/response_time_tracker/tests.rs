// v0.0.538: Response Time Tracker - Tests (Phase 114)
// Unit tests for response time tracking functionality

#[cfg(test)]
mod tests {
    use super::super::formatting::{is_response_time_query, response_time_fun_fact};
    use super::super::record::ResponseTimeRecord;
    use super::super::tracker::ResponseTimeTracker;
    use super::super::types::{ComplexityLevel, ResponseType};

    #[test]
    fn test_response_type_default() {
        let rt = ResponseType::default();
        assert_eq!(rt, ResponseType::Direct);
    }

    #[test]
    fn test_complexity_default() {
        let c = ComplexityLevel::default();
        assert_eq!(c, ComplexityLevel::Simple);
    }

    #[test]
    fn test_tracker_creation() {
        let tracker = ResponseTimeTracker::new();
        assert_eq!(tracker.total(), 0);
    }

    #[test]
    fn test_record_response() {
        let mut tracker = ResponseTimeTracker::new();
        let id = tracker.record(150, 25);
        assert!(tracker.get(&id).is_some());
        assert_eq!(tracker.total(), 1);
    }

    #[test]
    fn test_shortest_longest() {
        let mut tracker = ResponseTimeTracker::new();
        tracker.record(100, 10);
        tracker.record(500, 100);
        tracker.record(200, 50);

        let shortest = tracker.shortest_reply().unwrap();
        assert_eq!(shortest.word_count, 10);

        let longest = tracker.longest_reply().unwrap();
        assert_eq!(longest.word_count, 100);
    }

    #[test]
    fn test_average_time() {
        let mut tracker = ResponseTimeTracker::new();
        tracker.record(100, 10);
        tracker.record(200, 20);
        tracker.record(300, 30);

        let avg = tracker.average_time_ms().unwrap();
        assert_eq!(avg, 200);
    }

    #[test]
    fn test_type_stats() {
        let mut tracker = ResponseTimeTracker::new();
        tracker.record_full(100, 10, ResponseType::Direct, ComplexityLevel::Simple);
        tracker.record_full(200, 20, ResponseType::Direct, ComplexityLevel::Simple);
        tracker.record_full(300, 30, ResponseType::Specialist, ComplexityLevel::Complex);

        let stats = tracker.type_stats();
        assert_eq!(*stats.get(&ResponseType::Direct).unwrap_or(&0), 2);
        assert_eq!(*stats.get(&ResponseType::Specialist).unwrap_or(&0), 1);
    }

    #[test]
    fn test_percentile() {
        let mut tracker = ResponseTimeTracker::new();
        for i in 1..=100 {
            tracker.record(i * 10, 10);
        }

        let p95 = tracker.percentile_time(95).unwrap();
        assert!(p95 >= 900 && p95 <= 1000);
    }

    #[test]
    fn test_is_response_time_query() {
        assert!(is_response_time_query("What's my average response time?"));
        assert!(is_response_time_query("Show me the longest reply"));
        assert!(!is_response_time_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = response_time_fun_fact();
        assert!(fact.contains("response") || fact.contains("reply"));
    }

    #[test]
    fn test_words_per_second() {
        let record = ResponseTimeRecord::new("test", 1000, 50);
        assert!((record.words_per_second() - 50.0).abs() < 0.01);
    }
}
