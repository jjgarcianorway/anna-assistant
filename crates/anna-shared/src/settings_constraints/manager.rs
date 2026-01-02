// v0.0.570: Constraint Manager (Phase 146)
// Manages and validates settings constraints

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::result::ConstraintCheckResult;
use super::types::{ConstraintSeverity, ConstraintType, ConstraintViolation, SettingsConstraint};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
