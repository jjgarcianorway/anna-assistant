// v0.0.582: Settings Permissions (Phase 158)
// Access control and permissions for settings - Type Definitions

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
}
