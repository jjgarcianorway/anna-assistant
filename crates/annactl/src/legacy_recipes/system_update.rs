//! System Update Recipe
//!
//! Beta.152: Deterministic recipe for system updates via pacman
//!
//! This module generates safe ActionPlans for:
//! - Checking for available updates
//! - Performing full system upgrades
//! - Updating specific packages
//! - Displaying update statistics

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

/// System update scenario detector
pub struct SystemUpdateRecipe;

/// Update operation types
#[derive(Debug, Clone, PartialEq)]
enum UpdateOperation {
    CheckUpdates,     // "check for updates"
    FullUpgrade,      // "update system" / "upgrade all packages"
    UpgradePackage,   // "update firefox" / "upgrade specific package"
}

impl SystemUpdateRecipe {
    /// Check if user request matches system update operations
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Update-related keywords
        let has_update_keyword = input_lower.contains("update")
            || input_lower.contains("upgrade")
            || input_lower.contains("patch");

        // System context keywords
        let has_system_context = input_lower.contains("system")
            || input_lower.contains("pacman")
            || input_lower.contains("package")
            || input_lower.contains("all")
            || input_lower.contains("check")
            || input_lower.contains("available");

        has_update_keyword && has_system_context
    }

    /// Generate system update ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let operation = Self::detect_operation(
            telemetry
                .get("user_request")
                .map(|s| s.as_str())
                .unwrap_or(""),
        );

        match operation {
            UpdateOperation::CheckUpdates => Self::build_check_updates_plan(telemetry),
            UpdateOperation::FullUpgrade => Self::build_full_upgrade_plan(telemetry),
            UpdateOperation::UpgradePackage => {
                let package_name = telemetry
                    .get("package_name")
                    .map(|s| s.as_str())
                    .unwrap_or("<package>");
                Self::build_package_upgrade_plan(package_name, telemetry)
            }
        }
    }

    fn detect_operation(user_input: &str) -> UpdateOperation {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("check") || input_lower.contains("available") {
            UpdateOperation::CheckUpdates
        } else if input_lower.contains("system")
            || input_lower.contains("all")
            || input_lower.contains("everything")
        {
            UpdateOperation::FullUpgrade
        } else if input_lower.contains("package")
            || (input_lower.contains("update") || input_lower.contains("upgrade"))
                && !input_lower.contains("system")
                && !input_lower.contains("all")
        {
            // Specific package upgrade (package name should be in telemetry)
            UpdateOperation::UpgradePackage
        } else {
            // Default to full upgrade if ambiguous
            UpdateOperation::FullUpgrade
        }
    }

    fn build_check_updates_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let necessary_checks = vec![NecessaryCheck {
            id: "check-internet".to_string(),
            description: "Verify internet connectivity for package database sync".to_string(),
            command: "ping -c 1 archlinux.org".to_string(),
            risk_level: RiskLevel::Info,
            required: true,
        }];

        let command_plan = vec![
            CommandStep {
                id: "sync-databases".to_string(),
                description: "Synchronize package databases with mirrors".to_string(),
                command: "sudo pacman -Sy".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-updates".to_string(),
                description: "List available package updates".to_string(),
                command: "pacman -Qu".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "count-updates".to_string(),
                description: "Count number of available updates".to_string(),
                command: "pacman -Qu | wc -l".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "estimate-download-size".to_string(),
                description: "Estimate download size for updates".to_string(),
                command: "pacman -Sup | grep -v '://' | xargs -r du -ch 2>/dev/null | tail -1 || echo 'Size estimation unavailable'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut analysis_parts = vec![
            "User requests check for available system updates. This is a read-only operation \
             that syncs package databases and lists available updates."
                .to_string(),
        ];

        if !has_internet {
            analysis_parts.push(
                "⚠️ WARNING: Internet connectivity not confirmed. Update check requires network access."
                    .to_string(),
            );
        }

        let analysis = analysis_parts.join(" ");

        let goals = vec![
            "Synchronize package databases with Arch mirrors".to_string(),
            "List all packages with available updates".to_string(),
            "Show update count and estimated download size".to_string(),
        ];

        let notes_for_user = "This check shows available updates without installing them.\n\n\
             Output shows:\n\
             • Package name\n\
             • Current version → New version\n\
             • Total number of updates\n\
             • Estimated download size\n\n\
             To apply updates:\n\
             sudo pacman -Syu\n\n\
             To update only specific packages:\n\
             sudo pacman -S <package1> <package2>\n\n\
             To see detailed changelog:\n\
             pacman -Qc <package-name>\n\n\
             Best practices:\n\
             • Always check Arch Linux news before major updates: https://archlinux.org/news/\n\
             • Review the package list before updating\n\
             • Have a backup or timeshift snapshot for critical systems\n\
             • Avoid partial upgrades (always use -Syu, not -Sy)\n\n\
             Risk: LOW - Only syncs databases, does not install packages".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "system_check_updates",
        )
    }

    fn build_full_upgrade_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let disk_space_gb = telemetry
            .get("disk_free_gb")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-internet".to_string(),
                description: "Verify internet connectivity".to_string(),
                command: "ping -c 1 archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-disk-space".to_string(),
                description: "Verify sufficient disk space".to_string(),
                command: "df -h /".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-pacman-lock".to_string(),
                description: "Verify no other pacman instance is running".to_string(),
                command: "ls -la /var/lib/pacman/db.lck 2>/dev/null || echo 'No lock file'".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-arch-news".to_string(),
                description: "Reminder to check Arch Linux news for manual interventions".to_string(),
                command: "echo 'Before proceeding, review: https://archlinux.org/news/'".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-updates".to_string(),
                description: "Show packages that will be updated".to_string(),
                command: "sudo pacman -Syu --print-format='%n %v -> %v' 2>/dev/null || pacman -Qu".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "full-system-upgrade".to_string(),
                description: "Perform full system upgrade (sync databases and upgrade all packages)".to_string(),
                command: "sudo pacman -Syu --noconfirm".to_string(),
                risk_level: RiskLevel::High,
                rollback_id: Some("downgrade-packages".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "clean-package-cache".to_string(),
                description: "Clean old packages from cache (keep last 3 versions)".to_string(),
                command: "sudo paccache -rk3".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-failed-services".to_string(),
                description: "Check for any failed services after update".to_string(),
                command: "systemctl --failed".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-pacdiff".to_string(),
                description: "Check for .pacnew/.pacsave files (configuration merges needed)".to_string(),
                command: "sudo find /etc -name '*.pacnew' -o -name '*.pacsave' 2>/dev/null | head -20 || echo 'No .pacnew/.pacsave files found'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "downgrade-packages".to_string(),
                description: "Downgrade packages using pacman cache (if issues occur)".to_string(),
                command: "# To downgrade a specific package:\n\
                          # 1. Check cache: ls /var/cache/pacman/pkg/<package>*\n\
                          # 2. Downgrade: sudo pacman -U /var/cache/pacman/pkg/<package>-<old-version>.pkg.tar.zst\n\
                          #\n\
                          # To downgrade all packages (use with extreme caution):\n\
                          # Install 'downgrade' from AUR: yay -S downgrade\n\
                          # Then: sudo downgrade <package-name>".to_string(),
            },
        ];

        let mut analysis_parts = vec![
            "User requests full system upgrade. This will sync package databases and upgrade \
             all installed packages to their latest versions."
                .to_string(),
            format!("System has {:.1} GB free disk space.", disk_space_gb),
        ];

        if !has_internet {
            analysis_parts.push(
                "⚠️ WARNING: Internet connectivity not confirmed. System upgrade requires network access."
                    .to_string(),
            );
        }

        if disk_space_gb < 5.0 {
            analysis_parts.push(
                "⚠️ WARNING: Low disk space. System upgrades typically require 1-3 GB free space."
                    .to_string(),
            );
        }

        let analysis = analysis_parts.join(" ");

        let goals = vec![
            "Synchronize package databases with Arch mirrors".to_string(),
            "Upgrade all installed packages to latest versions".to_string(),
            "Clean old packages from cache to free space".to_string(),
            "Check for service failures after update".to_string(),
            "Identify configuration file conflicts (.pacnew/.pacsave)".to_string(),
        ];

        let notes_for_user = "⚠️ CRITICAL: Read Arch News before proceeding!\n\
             https://archlinux.org/news/\n\n\
             Some updates require manual intervention (bootloader updates, configuration changes, etc.)\n\n\
             This will perform a full system upgrade:\n\
             • Sync package databases (pacman -Sy)\n\
             • Download and install all available updates\n\
             • Clean old cached packages\n\n\
             After upgrade:\n\
             • Review any .pacnew/.pacsave files and merge config changes\n\
             • Check for failed services\n\
             • Reboot if kernel was updated\n\n\
             To merge .pacnew files:\n\
             sudo pacdiff\n\
             # Or manually:\n\
             # sudo vimdiff /etc/config.pacnew /etc/config\n\n\
             If issues occur after upgrade:\n\
             1. Check logs: journalctl -p err -b\n\
             2. Check Arch forums and bug tracker\n\
             3. Downgrade problematic package:\n\
                sudo pacman -U /var/cache/pacman/pkg/<package>-<old-version>.pkg.tar.zst\n\n\
             Kernel updates require reboot:\n\
             If kernel was updated, reboot with: sudo reboot\n\n\
             Risk: HIGH - System-wide package changes\n\
             Estimated time: 5-30 minutes depending on number of updates\n\
             Estimated download: Varies (typically 100MB - 2GB)".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "system_full_upgrade",
        )
    }

    fn build_package_upgrade_plan(package_name: &str, telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-package-installed".to_string(),
                description: format!("Check if {} is installed", package_name),
                command: format!("pacman -Q {}", package_name),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-update-available".to_string(),
                description: format!("Check if update is available for {}", package_name),
                command: format!("pacman -Qu | grep -w '{}'", package_name),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-current-version".to_string(),
                description: format!("Show current version of {}", package_name),
                command: format!("pacman -Q {}", package_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "upgrade-package".to_string(),
                description: format!("Upgrade {} to latest version", package_name),
                command: format!("sudo pacman -S --noconfirm {}", package_name),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("downgrade-package".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-new-version".to_string(),
                description: format!("Verify {} was upgraded successfully", package_name),
                command: format!("pacman -Q {}", package_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "downgrade-package".to_string(),
            description: format!("Downgrade {} to previous version", package_name),
            command: format!(
                "# Find old version in cache:\n\
                 # ls /var/cache/pacman/pkg/{}*\n\
                 # Downgrade:\n\
                 # sudo pacman -U /var/cache/pacman/pkg/{}-<old-version>.pkg.tar.zst",
                package_name, package_name
            ),
        }];

        let mut analysis_parts = vec![format!(
            "User requests upgrade of specific package: {}. This will update only this package \
             and its dependencies (if needed).",
            package_name
        )];

        if !has_internet {
            analysis_parts.push(
                "⚠️ WARNING: Internet connectivity not confirmed. Package upgrade requires network access."
                    .to_string(),
            );
        }

        let analysis = analysis_parts.join(" ");

        let goals = vec![
            format!("Show current version of {}", package_name),
            format!("Upgrade {} to latest available version", package_name),
            format!("Verify {} upgrade was successful", package_name),
        ];

        let notes_for_user = format!(
            "This will upgrade only {} and its dependencies.\n\n\
             Note: Partial upgrades can sometimes cause issues in Arch Linux.\n\
             It's generally recommended to use 'sudo pacman -Syu' for full system upgrades.\n\n\
             If this package has many dependencies, they will also be upgraded.\n\
             To see what would be upgraded without installing:\n\
             sudo pacman -S --print-format='%n %v' {}\n\n\
             After upgrade:\n\
             • If it's a service, restart it: sudo systemctl restart <service>\n\
             • If it's a daemon, reload it\n\
             • If it's a library, dependent programs may need restart\n\n\
             To downgrade if issues occur:\n\
             sudo pacman -U /var/cache/pacman/pkg/{}-<old-version>.pkg.tar.zst\n\n\
             Risk: MEDIUM - Updates specific package and dependencies",
            package_name, package_name, package_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "system_package_upgrade",
        )
    }

    fn build_action_plan(
        analysis: String,
        goals: Vec<String>,
        necessary_checks: Vec<NecessaryCheck>,
        command_plan: Vec<CommandStep>,
        rollback_plan: Vec<RollbackStep>,
        notes_for_user: String,
        template_name: &str,
    ) -> Result<ActionPlan> {
        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("system_update.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some(template_name.to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_update_requests() {
        // Check updates
        assert!(SystemUpdateRecipe::matches_request("check for system updates"));
        assert!(SystemUpdateRecipe::matches_request("are there available updates"));

        // Full upgrades
        assert!(SystemUpdateRecipe::matches_request("update system"));
        assert!(SystemUpdateRecipe::matches_request("upgrade all packages"));
        assert!(SystemUpdateRecipe::matches_request("full system update"));

        // Package-specific (detected as update context)
        assert!(SystemUpdateRecipe::matches_request("update firefox package"));
        assert!(SystemUpdateRecipe::matches_request("upgrade pacman"));

        // Should not match
        assert!(!SystemUpdateRecipe::matches_request("what is pacman"));
        assert!(!SystemUpdateRecipe::matches_request("install a package"));
        assert!(!SystemUpdateRecipe::matches_request("how do I update"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            SystemUpdateRecipe::detect_operation("check for updates"),
            UpdateOperation::CheckUpdates
        );
        assert_eq!(
            SystemUpdateRecipe::detect_operation("show available updates"),
            UpdateOperation::CheckUpdates
        );
        assert_eq!(
            SystemUpdateRecipe::detect_operation("update system"),
            UpdateOperation::FullUpgrade
        );
        assert_eq!(
            SystemUpdateRecipe::detect_operation("upgrade all packages"),
            UpdateOperation::FullUpgrade
        );
    }

    #[test]
    fn test_check_updates_plan_is_low_risk() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check for updates".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = SystemUpdateRecipe::build_plan(&telemetry).unwrap();

        // Check updates should be low/info risk, not require confirmation
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Low); // sync databases
        assert!(!plan.command_plan[0].requires_confirmation);
        assert_eq!(plan.command_plan[1].risk_level, RiskLevel::Info); // list updates

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "system_check_updates");
    }

    #[test]
    fn test_full_upgrade_plan_is_high_risk() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "update system".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("disk_free_gb".to_string(), "50.0".to_string());

        let plan = SystemUpdateRecipe::build_plan(&telemetry).unwrap();

        // Full upgrade should be HIGH risk and require confirmation
        let upgrade_step = plan.command_plan.iter().find(|c| c.id == "full-system-upgrade").unwrap();
        assert_eq!(upgrade_step.risk_level, RiskLevel::High);
        assert!(upgrade_step.requires_confirmation);
        assert!(upgrade_step.rollback_id.is_some());

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "system_full_upgrade");
    }

    #[test]
    fn test_low_disk_space_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "update system".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("disk_free_gb".to_string(), "2.0".to_string());

        let plan = SystemUpdateRecipe::build_plan(&telemetry).unwrap();

        // Should contain warning about low disk space
        assert!(plan.analysis.contains("WARNING"));
        assert!(plan.analysis.contains("Low disk space"));
    }

    #[test]
    fn test_package_upgrade_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "upgrade firefox".to_string());
        telemetry.insert("package_name".to_string(), "firefox".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = SystemUpdateRecipe::build_plan(&telemetry).unwrap();

        // Should mention firefox specifically
        assert!(plan.analysis.contains("firefox"));
        assert!(plan.command_plan[1].command.contains("firefox"));
        assert_eq!(plan.command_plan[1].risk_level, RiskLevel::Medium);
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "system_package_upgrade");
    }
}
