use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Dangerous actions that always require confirmation regardless of policy level
pub const DANGEROUS_ACTIONS: &[&str] = &[
    "btrfs balance",
    "pacman -R",
    "mkfs",
    "fdisk",
    "dd if=",
    "rm -rf /",
    "mkswap",
    "swapon",
    "swapoff",
    "parted",
    "gdisk",
    "sgdisk",
];

/// Policy auto-apply levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PolicyLevel {
    /// Level 0: Manual only (default)
    #[default]
    Manual = 0,
    /// Level 1: Auto-apply safe maintenance (cache cleanup, orphan packages, trim)
    SafeMaintenance = 1,
    /// Level 2: Auto-apply safe + moderate (cpu governor, ntp)
    SafeModerate = 2,
    /// Level 3: Allow full autonomy except flagged dangerous actions
    FullAutonomy = 3,
}

impl From<u8> for PolicyLevel {
    fn from(val: u8) -> Self {
        match val {
            0 => PolicyLevel::Manual,
            1 => PolicyLevel::SafeMaintenance,
            2 => PolicyLevel::SafeModerate,
            3 => PolicyLevel::FullAutonomy,
            _ => PolicyLevel::Manual,
        }
    }
}

/// Prompt style for approval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PromptStyle {
    #[default]
    Interactive,
    Silent,
    RequireSudo,
}

/// User policy configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Policy {
    #[serde(default)]
    pub level: LevelConfig,
    #[serde(default)]
    pub approval: ApprovalConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LevelConfig {
    #[serde(default)]
    pub auto_apply: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApprovalConfig {
    #[serde(default)]
    pub prompt_style: PromptStyle,
    #[serde(default = "default_confirm_dangerous")]
    pub confirm_dangerous: bool,
}

fn default_confirm_dangerous() -> bool {
    true
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            prompt_style: PromptStyle::Interactive,
            confirm_dangerous: true,
        }
    }
}

impl Policy {
    /// Load policy for a specific UID
    pub fn load_for_uid(uid: u32) -> Result<Self> {
        let policy_path = format!("/etc/anna/policy.d/{}.toml", uid);
        Self::load_from_path(&policy_path)
    }

    /// Load policy from a specific path
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            // No policy file → default (manual only)
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("read policy file {}", path.display()))?;

        let policy: Policy = toml::from_str(&contents)
            .with_context(|| format!("parse policy file {}", path.display()))?;

        Ok(policy)
    }

    /// Get the policy level
    pub fn level(&self) -> PolicyLevel {
        PolicyLevel::from(self.level.auto_apply)
    }

    /// Check if an action is allowed by this policy
    pub fn allows_action(&self, advice_kind: &str, commands: &[String]) -> ActionDecision {
        // Check for dangerous commands first
        if self.approval.confirm_dangerous && is_dangerous(commands) {
            return ActionDecision::RequiresConfirmation {
                reason: "Contains dangerous operation".to_string(),
            };
        }

        // Check policy level
        let level = self.level();
        let allowed = match level {
            PolicyLevel::Manual => false,
            PolicyLevel::SafeMaintenance => is_safe_maintenance(advice_kind),
            PolicyLevel::SafeModerate => {
                is_safe_maintenance(advice_kind) || is_moderate_action(advice_kind)
            }
            PolicyLevel::FullAutonomy => !is_dangerous(commands),
        };

        if allowed {
            ActionDecision::Allowed
        } else {
            ActionDecision::RequiresElevation {
                reason: format!(
                    "Policy level {} does not allow {}",
                    level as u8, advice_kind
                ),
            }
        }
    }
}

/// Decision for an action based on policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionDecision {
    /// Action is allowed by policy
    Allowed,
    /// Action requires user confirmation (dangerous)
    RequiresConfirmation { reason: String },
    /// Action requires elevation (higher policy level or sudo)
    RequiresElevation { reason: String },
}

/// Check if any command contains dangerous operations
pub fn is_dangerous(commands: &[String]) -> bool {
    for cmd in commands {
        for dangerous in DANGEROUS_ACTIONS {
            if cmd.contains(dangerous) {
                return true;
            }
        }
    }
    false
}

/// Check if advice kind is safe maintenance
fn is_safe_maintenance(kind: &str) -> bool {
    matches!(
        kind,
        "system/cache-cleanup"
            | "system/orphan-packages"
            | "system/journal-trim"
            | "system/tmp-cleanup"
    )
}

/// Check if advice kind is moderate action
fn is_moderate_action(kind: &str) -> bool {
    matches!(
        kind,
        "system/cpu-governor" | "system/ntp-sync" | "system/swap-optimization"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = Policy::default();
        assert_eq!(policy.level(), PolicyLevel::Manual);
        assert_eq!(policy.approval.prompt_style, PromptStyle::Interactive);
        assert!(policy.approval.confirm_dangerous);
    }

    #[test]
    fn test_policy_level_from_u8() {
        assert_eq!(PolicyLevel::from(0), PolicyLevel::Manual);
        assert_eq!(PolicyLevel::from(1), PolicyLevel::SafeMaintenance);
        assert_eq!(PolicyLevel::from(2), PolicyLevel::SafeModerate);
        assert_eq!(PolicyLevel::from(3), PolicyLevel::FullAutonomy);
        assert_eq!(PolicyLevel::from(99), PolicyLevel::Manual); // Invalid → Manual
    }

    #[test]
    fn test_dangerous_action_detection() {
        assert!(is_dangerous(&["sudo pacman -R some-package".to_string()]));
        assert!(is_dangerous(&["btrfs balance start /".to_string()]));
        assert!(is_dangerous(&["mkfs.ext4 /dev/sda1".to_string()]));
        assert!(!is_dangerous(&["pacman -Syu".to_string()]));
        assert!(!is_dangerous(&["systemctl restart annad".to_string()]));
    }

    #[test]
    fn test_safe_maintenance_classification() {
        assert!(is_safe_maintenance("system/cache-cleanup"));
        assert!(is_safe_maintenance("system/orphan-packages"));
        assert!(!is_safe_maintenance("system/cpu-governor"));
        assert!(!is_safe_maintenance("system/unknown"));
    }

    #[test]
    fn test_moderate_action_classification() {
        assert!(is_moderate_action("system/cpu-governor"));
        assert!(is_moderate_action("system/ntp-sync"));
        assert!(!is_moderate_action("system/cache-cleanup"));
    }

    #[test]
    fn test_manual_policy_blocks_everything() {
        let policy = Policy {
            level: LevelConfig { auto_apply: 0 },
            approval: ApprovalConfig::default(),
        };

        let decision = policy.allows_action("system/cache-cleanup", &["paccache -r".to_string()]);
        assert!(matches!(decision, ActionDecision::RequiresElevation { .. }));
    }

    #[test]
    fn test_safe_maintenance_policy() {
        let policy = Policy {
            level: LevelConfig { auto_apply: 1 },
            approval: ApprovalConfig::default(),
        };

        // Should allow safe maintenance
        let decision = policy.allows_action("system/cache-cleanup", &["paccache -r".to_string()]);
        assert_eq!(decision, ActionDecision::Allowed);

        // Should block moderate actions
        let decision = policy.allows_action("system/cpu-governor", &["cpupower".to_string()]);
        assert!(matches!(decision, ActionDecision::RequiresElevation { .. }));
    }

    #[test]
    fn test_dangerous_always_blocked() {
        let policy = Policy {
            level: LevelConfig { auto_apply: 3 }, // Full autonomy
            approval: ApprovalConfig {
                confirm_dangerous: true,
                ..Default::default()
            },
        };

        let decision =
            policy.allows_action("system/disk-ops", &["mkfs.ext4 /dev/sda1".to_string()]);
        assert!(matches!(
            decision,
            ActionDecision::RequiresConfirmation { .. }
        ));
    }

    #[test]
    fn test_full_autonomy_allows_non_dangerous() {
        let policy = Policy {
            level: LevelConfig { auto_apply: 3 },
            approval: ApprovalConfig::default(),
        };

        let decision = policy.allows_action("system/any-action", &["echo hello".to_string()]);
        assert_eq!(decision, ActionDecision::Allowed);
    }

    #[test]
    fn test_policy_toml_parsing() {
        let toml_content = r#"
[level]
auto_apply = 2

[approval]
prompt_style = "silent"
confirm_dangerous = true
        "#;

        let policy: Policy = toml::from_str(toml_content).unwrap();
        assert_eq!(policy.level(), PolicyLevel::SafeModerate);
        assert_eq!(policy.approval.prompt_style, PromptStyle::Silent);
        assert!(policy.approval.confirm_dangerous);
    }
}
