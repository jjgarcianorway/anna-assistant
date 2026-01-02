// v0.0.582: Settings Permissions (Phase 158)
// Permission manager for controlling access to settings

use std::collections::HashSet;

use crate::unified_settings::SettingsCategory;
use super::types::{CategoryPermission, PermissionAction, PermissionLevel, PermissionResult};
use super::role::PermissionRole;

/// Permission manager
#[derive(Debug, Clone, Default)]
pub struct PermissionManager {
    /// Available roles
    roles: Vec<PermissionRole>,
    /// Active role name
    active_role: Option<String>,
    /// Global locks
    locked_categories: HashSet<SettingsCategory>,
}

impl PermissionManager {
    /// Create new manager with default roles
    pub fn new() -> Self {
        let mut mgr = Self::default();
        mgr.add_builtin_roles();
        mgr
    }

    fn add_builtin_roles(&mut self) {
        // Viewer role
        self.roles.push(
            PermissionRole::new("Viewer", PermissionLevel::Read)
                .description("Read-only access to all settings")
                .builtin()
        );

        // Editor role
        self.roles.push(
            PermissionRole::new("Editor", PermissionLevel::Write)
                .description("Can modify most settings")
                .builtin()
                .override_category(
                    CategoryPermission::new(SettingsCategory::Privacy, PermissionLevel::Read)
                )
        );

        // Admin role
        self.roles.push(
            PermissionRole::new("Admin", PermissionLevel::Admin)
                .description("Full access to all settings")
                .builtin()
        );

        // Set default active role
        self.active_role = Some("Editor".to_string());
    }

    /// Add custom role
    pub fn add_role(&mut self, role: PermissionRole) {
        self.roles.push(role);
    }

    /// Remove role (non-builtin only)
    pub fn remove_role(&mut self, name: &str) -> bool {
        let len_before = self.roles.len();
        self.roles.retain(|r| r.name != name || r.builtin);
        self.roles.len() < len_before
    }

    /// Set active role
    pub fn set_active_role(&mut self, name: &str) -> bool {
        if self.roles.iter().any(|r| r.name == name) {
            self.active_role = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// Get active role
    pub fn active_role(&self) -> Option<&PermissionRole> {
        self.active_role.as_ref().and_then(|name| {
            self.roles.iter().find(|r| &r.name == name)
        })
    }

    /// Check permission
    pub fn check(&self, category: SettingsCategory, action: PermissionAction) -> PermissionResult {
        // Check global lock
        if self.locked_categories.contains(&category) && action != PermissionAction::View {
            return PermissionResult::Denied;
        }

        // Get active role
        let role = match self.active_role() {
            Some(r) => r,
            None => return PermissionResult::Denied,
        };

        // Get category permission
        let level = role.permission_for(category);
        let perm = CategoryPermission::new(category, level);

        if perm.can(action) {
            PermissionResult::Allowed
        } else {
            PermissionResult::Denied
        }
    }

    /// Lock a category
    pub fn lock_category(&mut self, category: SettingsCategory) {
        self.locked_categories.insert(category);
    }

    /// Unlock a category
    pub fn unlock_category(&mut self, category: SettingsCategory) {
        self.locked_categories.remove(&category);
    }

    /// Get all roles
    pub fn roles(&self) -> &[PermissionRole] {
        &self.roles
    }

    /// Get role by name
    pub fn get_role(&self, name: &str) -> Option<&PermissionRole> {
        self.roles.iter().find(|r| r.name == name)
    }

    /// Check if category is locked
    pub fn is_locked(&self, category: SettingsCategory) -> bool {
        self.locked_categories.contains(&category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_manager_new() {
        let mgr = PermissionManager::new();
        assert!(mgr.roles().len() >= 3);
    }

    #[test]
    fn test_permission_manager_check_allowed() {
        let mgr = PermissionManager::new();
        let result = mgr.check(SettingsCategory::Personality, PermissionAction::Modify);
        assert_eq!(result, PermissionResult::Allowed);
    }

    #[test]
    fn test_permission_manager_lock_category() {
        let mut mgr = PermissionManager::new();
        mgr.lock_category(SettingsCategory::Privacy);
        assert!(mgr.is_locked(SettingsCategory::Privacy));
        let result = mgr.check(SettingsCategory::Privacy, PermissionAction::Modify);
        assert_eq!(result, PermissionResult::Denied);
    }
}
