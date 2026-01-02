// v0.0.640: Settings Report Generator - Tests (Phase 216)
// Unit tests for settings reporter

#[cfg(test)]
mod tests {
    use crate::settings_report_generator::*;

    #[test]
    fn test_report_type_display() {
        assert_eq!(format!("{}", ReportType::Summary), "summary");
        assert_eq!(format!("{}", ReportType::Detailed), "detailed");
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", ReportFormat::Text), "text");
        assert_eq!(format!("{}", ReportFormat::Json), "json");
    }

    #[test]
    fn test_config_new() {
        let c = ReporterConfig::new(ReportType::Summary);
        assert!(c.include_timestamps);
    }

    #[test]
    fn test_config_builder() {
        let c = ReporterConfig::new(ReportType::Health)
            .format(ReportFormat::Json)
            .include_stats(false);
        assert_eq!(c.format, ReportFormat::Json);
        assert!(!c.include_stats);
    }

    #[test]
    fn test_section_new() {
        let s = ReportSection::new("Test");
        assert_eq!(s.item_count(), 0);
    }

    #[test]
    fn test_section_items() {
        let mut s = ReportSection::new("Test");
        s.add_item("item1");
        assert_eq!(s.item_count(), 1);
    }

    #[test]
    fn test_report_new() {
        let r = Report::new("r1", ReportType::Summary, "Test Report");
        assert_eq!(r.section_count(), 0);
    }

    #[test]
    fn test_report_sections() {
        let mut r = Report::new("r1", ReportType::Summary, "Test");
        r.add_section(ReportSection::new("Section 1"));
        assert_eq!(r.section_count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ReporterStats::default();
        s.record(ReportType::Summary, ReportFormat::Text);
        assert_eq!(s.total_generated, 1);
    }

    #[test]
    fn test_reporter_new() {
        let r = SettingsReporter::new(ReporterConfig::new(ReportType::Summary));
        assert_eq!(r.report_count(), 0);
    }

    #[test]
    fn test_reporter_generate() {
        let mut r = SettingsReporter::new(ReporterConfig::new(ReportType::Summary));
        r.generate("r1", "Test");
        assert_eq!(r.report_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsReporterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsReporterRegistry::new();
        r.register("rep1", SettingsReporter::new(ReporterConfig::new(ReportType::Summary)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_reporter_query() {
        assert!(is_reporter_query("settings reporter"));
        assert!(!is_reporter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = reporter_fun_fact();
        assert!(fact.contains("reporter"));
    }
}
