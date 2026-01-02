// v0.0.557: Settings Validation Formatting
// Display and formatting utilities for validation results

use super::types::{ValidationResult, ValidationSeverity};

/// Format validation result for display
pub fn format_validation_result(result: &ValidationResult) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Validation ===\n\n");

    if result.is_valid() && !result.has_issues() {
        output.push_str("All settings are valid.\n");
        return output;
    }

    let errors = result.count_by_severity(ValidationSeverity::Error);
    let warnings = result.count_by_severity(ValidationSeverity::Warning);
    let infos = result.count_by_severity(ValidationSeverity::Info);

    output.push_str(&format!(
        "Found {} issues ({} errors, {} warnings, {} info)\n\n",
        result.total_count(),
        errors,
        warnings,
        infos
    ));

    for issue in &result.issues {
        output.push_str(&format!(
            "[{}] {} - {}: {}\n",
            issue.severity, issue.category, issue.field, issue.message
        ));
        if let Some(suggestion) = &issue.suggestion {
            output.push_str(&format!("    Suggestion: {}\n", suggestion));
        }
    }

    output
}

/// Fun fact about settings validation
pub fn settings_validation_fun_fact() -> &'static str {
    "Anna validates your settings to catch conflicts before they cause problems!"
}
