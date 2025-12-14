// v0.0.582: Settings Permissions (Phase 158)
// Access control and permissions for settings

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::unified_settings::SettingsCategory;

/// Permission level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// No access
    None = 0,
    /// Read only
    Read = 1,
    /// Read and write
    Write = 2,
    /// Full admin access
    Admin = 3,
}

impl Default for PermissionLevel {
    fn default() -> Self {
        Self::Read
    }
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Read => write!(f, "Read"),
            Self::Write => write!(f, "Write"),
            Self::Admin => write!(f, "Admin"),
        }
    }
}

/// Permission action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionAction {
    /// View setting
    View,
    /// Modify setting
    Modify,
    /// Reset setting
    Reset,
    /// Export settings
    Export,
    /// Import settings
    Import,
    /// Manage permissions
    ManagePermissions,
}

impl std::fmt::Display for PermissionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::View => write!(f, "View"),
            Self::Modify => write!(f, "Modify"),
            Self::Reset => write!(f, "Reset"),
            Self::Export => write!(f, "Export"),
            Self::Import => write!(f, "Import"),
            Self::ManagePermissions => write!(f, "Manage Permissions"),
        }
    }
}

/// Permission check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionResult {
    /// Allowed
    Allowed,
    /// Denied
    Denied,
    /// Requires elevation
    RequiresElevation,
}

impl std::fmt::Display for PermissionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allowed => write!(f, "Allowed"),
            Self::Denied => write!(f, "Denied"),
            Self::RequiresElevation => write!(f, "Requires Elevation"),
        }
    }
}

/// Category permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPermission {
    /// Category
    pub category: SettingsCategory,
    /// Permission level
    pub level: PermissionLevel,
    /// Allowed actions
    pub allowed_actions: HashSet<PermissionAction>,
    /// Locked (cannot change)
    pub locked: bool,
}

impl CategoryPermission {
    /// Create new permission
    pub fn new(category: SettingsCategory, level: PermissionLevel) -> Self {
        let mut allowed = HashSet::new();
        match level {
            PermissionLevel::None => {}
            PermissionLevel::Read => {
                allowed.insert(PermissionAction::View);
            }
            PermissionLevel::Write => {
                allowed.insert(PermissionAction::View);
                allowed.insert(PermissionAction::Modify);
                allowed.insert(PermissionAction::Reset);
            }
            PermissionLevel::Admin => {
                allowed.insert(PermissionAction::View);
                allowed.insert(PermissionAction::Modify);
                allowed.insert(PermissionAction::Reset);
                allowed.insert(PermissionAction::Export);
                allowed.insert(PermissionAction::Import);
                allowed.insert(PermissionAction::ManagePermissions);
            }
        }

        Self {
            category,
            level,
            allowed_actions: allowed,
            locked: false,
        }
    }

    /// Lock permission
    pub fn lock(mut self) -> Self {
        self.locked = true;
        self
    }

    /// Check if action is allowed
    pub fn can(&self, action: PermissionAction) -> bool {
        self.allowed_actions.contains(&action)
    }
}

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

/// Format permissions for display
pub fn format_permissions(manager: &PermissionManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Permissions ===\n\n");

    if let Some(role) = manager.active_role() {
        output.push_str(&format!("Active Role: {}\n", role.name));
        output.push_str(&format!("Description: {}\n\n", role.description));
    }

    output.push_str("--- Available Roles ---\n");
    for role in manager.roles() {
        let marker = if manager.active_role().map(|r| &r.name) == Some(&role.name) {
            "*"
        } else {
            " "
        };
        output.push_str(&format!(
            "{} {} - {} ({})\n",
            marker, role.name, role.description, role.default_level
        ));
    }

    output
}

/// Check if query is about permissions
pub fn is_permissions_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("permission")
        || lower.contains("access control")
        || lower.contains("role")
}

/// Fun fact about permissions
pub fn settings_permissions_fun_fact() -> &'static str {
    "Anna uses role-based permissions to control who can modify settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_level_display() {
        assert_eq!(format!("{}", PermissionLevel::Admin), "Admin");
        assert_eq!(format!("{}", PermissionLevel::Read), "Read");
    }

    #[test]
    fn test_permission_action_display() {
        assert_eq!(format!("{}", PermissionAction::View), "View");
        assert_eq!(format!("{}", PermissionAction::Modify), "Modify");
    }

    #[test]
    fn test_permission_result_display() {
        assert_eq!(format!("{}", PermissionResult::Allowed), "Allowed");
        assert_eq!(format!("{}", PermissionResult::Denied), "Denied");
    }

    #[test]
    fn test_category_permission_new() {
        let perm = CategoryPermission::new(SettingsCategory::Personality, PermissionLevel::Write);
        assert!(perm.can(PermissionAction::View));
        assert!(perm.can(PermissionAction::Modify));
    }

    #[test]
    fn test_category_permission_read_only() {
        let perm = CategoryPermission::new(SettingsCategory::Risk, PermissionLevel::Read);
        assert!(perm.can(PermissionAction::View));
        assert!(!perm.can(PermissionAction::Modify));
    }

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

    #[test]
    fn test_format_permissions() {
        let mgr = PermissionManager::new();
        let output = format_permissions(&mgr);
        assert!(output.contains("Permissions"));
    }

    #[test]
    fn test_is_permissions_query() {
        assert!(is_permissions_query("show permissions"));
        assert!(is_permissions_query("access control settings"));
        assert!(!is_permissions_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_permissions_fun_fact();
        assert!(fact.contains("permission"));
    }
}
