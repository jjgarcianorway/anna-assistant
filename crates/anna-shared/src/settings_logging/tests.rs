// v0.0.585: Settings Logging - Tests (Phase 161)
// Test suite for settings logging functionality

#[cfg(test)]
mod tests {
    use crate::unified_settings::SettingsCategory;
    use super::super::*;

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }

    #[test]
    fn test_log_target_display() {
        assert_eq!(format!("{}", LogTarget::Core), "core");
        assert_eq!(format!("{}", LogTarget::Api), "api");
    }

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new(LogLevel::Info, LogTarget::Core, "Test message");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
    }

    #[test]
    fn test_log_entry_builder() {
        let entry = LogEntry::new(LogLevel::Error, LogTarget::Persistence, "Failed")
            .category(SettingsCategory::Backup)
            .error("IO error");
        assert!(entry.is_error());
        assert!(entry.error.is_some());
    }

    #[test]
    fn test_log_filter_new() {
        let filter = LogFilter::new();
        assert!(filter.min_level.is_none());
    }

    #[test]
    fn test_log_filter_matches() {
        let filter = LogFilter::new().level(LogLevel::Warn);
        let info = LogEntry::new(LogLevel::Info, LogTarget::Core, "Info");
        let warn = LogEntry::new(LogLevel::Warn, LogTarget::Core, "Warn");
        assert!(!filter.matches(&info));
        assert!(filter.matches(&warn));
    }

    #[test]
    fn test_settings_logger_new() {
        let logger = SettingsLogger::new();
        assert_eq!(logger.count(), 0);
    }

    #[test]
    fn test_settings_logger_log() {
        let mut logger = SettingsLogger::new();
        logger.info(LogTarget::Core, "Test");
        assert_eq!(logger.count(), 1);
    }

    #[test]
    fn test_settings_logger_error() {
        let mut logger = SettingsLogger::new();
        logger.error(LogTarget::Api, "Failed");
        assert_eq!(logger.error_count(), 1);
    }

    #[test]
    fn test_settings_logger_query() {
        let mut logger = SettingsLogger::new();
        logger.info(LogTarget::Core, "Test 1");
        logger.warn(LogTarget::Api, "Test 2");
        let filter = LogFilter::new().target(LogTarget::Api);
        let results = logger.query(&filter);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_format_logs() {
        let logger = SettingsLogger::new();
        let output = format_logs(&logger, 5);
        assert!(output.contains("Logs"));
    }

    #[test]
    fn test_is_logging_query() {
        assert!(is_logging_query("show logs"));
        assert!(is_logging_query("debug mode"));
        assert!(!is_logging_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_logging_fun_fact();
        assert!(fact.contains("log"));
    }
}
