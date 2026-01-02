// v0.0.684: Settings Scanner - Tests
// Test suite for settings scanner

#[cfg(test)]
mod tests {
    use super::super::registry::{format_scanner_registry, ScannerRegistry};
    use super::super::scanner::SettingsScanner;
    use super::super::types::{ScanFinding, ScanResult, ScanSeverity, ScanType, ScannerConfig, ScannerStats};
    use super::super::utils::{is_scanner_query, scanner_fun_fact};
    use std::collections::HashMap;

    #[test]
    fn test_scan_type_display() {
        assert_eq!(format!("{}", ScanType::Pattern), "pattern");
        assert_eq!(format!("{}", ScanType::Anomaly), "anomaly");
    }

    #[test]
    fn test_scan_severity_display() {
        assert_eq!(format!("{}", ScanSeverity::Info), "info");
        assert_eq!(format!("{}", ScanSeverity::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = ScannerConfig::new(ScanType::Pattern);
        assert!(c.case_insensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = ScannerConfig::new(ScanType::Anomaly)
            .min_severity(ScanSeverity::Warning)
            .pattern("test");
        assert_eq!(c.min_severity, ScanSeverity::Warning);
        assert_eq!(c.pattern, Some("test".to_string()));
    }

    #[test]
    fn test_finding_new() {
        let f = ScanFinding::new("key", "value", ScanType::Pattern, ScanSeverity::Info, "test");
        assert_eq!(f.key, "key");
        assert!(!f.is_critical());
    }

    #[test]
    fn test_finding_is_error_or_worse() {
        let error = ScanFinding::new("k", "v", ScanType::Anomaly, ScanSeverity::Error, "test");
        assert!(error.is_error_or_worse());

        let info = ScanFinding::new("k", "v", ScanType::Pattern, ScanSeverity::Info, "test");
        assert!(!info.is_error_or_worse());
    }

    #[test]
    fn test_result_new() {
        let r = ScanResult::new(vec![], 10, ScanType::Pattern);
        assert_eq!(r.total_scanned, 10);
        assert!(!r.has_findings());
    }

    #[test]
    fn test_result_count_by_severity() {
        let findings = vec![
            ScanFinding::new("k1", "v1", ScanType::Pattern, ScanSeverity::Info, "test"),
            ScanFinding::new("k2", "v2", ScanType::Pattern, ScanSeverity::Warning, "test"),
        ];
        let r = ScanResult::new(findings, 10, ScanType::Pattern);
        assert_eq!(r.count_by_severity(ScanSeverity::Info), 1);
        assert_eq!(r.count_by_severity(ScanSeverity::Warning), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ScannerStats::default();
        let r = ScanResult::new(vec![ScanFinding::new("k", "v", ScanType::Pattern, ScanSeverity::Info, "t")], 5, ScanType::Pattern);
        s.record(&r);
        assert_eq!(s.total_scans, 1);
        assert_eq!(s.total_findings, 1);
    }

    #[test]
    fn test_scanner_new() {
        let s = SettingsScanner::new(ScannerConfig::default());
        assert_eq!(s.stats().total_scans, 0);
    }

    #[test]
    fn test_scanner_scan_pattern() {
        let mut s = SettingsScanner::new(ScannerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = s.scan_pattern(&settings, "app");
        assert_eq!(result.total_findings, 1);
    }

    #[test]
    fn test_scanner_scan_empty() {
        let mut s = SettingsScanner::new(ScannerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("filled".to_string(), "value".to_string());
        settings.insert("empty".to_string(), "".to_string());

        let result = s.scan_empty(&settings);
        assert_eq!(result.total_findings, 1);
    }

    #[test]
    fn test_scanner_scan_duplicates() {
        let mut s = SettingsScanner::new(ScannerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("key1".to_string(), "same_value".to_string());
        settings.insert("key2".to_string(), "same_value".to_string());
        settings.insert("key3".to_string(), "different".to_string());

        let result = s.scan_duplicates(&settings);
        assert_eq!(result.total_findings, 2); // key1 and key2 are duplicates
    }

    #[test]
    fn test_registry_new() {
        let r = ScannerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ScannerRegistry::new();
        r.register("s1", SettingsScanner::new(ScannerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_scanner_query() {
        assert!(is_scanner_query("scan settings"));
        assert!(!is_scanner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = scanner_fun_fact();
        assert!(fact.contains("scanner"));
    }
}
