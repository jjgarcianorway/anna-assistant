// v0.0.650: Settings Converter Module (Phase 226)
// Converter for transforming settings between formats

mod formats;
mod config;
mod result;
mod stats;
mod converter;
mod registry;
mod utils;

// Re-export all public types to maintain the same API
pub use formats::{SourceFormat, TargetFormat};
pub use config::ConverterConfig;
pub use result::ConversionResult;
pub use stats::ConverterStats;
pub use converter::SettingsConverter;
pub use registry::{SettingsConverterRegistry, format_converter_registry};
pub use utils::{is_converter_query, converter_fun_fact};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_format_display() {
        assert_eq!(format!("{}", SourceFormat::Json), "json");
        assert_eq!(format!("{}", SourceFormat::Toml), "toml");
    }

    #[test]
    fn test_target_format_display() {
        assert_eq!(format!("{}", TargetFormat::Yaml), "yaml");
        assert_eq!(format!("{}", TargetFormat::Env), "env");
    }

    #[test]
    fn test_config_new() {
        let c = ConverterConfig::new(SourceFormat::Json, TargetFormat::Toml);
        assert!(c.pretty);
    }

    #[test]
    fn test_config_builder() {
        let c = ConverterConfig::new(SourceFormat::Toml, TargetFormat::Yaml)
            .preserve_comments(true)
            .pretty(false);
        assert!(c.preserve_comments);
        assert!(!c.pretty);
    }

    #[test]
    fn test_result_success() {
        let r = ConversionResult::success("data", SourceFormat::Json, TargetFormat::Toml, 5);
        assert!(r.success);
        assert_eq!(r.key_count, 5);
    }

    #[test]
    fn test_result_failure() {
        let r = ConversionResult::failure(SourceFormat::Json, TargetFormat::Toml);
        assert!(!r.success);
        assert!(r.is_empty());
    }

    #[test]
    fn test_stats_record() {
        let mut s = ConverterStats::default();
        s.record(SourceFormat::Json, TargetFormat::Toml, true);
        s.record(SourceFormat::Toml, TargetFormat::Yaml, false);
        assert_eq!(s.total_conversions, 2);
        assert_eq!(s.successful, 1);
    }

    #[test]
    fn test_converter_new() {
        let c = SettingsConverter::new(ConverterConfig::new(SourceFormat::Json, TargetFormat::Toml));
        assert_eq!(c.result_count(), 0);
    }

    #[test]
    fn test_converter_json_to_toml() {
        let mut c = SettingsConverter::new(ConverterConfig::new(SourceFormat::Json, TargetFormat::Toml));
        let r = c.convert(r#"{"key":"value"}"#);
        assert!(r.success);
        assert!(r.data.contains("key = \"value\""));
    }

    #[test]
    fn test_converter_toml_to_yaml() {
        let mut c = SettingsConverter::new(ConverterConfig::new(SourceFormat::Toml, TargetFormat::Yaml));
        let r = c.convert("key = \"value\"");
        assert!(r.success);
        assert!(r.data.contains("key: value"));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsConverterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsConverterRegistry::new();
        r.register("conv1", SettingsConverter::new(ConverterConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_converter_query() {
        assert!(is_converter_query("settings converter"));
        assert!(!is_converter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = converter_fun_fact();
        assert!(fact.contains("converter"));
    }
}
