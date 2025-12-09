//! Tests for render module (v0.0.203).

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::render::{
        determine_risk_level, format_time_delta, generate_case_id, RiskLevel, Verbosity,
    };

    #[test]
    fn test_generate_case_id() {
        let id = generate_case_id(42);
        assert!(id.starts_with("CN-"));
        assert!(id.contains("-0042"));
    }

    #[test]
    fn test_format_time_delta() {
        assert_eq!(format_time_delta(Duration::seconds(30)), "just now");
        assert_eq!(format_time_delta(Duration::seconds(120)), "2 minutes");
        assert_eq!(format_time_delta(Duration::seconds(3600)), "1 hour");
        assert_eq!(format_time_delta(Duration::seconds(86400)), "1 day");
    }

    #[test]
    fn test_risk_level_detection() {
        assert_eq!(determine_risk_level("pacman -S vim"), RiskLevel::High);
        assert_eq!(determine_risk_level("edit ~/.vimrc"), RiskLevel::Medium);
        assert_eq!(determine_risk_level("memory usage is 4GB"), RiskLevel::Low);
    }

    #[test]
    fn test_verbosity_from_str() {
        assert_eq!(Verbosity::from_str("low"), Verbosity::Low);
        assert_eq!(Verbosity::from_str("HIGH"), Verbosity::High);
        assert_eq!(Verbosity::from_str("normal"), Verbosity::Normal);
        assert_eq!(Verbosity::from_str("invalid"), Verbosity::Normal);
    }
}
