// v0.0.644: Settings Formatter Tests (Phase 220)
// Tests for settings formatter module

#[cfg(test)]
mod tests {
    use crate::settings_formatter::{
        formatter::SettingsFormatter,
        formatter_fun_fact, format_formatter_registry, is_formatter_query,
        registry::SettingsFormatterRegistry,
        types::{FormatResult, FormatStyle, FormatType, FormatterConfig, FormatterStats},
    };

    #[test]
    fn test_format_type_display() {
        assert_eq!(format!("{}", FormatType::Plain), "plain");
        assert_eq!(format!("{}", FormatType::Json), "json");
    }

    #[test]
    fn test_format_style_display() {
        assert_eq!(format!("{}", FormatStyle::Compact), "compact");
        assert_eq!(format!("{}", FormatStyle::Pretty), "pretty");
    }

    #[test]
    fn test_config_new() {
        let c = FormatterConfig::new(FormatType::Json);
        assert_eq!(c.format_type, FormatType::Json);
    }

    #[test]
    fn test_config_builder() {
        let c = FormatterConfig::new(FormatType::Plain)
            .format_style(FormatStyle::Pretty)
            .indent_size(4);
        assert_eq!(c.format_style, FormatStyle::Pretty);
        assert_eq!(c.indent_size, 4);
    }

    #[test]
    fn test_result_new() {
        let r = FormatResult::new("test", "\"test\"", FormatType::Json, FormatStyle::Compact);
        assert!(r.was_transformed());
    }

    #[test]
    fn test_result_unchanged() {
        let r = FormatResult::new("test", "test", FormatType::Plain, FormatStyle::Compact);
        assert!(!r.was_transformed());
    }

    #[test]
    fn test_stats_record() {
        let mut s = FormatterStats::default();
        s.record(FormatType::Json, FormatStyle::Compact, 10);
        s.record(FormatType::Plain, FormatStyle::Pretty, 20);
        assert_eq!(s.total_formatted, 2);
        assert_eq!(s.total_output_bytes, 30);
    }

    #[test]
    fn test_formatter_new() {
        let f = SettingsFormatter::new(FormatterConfig::new(FormatType::Plain));
        assert_eq!(f.result_count(), 0);
    }

    #[test]
    fn test_formatter_format_plain() {
        let mut f = SettingsFormatter::new(FormatterConfig::new(FormatType::Plain));
        let r = f.format("test");
        assert_eq!(r.formatted, "test");
    }

    #[test]
    fn test_formatter_format_json() {
        let mut f = SettingsFormatter::new(FormatterConfig::new(FormatType::Json));
        let r = f.format("test");
        assert_eq!(r.formatted, "\"test\"");
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsFormatterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsFormatterRegistry::new();
        r.register("fmt1", SettingsFormatter::new(FormatterConfig::new(FormatType::Plain)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_formatter_query() {
        assert!(is_formatter_query("settings formatter"));
        assert!(!is_formatter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = formatter_fun_fact();
        assert!(fact.contains("formatter"));
    }
}
