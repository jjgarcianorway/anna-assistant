// v0.0.582: Settings Permissions (Phase 158)
// Role definition and management

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{CategoryPermission, PermissionLevel};

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRole {
    /// Role name
    pub name: String,
    /// Description
    pub description: String,
    /// Default level for all categories
    pub default_level: PermissionLevel,
    /// Category overrides
    pub overrides: Vec<CategoryPermission>,
    /// Built-in role
    pub builtin: bool,
}

impl PermissionRole {
    /// Create new role
    pub fn new(name: impl Into<String>, default_level: PermissionLevel) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            default_level,
            overrides: Vec::new(),
            builtin: false,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Mark as built-in
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }

    /// Add category override
    pub fn override_category(mut self, perm: CategoryPermission) -> Self {
        self.overrides.push(perm);
        self
    }

    /// Get permission for category
    pub fn permission_for(&self, category: SettingsCategory) -> PermissionLevel {
        self.overrides
            .iter()
            .find(|p| p.category == category)
            .map(|p| p.level)
            .unwrap_or(self.default_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_role_new() {
        let role = PermissionRole::new("TestRole", PermissionLevel::Write);
        assert_eq!(role.name, "TestRole");
        assert_eq!(role.default_level, PermissionLevel::Write);
    }

    #[test]
    fn test_permission_role_override() {
        let role = PermissionRole::new("Test", PermissionLevel::Write)
            .override_category(
                CategoryPermission::new(SettingsCategory::Privacy, PermissionLevel::Read)
            );
        assert_eq!(role.permission_for(SettingsCategory::Privacy), PermissionLevel::Read);
        assert_eq!(role.permission_for(SettingsCategory::Risk), PermissionLevel::Write);
    }
}
