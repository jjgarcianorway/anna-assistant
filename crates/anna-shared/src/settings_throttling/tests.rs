// v0.0.589: Settings Throttling - Tests (Phase 165)
// Unit tests for throttling functionality

#[cfg(test)]
mod tests {
    use crate::settings_throttling::*;

    #[test]
    fn test_throttle_action_display() {
        assert_eq!(format!("{}", ThrottleAction::Read), "read");
        assert_eq!(format!("{}", ThrottleAction::Write), "write");
    }

    #[test]
    fn test_throttle_result_display() {
        assert_eq!(format!("{}", ThrottleResult::Allowed), "allowed");
        assert_eq!(format!("{}", ThrottleResult::Limited), "limited");
    }

    #[test]
    fn test_rate_limit_default() {
        let limit = RateLimit::default();
        assert_eq!(limit.max_requests, 100);
        assert_eq!(limit.window_secs, 60);
    }

    #[test]
    fn test_rate_limit_new() {
        let limit = RateLimit::new(50, 30).burst(5);
        assert_eq!(limit.max_requests, 50);
        assert_eq!(limit.burst, 5);
    }

    #[test]
    fn test_throttler_new() {
        let throttler = SettingsThrottler::new();
        assert!(throttler.is_enabled());
    }

    #[test]
    fn test_throttler_set_limit() {
        let mut throttler = SettingsThrottler::new();
        throttler.set_limit(ThrottleAction::Write, RateLimit::new(10, 60));
        assert!(throttler.get_limit(ThrottleAction::Write).is_some());
    }

    #[test]
    fn test_throttler_block() {
        let mut throttler = SettingsThrottler::new();
        throttler.block(ThrottleAction::Import);
        assert!(throttler.is_blocked(ThrottleAction::Import));
        throttler.unblock(ThrottleAction::Import);
        assert!(!throttler.is_blocked(ThrottleAction::Import));
    }

    #[test]
    fn test_throttler_check_allowed() {
        let mut throttler = SettingsThrottler::new();
        let result = throttler.check(ThrottleAction::Read, None);
        assert_eq!(result, ThrottleResult::Allowed);
    }

    #[test]
    fn test_throttler_check_blocked() {
        let mut throttler = SettingsThrottler::new();
        throttler.block(ThrottleAction::Write);
        let result = throttler.check(ThrottleAction::Write, None);
        assert_eq!(result, ThrottleResult::Blocked);
    }

    #[test]
    fn test_throttler_disable() {
        let mut throttler = SettingsThrottler::new();
        throttler.disable();
        assert!(!throttler.is_enabled());
    }

    #[test]
    fn test_format_throttle() {
        let throttler = SettingsThrottler::new();
        let output = format_throttle(&throttler);
        assert!(output.contains("Throttling"));
    }

    #[test]
    fn test_is_throttling_query() {
        assert!(is_throttling_query("enable rate limiting"));
        assert!(is_throttling_query("throttle writes"));
        assert!(!is_throttling_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_throttling_fun_fact();
        assert!(fact.contains("throttle"));
    }
}
