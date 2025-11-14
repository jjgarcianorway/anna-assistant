//! Safe Change Recipe System v0.1
//!
//! Phase 7: LLM-Assisted Changes with Strict Guardrails
//!
//! The LLM never touches the system directly. It only proposes plans.
//! Anna validates, classifies risk, shows consequences, and executes through
//! well-tested primitives with backups and rollback.
//!
//! Design principles:
//! - Whitelisted action types only (no arbitrary shell commands)
//! - Explicit risk classification for every action
//! - Human-readable explanations of consequences
//! - Rollback strategy must be clear before execution
//! - System-wide destructive changes are forbidden

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Risk level for a change action
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChangeRisk {
    /// Low risk: Cosmetic changes, user-space only
    /// Examples: wallpaper, terminal colors, harmless config tweaks
    /// Worst case: User might not like the aesthetic
    Low,

    /// Medium risk: Configuration changes that might cause minor issues
    /// Examples: editing configs in $HOME, enabling user systemd services
    /// Worst case: Application might fail to start until config is fixed
    Medium,

    /// High risk: Changes that can break important functionality
    /// Examples: system-wide configs, modifying /etc, installing critical packages
    /// Worst case: System services might fail, manual intervention needed
    High,

    /// Forbidden: Changes that are never allowed in v0.1
    /// Examples: bootloader, partitions, fstab, initrd, cryptsetup
    /// These require specialized knowledge and are too dangerous to automate
    Forbidden,
}

impl ChangeRisk {
    /// Get a human-readable description of this risk level
    pub fn description(&self) -> &'static str {
        match self {
            ChangeRisk::Low => "Low risk - cosmetic or user-space only",
            ChangeRisk::Medium => "Medium risk - may cause application startup issues",
            ChangeRisk::High => "High risk - system services may be affected",
            ChangeRisk::Forbidden => "FORBIDDEN - too dangerous to automate",
        }
    }

    /// Get realistic worst-case scenario for this risk level
    pub fn worst_case(&self) -> &'static str {
        match self {
            ChangeRisk::Low => "You might not like how it looks",
            ChangeRisk::Medium => "Application may fail to start until config is fixed",
            ChangeRisk::High => "System services may fail, manual recovery needed",
            ChangeRisk::Forbidden => "Data loss, unbootable system, corrupted filesystem",
        }
    }
}

/// Category of change being made
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeCategory {
    /// Cosmetic user-space changes
    /// Examples: wallpapers, themes, terminal colors
    CosmeticUser,

    /// User configuration files ($HOME dotfiles)
    /// Examples: vim, zsh, tmux, git configs
    UserConfig,

    /// System service management
    /// Examples: systemd unit enable/disable, log rotation
    SystemService,

    /// Package management
    /// Examples: install/remove packages via pacman/yay
    SystemPackage,

    /// Boot and storage (ALWAYS forbidden in v0.1)
    /// Examples: bootloader, partitions, fstab, initrd
    BootAndStorage,
}

impl ChangeCategory {
    /// Get default risk level for this category
    pub fn default_risk(&self) -> ChangeRisk {
        match self {
            ChangeCategory::CosmeticUser => ChangeRisk::Low,
            ChangeCategory::UserConfig => ChangeRisk::Medium,
            ChangeCategory::SystemService => ChangeRisk::High,
            ChangeCategory::SystemPackage => ChangeRisk::Medium,
            ChangeCategory::BootAndStorage => ChangeRisk::Forbidden,
        }
    }

    /// Get category-specific risk notes
    pub fn risk_notes(&self, risk: ChangeRisk) -> String {
        match (self, risk) {
            (ChangeCategory::CosmeticUser, ChangeRisk::Low) => {
                "This only affects appearance. You can easily change it back.".to_string()
            }
            (ChangeCategory::UserConfig, ChangeRisk::Medium) => {
                "If this goes wrong, the program may not start. \
                 I'll back up your config first so you can restore it."
                    .to_string()
            }
            (ChangeCategory::SystemService, ChangeRisk::High) => {
                "This affects system-wide services. If something breaks, \
                 you may need to manually fix it with sudo."
                    .to_string()
            }
            (ChangeCategory::SystemPackage, ChangeRisk::Medium) => {
                "Package changes can pull in dependencies or fail during install. \
                 You can uninstall packages later if needed."
                    .to_string()
            }
            (ChangeCategory::BootAndStorage, _) => {
                "FORBIDDEN: Bootloader, partition, and filesystem changes \
                 are too dangerous to automate. Do these manually."
                    .to_string()
            }
            _ => format!(
                "This change has {} risk. Proceed with caution.",
                risk.description()
            ),
        }
    }
}

/// Whitelisted action types - NO arbitrary shell commands allowed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChangeActionKind {
    /// Edit a file with structured approach (not free-form shell)
    EditFile {
        path: PathBuf,
        strategy: EditStrategy,
    },

    /// Append content to a file
    AppendToFile { path: PathBuf, content: String },

    /// Install packages via pacman (requires sudo)
    InstallPackages { packages: Vec<String> },

    /// Remove packages via pacman (requires sudo)
    RemovePackages { packages: Vec<String> },

    /// Enable a systemd service (user or system)
    EnableService {
        service_name: String,
        user_service: bool,
    },

    /// Disable a systemd service (user or system)
    DisableService {
        service_name: String,
        user_service: bool,
    },

    /// Set wallpaper (abstract - implementation branches on DE/WM)
    SetWallpaper { image_path: PathBuf },

    /// Run a read-only command for verification (ls, grep, pacman -Qi)
    /// IMPORTANT: No mutating commands allowed here
    RunReadOnlyCommand {
        command: String,
        args: Vec<String>,
    },
}

impl ChangeActionKind {
    /// Check if this action kind is allowed (not Forbidden)
    pub fn is_allowed(&self) -> bool {
        // In v0.1, all defined actions are allowed
        // Forbidden actions are caught by path validation
        true
    }

    /// Get the category for this action kind
    pub fn category(&self) -> ChangeCategory {
        match self {
            ChangeActionKind::SetWallpaper { .. } => ChangeCategory::CosmeticUser,
            ChangeActionKind::EditFile { path, .. } | ChangeActionKind::AppendToFile { path, .. } => {
                // Check path to determine category
                if path.starts_with("/etc") {
                    ChangeCategory::SystemService
                } else if path.to_string_lossy().contains("/boot")
                    || path.to_string_lossy().contains("fstab")
                    || path.to_string_lossy().contains("grub")
                {
                    ChangeCategory::BootAndStorage
                } else {
                    ChangeCategory::UserConfig
                }
            }
            ChangeActionKind::InstallPackages { .. } | ChangeActionKind::RemovePackages { .. } => {
                ChangeCategory::SystemPackage
            }
            ChangeActionKind::EnableService { .. } | ChangeActionKind::DisableService { .. } => {
                ChangeCategory::SystemService
            }
            ChangeActionKind::RunReadOnlyCommand { .. } => ChangeCategory::CosmeticUser,
        }
    }

    /// Calculate risk level for this specific action
    pub fn calculate_risk(&self) -> ChangeRisk {
        let category = self.category();

        // Override risk based on specific patterns
        match self {
            ChangeActionKind::EditFile { path, .. } | ChangeActionKind::AppendToFile { path, .. } => {
                // Boot and storage paths are ALWAYS forbidden
                if path.to_string_lossy().contains("/boot")
                    || path.to_string_lossy().contains("fstab")
                    || path.to_string_lossy().contains("grub")
                    || path.to_string_lossy().contains("initramfs")
                    || path.to_string_lossy().contains("crypttab")
                {
                    return ChangeRisk::Forbidden;
                }

                // System configs are high risk
                if path.starts_with("/etc") {
                    return ChangeRisk::High;
                }

                // User configs are medium risk
                category.default_risk()
            }
            ChangeActionKind::InstallPackages { packages } => {
                // Installing critical system packages is high risk
                let critical_packages = ["systemd", "kernel", "grub", "pacman"];
                if packages
                    .iter()
                    .any(|p| critical_packages.iter().any(|c| p.contains(c)))
                {
                    ChangeRisk::High
                } else {
                    ChangeRisk::Medium
                }
            }
            _ => category.default_risk(),
        }
    }

    /// Check if this action needs sudo
    pub fn needs_sudo(&self) -> bool {
        match self {
            ChangeActionKind::EditFile { path, .. } | ChangeActionKind::AppendToFile { path, .. } => {
                path.starts_with("/etc") || path.starts_with("/usr")
            }
            ChangeActionKind::InstallPackages { .. }
            | ChangeActionKind::RemovePackages { .. } => true,
            ChangeActionKind::EnableService { user_service, .. }
            | ChangeActionKind::DisableService { user_service, .. } => !user_service,
            _ => false,
        }
    }
}

/// Strategy for editing a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditStrategy {
    /// Append lines if they don't already exist
    AppendIfMissing { lines: Vec<String> },

    /// Replace a specific section marked by comments
    ReplaceSection {
        start_marker: String,
        end_marker: String,
        new_content: String,
    },

    /// Replace the entire file (DANGEROUS - requires explicit confirmation)
    ReplaceEntire { new_content: String },
}

/// A single action in a change recipe
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeAction {
    /// Unique ID for this action
    pub id: String,

    /// The kind of action and its specific details
    pub kind: ChangeActionKind,

    /// Category of change
    pub category: ChangeCategory,

    /// Risk level
    pub risk: ChangeRisk,

    /// Human-readable description of what this action does
    pub description: String,

    /// Estimated impact (shown to user)
    pub estimated_impact: String,
}

impl ChangeAction {
    /// Create a new change action with auto-calculated category and risk
    pub fn new(kind: ChangeActionKind, description: String, estimated_impact: String) -> Self {
        let category = kind.category();
        let risk = kind.calculate_risk();

        Self {
            id: Uuid::new_v4().to_string(),
            kind,
            category,
            risk,
            description,
            estimated_impact,
        }
    }

    /// Validate this action is allowed
    pub fn validate(&self) -> Result<()> {
        // Check if action is forbidden
        if self.risk == ChangeRisk::Forbidden {
            // Include path in error message for file operations
            let detail = match &self.kind {
                ChangeActionKind::EditFile { path, .. } | ChangeActionKind::AppendToFile { path, .. } => {
                    format!(" (Path: {})", path.display())
                }
                _ => String::new(),
            };

            anyhow::bail!(
                "Action '{}' is FORBIDDEN{}: {}",
                self.description,
                detail,
                ChangeRisk::Forbidden.worst_case()
            );
        }

        // Check if action kind is allowed
        if !self.kind.is_allowed() {
            anyhow::bail!("Action kind is not allowed");
        }

        Ok(())
    }
}

/// Source of a change recipe
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeRecipeSource {
    /// Planned by LLM based on user request
    LlmPlanned {
        model_profile_id: String,
        user_query: String,
    },

    /// Predefined recipe from Anna's built-in library
    Predefined { recipe_id: String },

    /// Manual recipe created by user
    Manual,
}

/// A complete change recipe with multiple actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeRecipe {
    /// Unique ID for this recipe
    pub id: String,

    /// Title of the change
    pub title: String,

    /// Summary of what this recipe does
    pub summary: String,

    /// Why this change matters (can be user-facing explanation)
    pub why_it_matters: String,

    /// Actions to execute in order
    pub actions: Vec<ChangeAction>,

    /// Overall risk level (computed from actions)
    pub overall_risk: ChangeRisk,

    /// Notes about rollback strategy
    pub rollback_notes: String,

    /// Source of this recipe
    pub source: ChangeRecipeSource,
}

impl ChangeRecipe {
    /// Create a new change recipe
    pub fn new(
        title: String,
        summary: String,
        why_it_matters: String,
        actions: Vec<ChangeAction>,
        rollback_notes: String,
        source: ChangeRecipeSource,
    ) -> Self {
        // Compute overall risk as max of individual action risks
        let overall_risk = actions
            .iter()
            .map(|a| a.risk)
            .max()
            .unwrap_or(ChangeRisk::Low);

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            summary,
            why_it_matters,
            actions,
            overall_risk,
            rollback_notes,
            source,
        }
    }

    /// Validate entire recipe
    pub fn validate(&self) -> Result<()> {
        // Validate each action (this will catch forbidden actions with detailed error messages)
        for action in &self.actions {
            action.validate().with_context(|| {
                format!("In recipe '{}'", self.title)
            })?;
        }

        // Check that we have at least one action
        if self.actions.is_empty() {
            anyhow::bail!("Recipe '{}' has no actions", self.title);
        }

        Ok(())
    }

    /// Get a list of actions that need sudo
    pub fn sudo_actions(&self) -> Vec<&ChangeAction> {
        self.actions
            .iter()
            .filter(|a| a.kind.needs_sudo())
            .collect()
    }

    /// Check if recipe needs any sudo access
    pub fn needs_sudo(&self) -> bool {
        self.actions.iter().any(|a| a.kind.needs_sudo())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_ordering() {
        assert!(ChangeRisk::Low < ChangeRisk::Medium);
        assert!(ChangeRisk::Medium < ChangeRisk::High);
        assert!(ChangeRisk::High < ChangeRisk::Forbidden);
    }

    #[test]
    fn test_category_default_risk() {
        assert_eq!(ChangeCategory::CosmeticUser.default_risk(), ChangeRisk::Low);
        assert_eq!(
            ChangeCategory::UserConfig.default_risk(),
            ChangeRisk::Medium
        );
        assert_eq!(
            ChangeCategory::SystemService.default_risk(),
            ChangeRisk::High
        );
        assert_eq!(
            ChangeCategory::BootAndStorage.default_risk(),
            ChangeRisk::Forbidden
        );
    }

    #[test]
    fn test_wallpaper_change_is_low_risk() {
        let action_kind = ChangeActionKind::SetWallpaper {
            image_path: PathBuf::from("/home/user/Pictures/wallpaper.jpg"),
        };

        assert_eq!(action_kind.category(), ChangeCategory::CosmeticUser);
        assert_eq!(action_kind.calculate_risk(), ChangeRisk::Low);
        assert!(!action_kind.needs_sudo());
    }

    #[test]
    fn test_boot_file_edit_is_forbidden() {
        let action_kind = ChangeActionKind::EditFile {
            path: PathBuf::from("/boot/grub/grub.cfg"),
            strategy: EditStrategy::AppendIfMissing {
                lines: vec!["test".to_string()],
            },
        };

        assert_eq!(action_kind.calculate_risk(), ChangeRisk::Forbidden);
        assert_eq!(action_kind.category(), ChangeCategory::BootAndStorage);
    }

    #[test]
    fn test_fstab_edit_is_forbidden() {
        let action_kind = ChangeActionKind::EditFile {
            path: PathBuf::from("/etc/fstab"),
            strategy: EditStrategy::AppendIfMissing {
                lines: vec!["test".to_string()],
            },
        };

        assert_eq!(action_kind.calculate_risk(), ChangeRisk::Forbidden);
    }

    #[test]
    fn test_user_config_edit_is_medium_risk() {
        let action_kind = ChangeActionKind::EditFile {
            path: PathBuf::from("/home/user/.vimrc"),
            strategy: EditStrategy::AppendIfMissing {
                lines: vec!["syntax on".to_string()],
            },
        };

        assert_eq!(action_kind.category(), ChangeCategory::UserConfig);
        assert_eq!(action_kind.calculate_risk(), ChangeRisk::Medium);
        assert!(!action_kind.needs_sudo());
    }

    #[test]
    fn test_etc_config_edit_is_high_risk() {
        let action_kind = ChangeActionKind::EditFile {
            path: PathBuf::from("/etc/systemd/system/test.service"),
            strategy: EditStrategy::ReplaceEntire {
                new_content: "test".to_string(),
            },
        };

        assert_eq!(action_kind.calculate_risk(), ChangeRisk::High);
        assert!(action_kind.needs_sudo());
    }

    #[test]
    fn test_package_install_needs_sudo() {
        let action_kind = ChangeActionKind::InstallPackages {
            packages: vec!["vim".to_string()],
        };

        assert!(action_kind.needs_sudo());
        assert_eq!(action_kind.calculate_risk(), ChangeRisk::Medium);
    }

    #[test]
    fn test_critical_package_install_is_high_risk() {
        let action_kind = ChangeActionKind::InstallPackages {
            packages: vec!["systemd".to_string()],
        };

        assert_eq!(action_kind.calculate_risk(), ChangeRisk::High);
    }

    #[test]
    fn test_recipe_overall_risk_is_max() {
        let low_action = ChangeAction::new(
            ChangeActionKind::SetWallpaper {
                image_path: PathBuf::from("/home/user/pic.jpg"),
            },
            "Set wallpaper".to_string(),
            "Changes desktop background".to_string(),
        );

        let high_action = ChangeAction::new(
            ChangeActionKind::EditFile {
                path: PathBuf::from("/etc/test.conf"),
                strategy: EditStrategy::AppendIfMissing {
                    lines: vec!["test".to_string()],
                },
            },
            "Edit system config".to_string(),
            "Modifies system configuration".to_string(),
        );

        let recipe = ChangeRecipe::new(
            "Test Recipe".to_string(),
            "A test recipe".to_string(),
            "For testing".to_string(),
            vec![low_action, high_action],
            "Restore from backup".to_string(),
            ChangeRecipeSource::Manual,
        );

        assert_eq!(recipe.overall_risk, ChangeRisk::High);
    }

    #[test]
    fn test_forbidden_action_validation_fails() {
        let forbidden_action = ChangeAction::new(
            ChangeActionKind::EditFile {
                path: PathBuf::from("/boot/grub/grub.cfg"),
                strategy: EditStrategy::ReplaceEntire {
                    new_content: "test".to_string(),
                },
            },
            "Edit bootloader".to_string(),
            "Modifies GRUB config".to_string(),
        );

        assert!(forbidden_action.validate().is_err());
    }

    #[test]
    fn test_recipe_with_forbidden_action_validation_fails() {
        let forbidden_action = ChangeAction::new(
            ChangeActionKind::EditFile {
                path: PathBuf::from("/etc/fstab"),
                strategy: EditStrategy::AppendIfMissing {
                    lines: vec!["test".to_string()],
                },
            },
            "Edit fstab".to_string(),
            "Adds mount point".to_string(),
        );

        let recipe = ChangeRecipe::new(
            "Dangerous Recipe".to_string(),
            "This should not be allowed".to_string(),
            "Testing forbidden actions".to_string(),
            vec![forbidden_action],
            "Cannot rollback filesystem changes".to_string(),
            ChangeRecipeSource::Manual,
        );

        assert_eq!(recipe.overall_risk, ChangeRisk::Forbidden);
        assert!(recipe.validate().is_err());
    }

    #[test]
    fn test_empty_recipe_validation_fails() {
        let recipe = ChangeRecipe::new(
            "Empty Recipe".to_string(),
            "Has no actions".to_string(),
            "Testing empty recipe".to_string(),
            vec![],
            "Nothing to rollback".to_string(),
            ChangeRecipeSource::Manual,
        );

        assert!(recipe.validate().is_err());
    }

    #[test]
    fn test_valid_recipe_passes_validation() {
        let action = ChangeAction::new(
            ChangeActionKind::SetWallpaper {
                image_path: PathBuf::from("/home/user/wallpaper.jpg"),
            },
            "Set wallpaper".to_string(),
            "Changes desktop background".to_string(),
        );

        let recipe = ChangeRecipe::new(
            "Wallpaper Change".to_string(),
            "Sets a new wallpaper".to_string(),
            "Personalizes your desktop".to_string(),
            vec![action],
            "Just change it back".to_string(),
            ChangeRecipeSource::Manual,
        );

        assert!(recipe.validate().is_ok());
        assert_eq!(recipe.overall_risk, ChangeRisk::Low);
        assert!(!recipe.needs_sudo());
    }
}
