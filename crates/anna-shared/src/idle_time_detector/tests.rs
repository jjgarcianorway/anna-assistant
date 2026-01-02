//! Tests for idle time detection

#[cfg(test)]
mod tests {
    use crate::idle_time_detector::*;

    #[test]
    fn test_idle_state() {
        assert_eq!(IdleState::Active.name(), "Active");
        assert_eq!(IdleState::Idle.symbol(), "~");
        assert!(IdleState::Idle.allows_background_work());
        assert!(!IdleState::Active.allows_background_work());
    }

    #[test]
    fn test_activity_level() {
        assert_eq!(ActivityLevel::High.name(), "High");
        assert_eq!(ActivityLevel::Minimal.name(), "Minimal");
    }

    #[test]
    fn test_idle_config_default() {
        let config = IdleConfig::default();
        assert_eq!(config.idle_threshold_secs, 300);
        assert_eq!(config.deep_idle_threshold_secs, 900);
    }

    #[test]
    fn test_idle_tracker_new() {
        let tracker = IdleTimeTracker::new();
        assert_eq!(tracker.current_state, IdleState::Active);
        assert_eq!(tracker.tasks_completed, 0);
    }

    #[test]
    fn test_record_activity() {
        let mut tracker = IdleTimeTracker::new();
        tracker.record_activity(1000);
        assert_eq!(tracker.last_activity, 1000);
        assert_eq!(tracker.current_state, IdleState::Active);
    }

    #[test]
    fn test_check_idle_transition() {
        let mut tracker = IdleTimeTracker::new();
        tracker.record_activity(1000);

        // Not enough time passed
        let state = tracker.check_idle(1100);
        assert_eq!(state, IdleState::Active);

        // Idle threshold passed (300 sec)
        let state = tracker.check_idle(1400);
        assert_eq!(state, IdleState::Idle);
        assert_eq!(tracker.period_count(), 1);

        // Deep idle threshold passed (900 sec)
        let state = tracker.check_idle(2000);
        assert_eq!(state, IdleState::DeepIdle);
    }

    #[test]
    fn test_can_do_background_work() {
        let mut tracker = IdleTimeTracker::new();
        assert!(!tracker.can_do_background_work());

        tracker.current_state = IdleState::Idle;
        assert!(tracker.can_do_background_work());

        tracker.config.enable_background_work = false;
        assert!(!tracker.can_do_background_work());
    }

    #[test]
    fn test_quiet_hours() {
        let mut tracker = IdleTimeTracker::new();
        tracker.config.quiet_hours = Some((22, 6));

        assert!(tracker.is_quiet_hours(23));
        assert!(tracker.is_quiet_hours(2));
        assert!(!tracker.is_quiet_hours(12));
    }

    #[test]
    fn test_record_task_completed() {
        let mut tracker = IdleTimeTracker::new();
        tracker.record_activity(1000);
        tracker.check_idle(1400); // Go idle

        tracker.record_task_completed();
        assert_eq!(tracker.tasks_completed, 1);
        assert_eq!(tracker.periods.last().unwrap().tasks_completed, 1);
    }

    #[test]
    fn test_format_idle_tracker() {
        let tracker = IdleTimeTracker::new();
        let output = format_idle_tracker(&tracker);
        assert!(output.contains("Idle Time Tracker"));
        assert!(output.contains("Active"));
    }

    #[test]
    fn test_is_idle_query() {
        assert!(is_idle_query("when is the machine idle?"));
        assert!(is_idle_query("show idle time"));
        assert!(is_idle_query("background work status"));
        assert!(!is_idle_query("what is the weather?"));
    }

    #[test]
    fn test_idle_fun_fact() {
        let mut tracker = IdleTimeTracker::new();
        tracker.record_activity(1000);
        tracker.check_idle(1400);

        let fact = idle_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
