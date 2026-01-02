// v0.0.558: Settings Export/Import (Phase 134)
// Helper functions for settings export/import

use std::path::PathBuf;

use super::types::ExportFormat;

/// Detect format from file path
pub fn detect_format(path: &PathBuf) -> ExportFormat {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(ExportFormat::from_extension)
        .unwrap_or_default()
}

/// Format export info for display
pub fn format_export_info(format: ExportFormat, path: Option<&PathBuf>) -> String {
    let mut output = String::new();
    output.push_str(&format!("Format: {}\n", format));
    if let Some(p) = path {
        output.push_str(&format!("Path: {}\n", p.display()));
    }
    output
}

/// Fun fact about settings export
pub fn settings_export_fun_fact() -> &'static str {
    "Anna can export your settings to share with friends or backup to a USB drive!"
}
