// v0.0.558: Settings Export/Import (Phase 134)
// Tests for settings export/import

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::unified_settings::UnifiedSettings;

    use super::super::exporter::{export_settings, SettingsExporter};
    use super::super::helpers::{detect_format, settings_export_fun_fact};
    use super::super::importer::import_settings;
    use super::super::types::{ExportFormat, ExportMetadata, ExportOptions, SettingsExport};

    #[test]
    fn test_export_format_display() {
        assert_eq!(format!("{}", ExportFormat::Json), "JSON");
        assert_eq!(format!("{}", ExportFormat::Toml), "TOML");
        assert_eq!(format!("{}", ExportFormat::JsonCompact), "JSON (compact)");
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Toml.extension(), "toml");
    }

    #[test]
    fn test_export_format_from_extension() {
        assert_eq!(ExportFormat::from_extension("toml"), ExportFormat::Toml);
        assert_eq!(ExportFormat::from_extension("json"), ExportFormat::Json);
        assert_eq!(ExportFormat::from_extension("unknown"), ExportFormat::Json);
    }

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::new();
        assert_eq!(options.format, ExportFormat::Json);
        assert!(!options.include_metadata);
    }

    #[test]
    fn test_export_options_builder() {
        let options = ExportOptions::new()
            .format(ExportFormat::Toml)
            .with_metadata()
            .obfuscate();
        assert_eq!(options.format, ExportFormat::Toml);
        assert!(options.include_metadata);
        assert!(options.obfuscate_sensitive);
    }

    #[test]
    fn test_export_metadata_default() {
        let meta = ExportMetadata::default();
        assert!(!meta.anna_version.is_empty());
        assert!(meta.description.is_none());
    }

    #[test]
    fn test_settings_export_new() {
        let settings = UnifiedSettings::default();
        let export = SettingsExport::new(settings);
        assert!(export.metadata.is_none());
    }

    #[test]
    fn test_settings_export_with_metadata() {
        let settings = UnifiedSettings::default();
        let export = SettingsExport::new(settings).with_metadata();
        assert!(export.metadata.is_some());
    }

    #[test]
    fn test_exporter_json() {
        let settings = UnifiedSettings::default();
        let exporter = SettingsExporter::new();
        let result = exporter.export_string(&settings);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("personality"));
    }

    #[test]
    fn test_importer_json() {
        let settings = UnifiedSettings::default();
        let exported = export_settings(&settings, ExportFormat::Json).unwrap();
        let imported = import_settings(&exported).unwrap();
        // Basic check that import worked
        assert_eq!(
            imported.personality.formality,
            settings.personality.formality
        );
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(detect_format(&PathBuf::from("test.toml")), ExportFormat::Toml);
        assert_eq!(detect_format(&PathBuf::from("test.json")), ExportFormat::Json);
    }

    #[test]
    fn test_default_filename() {
        let exporter = SettingsExporter::new();
        let filename = exporter.default_filename();
        assert!(filename.contains("anna_settings"));
        assert!(filename.contains(".json"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_export_fun_fact();
        assert!(fact.contains("export"));
    }
}
