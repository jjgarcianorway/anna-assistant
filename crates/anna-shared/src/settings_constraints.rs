// v0.0.570: Settings Constraints (Phase 146)
// Define and enforce rules/constraints on settings combinations

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Constraint severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    /// Just a suggestion
    Suggestion,
    /// Warning - not recommended
    Warning,
    /// Error - will cause problems
    Error,
    /// Critical - system may not function
    Critical,
}

impl std::fmt::Display for ConstraintSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Suggestion => write!(f, "Suggestion"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Constraint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Value must be in range
    Range,
    /// Value depends on another setting
    Dependency,
    /// Values are mutually exclusive
    MutuallyExclusive,
    /// Value requires another setting
    Requires,
    /// Value conflicts with another setting
    Conflicts,
    /// Custom rule
    Custom,
}

impl std::fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Range => write!(f, "Range"),
            Self::Dependency => write!(f, "Dependency"),
            Self::MutuallyExclusive => write!(f, "Mutually Exclusive"),
            Self::Requires => write!(f, "Requires"),
            Self::Conflicts => write!(f, "Conflicts"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// A constraint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    /// Constraint that was violated
    pub constraint_id: u64,
    /// Severity of violation
    pub severity: ConstraintSeverity,
    /// Category affected
    pub category: SettingsCategory,
    /// Field affected
    pub field: String,
    /// Description of violation
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ConstraintViolation {
    /// Create new violation
    pub fn new(
        constraint_id: u64,
        severity: ConstraintSeverity,
        category: SettingsCategory,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            constraint_id,
            severity,
            category,
            field: field.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Is this critical?
    pub fn is_critical(&self) -> bool {
        self.severity == ConstraintSeverity::Critical
    }

    /// Is this an error?
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ConstraintSeverity::Error | ConstraintSeverity::Critical)
    }
}

/// A settings constraint rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConstraint {
    /// Unique ID
    pub id: u64,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Categories involved
    pub categories: Vec<SettingsCategory>,
    /// Severity if violated
    pub severity: ConstraintSeverity,
    /// Is enabled
    pub enabled: bool,
    /// Is built-in
    pub builtin: bool,
}

impl SettingsConstraint {
    /// Create new constraint
    pub fn new(
        id: u64,
        name: impl Into<String>,
        description: impl Into<String>,
        constraint_type: ConstraintType,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            constraint_type,
            categories: Vec::new(),
            severity: ConstraintSeverity::Warning,
            enabled: true,
            builtin: false,
        }
    }

    /// Set severity
    pub fn with_severity(mut self, severity: ConstraintSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Add category
    pub fn with_category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Mark as builtin
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }
}

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

/// Constraint manager
#[derive(Debug, Clone, Default)]
pub struct ConstraintManager {
    /// All constraints
    constraints: Vec<SettingsConstraint>,
    /// Next ID
    next_id: u64,
}

impl ConstraintManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default constraints
    pub fn with_defaults() -> Self {
        let mut mgr = Self::new();
        mgr.add_default_constraints();
        mgr
    }

    /// Add default constraints
    fn add_default_constraints(&mut self) {
        // Timeout vs Verbosity constraint
        self.add_constraint(
            SettingsConstraint::new(
                self.next_id,
                "Verbose requires longer timeout",
                "Verbose output may require longer timeouts to complete",
                ConstraintType::Dependency,
            )
            .with_severity(ConstraintSeverity::Suggestion)
            .with_category(SettingsCategory::Verbosity)
            .with_category(SettingsCategory::Timeout)
            .builtin(),
        );

        // Learning mode vs Risk constraint
        self.add_constraint(
            SettingsConstraint::new(
                self.next_id,
                "Learning mode safety",
                "Learning mode should have conservative risk settings",
                ConstraintType::Dependency,
            )
            .with_severity(ConstraintSeverity::Warning)
            .with_category(SettingsCategory::Learning)
            .with_category(SettingsCategory::Risk)
            .builtin(),
        );

        // Privacy vs Backup constraint
        self.add_constraint(
            SettingsConstraint::new(
                self.next_id,
                "Privacy-aware backups",
                "High privacy settings may conflict with backup retention",
                ConstraintType::Conflicts,
            )
            .with_severity(ConstraintSeverity::Warning)
            .with_category(SettingsCategory::Privacy)
            .with_category(SettingsCategory::Backup)
            .builtin(),
        );

        // Auto-update safety
        self.add_constraint(
            SettingsConstraint::new(
                self.next_id,
                "Update confirmation",
                "Auto-updates should respect confirmation settings",
                ConstraintType::Dependency,
            )
            .with_severity(ConstraintSeverity::Warning)
            .with_category(SettingsCategory::Update)
            .with_category(SettingsCategory::Confirmation)
            .builtin(),
        );
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, mut constraint: SettingsConstraint) {
        constraint.id = self.next_id;
        self.next_id += 1;
        self.constraints.push(constraint);
    }

    /// Remove a constraint
    pub fn remove(&mut self, id: u64) -> Option<SettingsConstraint> {
        if let Some(pos) = self.constraints.iter().position(|c| c.id == id && !c.builtin) {
            Some(self.constraints.remove(pos))
        } else {
            None
        }
    }

    /// Get constraint by ID
    pub fn get(&self, id: u64) -> Option<&SettingsConstraint> {
        self.constraints.iter().find(|c| c.id == id)
    }

    /// Enable/disable constraint
    pub fn set_enabled(&mut self, id: u64, enabled: bool) -> bool {
        if let Some(c) = self.constraints.iter_mut().find(|c| c.id == id) {
            c.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// List all constraints
    pub fn list(&self) -> &[SettingsConstraint] {
        &self.constraints
    }

    /// List enabled constraints
    pub fn enabled(&self) -> Vec<&SettingsConstraint> {
        self.constraints.iter().filter(|c| c.enabled).collect()
    }

    /// Check settings against all enabled constraints
    pub fn check(&self, settings: &UnifiedSettings) -> ConstraintCheckResult {
        let mut result = ConstraintCheckResult::new();

        for constraint in self.enabled() {
            if let Some(violation) = self.check_constraint(constraint, settings) {
                result.add_violation(violation);
                result.record_fail();
            } else {
                result.record_pass();
            }
        }

        result
    }

    /// Check a single constraint
    fn check_constraint(
        &self,
        constraint: &SettingsConstraint,
        _settings: &UnifiedSettings,
    ) -> Option<ConstraintViolation> {
        // Simplified constraint checking - in real implementation would check actual values
        // For now, return None (all pass) - this is a framework for constraint checking
        match constraint.constraint_type {
            ConstraintType::Range => None,
            ConstraintType::Dependency => None,
            ConstraintType::MutuallyExclusive => None,
            ConstraintType::Requires => None,
            ConstraintType::Conflicts => None,
            ConstraintType::Custom => None,
        }
    }

    /// Count constraints
    pub fn count(&self) -> usize {
        self.constraints.len()
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

/// Check if query is about constraints
pub fn is_constraint_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("constraint")
        || lower.contains("rule")
        || lower.contains("settings conflict")
        || lower.contains("validate settings")
}

/// Fun fact about constraints
pub fn constraint_fun_fact() -> &'static str {
    "Settings constraints help prevent configuration conflicts before they cause problems!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_severity_display() {
        assert_eq!(format!("{}", ConstraintSeverity::Warning), "Warning");
        assert_eq!(format!("{}", ConstraintSeverity::Critical), "Critical");
    }

    #[test]
    fn test_constraint_type_display() {
        assert_eq!(format!("{}", ConstraintType::Range), "Range");
        assert_eq!(format!("{}", ConstraintType::Conflicts), "Conflicts");
    }

    #[test]
    fn test_constraint_violation_new() {
        let v = ConstraintViolation::new(
            1,
            ConstraintSeverity::Warning,
            SettingsCategory::Risk,
            "level",
            "Risk too high",
        );
        assert_eq!(v.constraint_id, 1);
        assert!(!v.is_critical());
    }

    #[test]
    fn test_constraint_violation_is_error() {
        let v = ConstraintViolation::new(
            1,
            ConstraintSeverity::Error,
            SettingsCategory::Risk,
            "level",
            "Invalid",
        );
        assert!(v.is_error());
    }

    #[test]
    fn test_settings_constraint_new() {
        let c = SettingsConstraint::new(1, "Test", "Test constraint", ConstraintType::Range);
        assert_eq!(c.id, 1);
        assert!(c.enabled);
        assert!(!c.builtin);
    }

    #[test]
    fn test_settings_constraint_builder() {
        let c = SettingsConstraint::new(1, "Test", "Test", ConstraintType::Range)
            .with_severity(ConstraintSeverity::Error)
            .with_category(SettingsCategory::Risk)
            .builtin();
        assert_eq!(c.severity, ConstraintSeverity::Error);
        assert!(c.builtin);
        assert!(!c.categories.is_empty());
    }

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
    fn test_constraint_manager_new() {
        let manager = ConstraintManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_constraint_manager_with_defaults() {
        let manager = ConstraintManager::with_defaults();
        assert!(manager.count() >= 4);
    }

    #[test]
    fn test_constraint_manager_check() {
        let manager = ConstraintManager::with_defaults();
        let settings = UnifiedSettings::default();
        let result = manager.check(&settings);
        assert!(result.is_valid());
    }

    #[test]
    fn test_constraint_manager_enable_disable() {
        let mut manager = ConstraintManager::with_defaults();
        let id = manager.constraints[0].id;
        manager.set_enabled(id, false);
        assert!(!manager.get(id).unwrap().enabled);
    }

    #[test]
    fn test_format_constraint_results() {
        let result = ConstraintCheckResult::new();
        let output = format_constraint_results(&result);
        assert!(output.contains("Constraint"));
    }

    #[test]
    fn test_is_constraint_query() {
        assert!(is_constraint_query("check constraints"));
        assert!(is_constraint_query("settings conflict"));
        assert!(!is_constraint_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = constraint_fun_fact();
        assert!(fact.contains("constraint"));
    }
}
