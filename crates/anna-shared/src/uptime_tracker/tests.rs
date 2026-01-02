//! Uptime tracking tests.

#[cfg(test)]
mod tests {
    use crate::uptime_tracker::{
        record::UptimeRecord,
        tracker::UptimeTracker,
        formatting::*,
    };

    #[test]
    fn test_uptime_record_start() {
        let record = UptimeRecord::start(1000);
        assert!(record.is_active());
        assert_eq!(record.start_time, 1000);
    }

    #[test]
    fn test_uptime_record_end() {
        let mut record = UptimeRecord::start(1000);
        record.end(2000, true);

        assert!(!record.is_active());
        assert_eq!(record.duration_secs, 1000);
        assert!(record.clean_shutdown);
    }

    #[test]
    fn test_tracker_new() {
        let tracker = UptimeTracker::new(1000);
        assert_eq!(tracker.installed_at, 1000);
        assert!(!tracker.is_running());
    }

    #[test]
    fn test_start_session() {
        let mut tracker = UptimeTracker::new(1000);
        tracker.start_session(2000);

        assert!(tracker.is_running());
        assert_eq!(tracker.session_count, 1);
    }

    #[test]
    fn test_end_session() {
        let mut tracker = UptimeTracker::new(1000);
        tracker.start_session(2000);
        tracker.end_session(3000, true);

        assert!(!tracker.is_running());
        assert_eq!(tracker.total_uptime_secs, 1000);
        assert_eq!(tracker.clean_shutdowns, 1);
    }

    #[test]
    fn test_crash_tracking() {
        let mut tracker = UptimeTracker::new(1000);
        tracker.start_session(2000);
        tracker.end_session(3000, false);

        assert_eq!(tracker.crashes, 1);
        assert_eq!(tracker.clean_shutdown_rate(), 0.0);
    }

    #[test]
    fn test_days_since_install() {
        let tracker = UptimeTracker::new(0);
        // 2 days later
        assert_eq!(tracker.days_since_install(86400 * 2), 2);
    }

    #[test]
    fn test_uptime_percentage() {
        let mut tracker = UptimeTracker::new(0);
        tracker.start_session(0);
        tracker.end_session(50, true);

        // 50 secs uptime / 100 secs total = 50%
        assert_eq!(tracker.uptime_percentage(100), 50.0);
    }

    #[test]
    fn test_avg_session_duration() {
        let mut tracker = UptimeTracker::new(0);

        tracker.start_session(0);
        tracker.end_session(100, true);

        tracker.start_session(100);
        tracker.end_session(300, true);

        // (100 + 200) / 2 = 150
        assert_eq!(tracker.avg_session_duration(), 150.0);
    }

    #[test]
    fn test_longest_shortest() {
        let mut tracker = UptimeTracker::new(0);

        tracker.start_session(0);
        tracker.end_session(100, true);

        tracker.start_session(100);
        tracker.end_session(400, true);

        assert_eq!(tracker.longest_session_secs, 300);
        assert_eq!(tracker.shortest_session_secs, 100);
    }

    #[test]
    fn test_recent_sessions_limit() {
        let mut tracker = UptimeTracker::new(0);

        for i in 0..15 {
            tracker.start_session(i * 100);
            tracker.end_session(i * 100 + 50, true);
        }

        assert_eq!(tracker.recent_sessions.len(), 10);
    }

    #[test]
    fn test_format_duration_secs() {
        assert_eq!(format_duration_secs(30), "30s");
        assert_eq!(format_duration_secs(90), "1m 30s");
        assert_eq!(format_duration_secs(3700), "1h 1m");
        assert_eq!(format_duration_secs(90000), "1d 1h");
    }

    #[test]
    fn test_summary() {
        let mut tracker = UptimeTracker::new(0);
        tracker.start_session(0);
        tracker.end_session(3600, true);

        let summary = tracker.summary(7200);
        assert_eq!(summary.sessions, 1);
        assert!(summary.total_hours > 0.0);
    }

    #[test]
    fn test_format_compact() {
        let mut tracker = UptimeTracker::new(0);
        tracker.start_session(0);

        let output = format_uptime_compact(&tracker, 100);
        assert!(output.contains("up"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = UptimeTracker::new(0);
        tracker.start_session(0);
        tracker.end_session(100, true);
        tracker.start_session(100);
        tracker.end_session(200, true);

        let fact = uptime_fun_fact(&tracker, 86400 * 30);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_uptime_query() {
        assert!(is_uptime_query("show uptime"));
        assert!(is_uptime_query("how long running"));
        assert!(is_uptime_query("when installed"));

        assert!(!is_uptime_query("install vim"));
    }
}
