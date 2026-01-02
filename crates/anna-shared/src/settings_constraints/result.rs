// v0.0.570: Constraint Check Results (Phase 146)
// Types and formatting for constraint checking results

use super::types::{ConstraintSeverity, ConstraintViolation};

/// Constraint check result
#[derive(Debug, Clone, Default)]
pub struct ConstraintCheckResult {
    /// Violations found
    pub violations: Vec<ConstraintViolation>,
    /// Constraints checked
    pub checked_count: usize,
    /// Passed constraints
    pub passed_count: usize,
}

impl ConstraintCheckResult {
    /// Create empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Add violation
    pub fn add_violation(&mut self, violation: ConstraintViolation) {
        self.violations.push(violation);
    }

    /// Record passed constraint
    pub fn record_pass(&mut self) {
        self.checked_count += 1;
        self.passed_count += 1;
    }

    /// Record failed constraint
    pub fn record_fail(&mut self) {
        self.checked_count += 1;
    }

    /// Is valid (no errors or critical)?
    pub fn is_valid(&self) -> bool {
        !self.violations.iter().any(|v| v.is_error())
    }

    /// Has warnings?
    pub fn has_warnings(&self) -> bool {
        self.violations.iter().any(|v| v.severity == ConstraintSeverity::Warning)
    }

    /// Has suggestions?
    pub fn has_suggestions(&self) -> bool {
        self.violations.iter().any(|v| v.severity == ConstraintSeverity::Suggestion)
    }

    /// Count by severity
    pub fn count_by_severity(&self, severity: ConstraintSeverity) -> usize {
        self.violations.iter().filter(|v| v.severity == severity).count()
    }

    /// Get critical violations
    pub fn critical(&self) -> Vec<&ConstraintViolation> {
        self.violations.iter().filter(|v| v.is_critical()).collect()
    }

    /// Get errors
    pub fn errors(&self) -> Vec<&ConstraintViolation> {
        self.violations.iter().filter(|v| v.is_error()).collect()
    }
}

/// Format constraint check results
pub fn format_constraint_results(result: &ConstraintCheckResult) -> String {
    let mut output = String::new();

    output.push_str("=== Constraint Check Results ===\n\n");

    output.push_str(&format!(
        "Checked: {} | Passed: {} | Failed: {}\n\n",
        result.checked_count,
        result.passed_count,
        result.violations.len()
    ));

    if result.violations.is_empty() {
        output.push_str("All constraints satisfied.\n");
        return output;
    }

    // Group by severity
    for severity in [
        ConstraintSeverity::Critical,
        ConstraintSeverity::Error,
        ConstraintSeverity::Warning,
        ConstraintSeverity::Suggestion,
    ] {
        let violations: Vec<_> = result.violations.iter()
            .filter(|v| v.severity == severity)
            .collect();

        if !violations.is_empty() {
            output.push_str(&format!("{}:\n", severity));
            for v in violations {
                output.push_str(&format!("  • {}: {}\n", v.field, v.message));
                if let Some(suggestion) = &v.suggestion {
                    output.push_str(&format!("    Suggestion: {}\n", suggestion));
                }
            }
            output.push('\n');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_constraints::types::ConstraintSeverity;
    use crate::unified_settings::SettingsCategory;

    #[test]
    fn test_constraint_check_result_new() {
        let result = ConstraintCheckResult::new();
        assert!(result.is_valid());
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_constraint_check_result_add_violation() {
        let mut result = ConstraintCheckResult::new();
        result.add_violation(ConstraintViolation::new(
            1,
            ConstraintSeverity::Warning,
            SettingsCategory::Risk,
            "test",
            "test",
        ));
        assert!(result.has_warnings());
    }

    #[test]
    fn test_format_constraint_results() {
        let result = ConstraintCheckResult::new();
        let output = format_constraint_results(&result);
        assert!(output.contains("Constraint"));
    }
}
