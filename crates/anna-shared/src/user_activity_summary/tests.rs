//! Tests for user activity summary

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_time_of_day_display() {
        assert_eq!(TimeOfDay::Morning.display(), "Morning");
        assert_eq!(TimeOfDay::Night.display(), "Night");
    }

    #[test]
    fn test_time_of_day_from_hour() {
        assert_eq!(TimeOfDay::from_hour(8), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(20), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(2), TimeOfDay::Night);
    }

    #[test]
    fn test_day_of_week_display() {
        assert_eq!(DayOfWeek::Monday.display(), "Monday");
        assert_eq!(DayOfWeek::Monday.short(), "Mon");
    }

    #[test]
    fn test_day_of_week_from_index() {
        assert_eq!(DayOfWeek::from_index(0), DayOfWeek::Monday);
        assert_eq!(DayOfWeek::from_index(6), DayOfWeek::Sunday);
        assert_eq!(DayOfWeek::from_index(7), DayOfWeek::Monday); // Wraps
    }

    #[test]
    fn test_activity_record_new() {
        let record = ActivityRecord::new("query", 1000);
        assert_eq!(record.activity_type, "query");
        assert_eq!(record.timestamp, 1000);
    }

    #[test]
    fn test_user_activity_summary_record() {
        let mut summary = UserActivitySummary::new();
        let record = ActivityRecord::new("query", 1000);
        summary.record(record, TimeOfDay::Morning, DayOfWeek::Monday);

        assert_eq!(summary.total_interactions, 1);
        assert_eq!(summary.by_time_of_day.get("Morning"), Some(&1));
        assert_eq!(summary.by_day_of_week.get("Monday"), Some(&1));
    }

    #[test]
    fn test_user_activity_summary_most_active() {
        let mut summary = UserActivitySummary::new();

        // Record more morning activity
        for _ in 0..5 {
            let record = ActivityRecord::new("query", 1000);
            summary.record(record, TimeOfDay::Morning, DayOfWeek::Monday);
        }

        for _ in 0..2 {
            let record = ActivityRecord::new("query", 1000);
            summary.record(record, TimeOfDay::Afternoon, DayOfWeek::Tuesday);
        }

        let (time, count) = summary.most_active_time().unwrap();
        assert_eq!(time, "Morning");
        assert_eq!(count, 5);
    }

    #[test]
    fn test_days_active() {
        let mut summary = UserActivitySummary::new();
        summary.first_activity = 1000000;
        summary.last_activity = 1000000 + (86400 * 5); // 5 days later

        assert_eq!(summary.days_active(), 6); // Inclusive
    }

    #[test]
    fn test_avg_interactions_per_day() {
        let mut summary = UserActivitySummary::new();
        summary.total_interactions = 100;
        summary.first_activity = 1000000;
        summary.last_activity = 1000000 + (86400 * 9); // 10 days

        assert!((summary.avg_interactions_per_day() - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_format_activity_summary() {
        let mut summary = UserActivitySummary::new();
        let record = ActivityRecord::new("query", 1000);
        summary.record(record, TimeOfDay::Morning, DayOfWeek::Monday);

        let output = format_activity_summary(&summary);
        assert!(output.contains("User Activity"));
        assert!(output.contains("Morning"));
    }

    #[test]
    fn test_format_activity_summary_compact() {
        let mut summary = UserActivitySummary::new();
        summary.total_interactions = 42;
        summary.first_activity = 1000000;
        summary.last_activity = 1000000 + 86400;

        let output = format_activity_summary_compact(&summary);
        assert!(output.contains("42i"));
    }

    #[test]
    fn test_activity_insight() {
        let summary = UserActivitySummary::new();
        assert!(activity_insight(&summary).is_none());

        let mut summary2 = UserActivitySummary::new();
        summary2.total_interactions = 10;
        summary2.first_activity = 1000;
        summary2.last_activity = 2000;

        let insight = activity_insight(&summary2);
        assert!(insight.is_some());
    }

    #[test]
    fn test_is_activity_query() {
        assert!(is_activity_query("show my activity"));
        assert!(is_activity_query("what are my usage patterns?"));
        assert!(is_activity_query("how often do I use anna?"));
        assert!(!is_activity_query("how do I install vim?"));
    }

    #[test]
    fn test_detect_topic() {
        assert_eq!(detect_topic("install vim"), Some("package".to_string()));
        assert_eq!(detect_topic("restart docker"), Some("docker".to_string()));
        assert_eq!(detect_topic("git push"), Some("git".to_string()));
        assert_eq!(detect_topic("hello world"), None);
    }
}
