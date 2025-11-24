//! Package Repair Recipe
//!
//! Beta.151: Deterministic recipe for fixing broken pacman packages
//!
//! This module generates a safe ActionPlan for:
//! - Detecting broken package states
//! - Clearing package cache conflicts
//! - Rebuilding package database
//! - Fixing common pacman issues

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

/// Package repair scenario detector
pub struct PackagesRecipe;

impl PackagesRecipe {
    /// Check if user request matches package repair
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        (input_lower.contains("package") || input_lower.contains("pacman"))
            && (input_lower.contains("fix")
                || input_lower.contains("repair")
                || input_lower.contains("broken")
                || input_lower.contains("error")
                || input_lower.contains("conflict")
                || input_lower.contains("problem"))
    }

    /// Generate package repair ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
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
                id: "check-pacman-lock".to_string(),
                description: "Check if pacman lock file exists (another pacman instance running)"
                    .to_string(),
                command: "ls -la /var/lib/pacman/db.lck 2>/dev/null || echo 'No lock file'"
                    .to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-disk-space".to_string(),
                description: "Verify sufficient disk space for operations".to_string(),
                command: "df -h /".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-pacman-log".to_string(),
                description: "Check recent pacman log for errors".to_string(),
                command:
                    "tail -n 50 /var/log/pacman.log | grep -i error || echo 'No recent errors'"
                        .to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-failed-services".to_string(),
                description: "Check for failed systemd services that might affect packages"
                    .to_string(),
                command: "systemctl --failed".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let mut command_plan = vec![];

        // Step 1: Remove lock file if present (safe - only if no pacman running)
        command_plan.push(CommandStep {
            id: "remove-lock-if-stale".to_string(),
            description: "Remove pacman lock file if stale (no pacman process running)".to_string(),
            command: "if [ -f /var/lib/pacman/db.lck ] && ! pgrep -x pacman > /dev/null; then \
                      sudo rm /var/lib/pacman/db.lck && echo 'Removed stale lock'; \
                      else echo 'Lock not stale or not present'; fi"
                .to_string(),
            risk_level: RiskLevel::Medium,
            rollback_id: None,
            requires_confirmation: true,
        });

        // Step 2: Sync package databases
        command_plan.push(CommandStep {
            id: "sync-databases".to_string(),
            description: "Synchronize package databases with mirrors".to_string(),
            command: "sudo pacman -Syy".to_string(),
            risk_level: RiskLevel::Low,
            rollback_id: None,
            requires_confirmation: false,
        });

        // Step 3: Clean package cache
        command_plan.push(CommandStep {
            id: "clean-cache".to_string(),
            description: "Clean old packages from cache (keep last 3 versions)".to_string(),
            command: "sudo paccache -rk3".to_string(),
            risk_level: RiskLevel::Low,
            rollback_id: None,
            requires_confirmation: false,
        });

        // Step 4: Check for partial upgrades and conflicts
        command_plan.push(CommandStep {
            id: "check-upgrades".to_string(),
            description: "Check for available updates and conflicts".to_string(),
            command: "sudo pacman -Qu || echo 'System up to date'".to_string(),
            risk_level: RiskLevel::Info,
            rollback_id: None,
            requires_confirmation: false,
        });

        // Step 5: Verify package integrity
        command_plan.push(CommandStep {
            id: "verify-packages".to_string(),
            description: "Verify integrity of installed packages".to_string(),
            command: "sudo pacman -Qkk 2>&1 | grep -E 'warning|error' | head -20 || echo 'No integrity issues found'".to_string(),
            risk_level: RiskLevel::Info,
            rollback_id: None,
            requires_confirmation: false,
        });

        // Step 6: List orphaned packages
        command_plan.push(CommandStep {
            id: "list-orphans".to_string(),
            description: "List orphaned packages (no longer needed)".to_string(),
            command: "pacman -Qtdq || echo 'No orphaned packages'".to_string(),
            risk_level: RiskLevel::Info,
            rollback_id: None,
            requires_confirmation: false,
        });

        // Rollback plan (minimal - most operations are read-only or safe)
        let rollback_plan = vec![
            RollbackStep {
                id: "restore-package-cache".to_string(),
                description: "Package cache was cleaned but old packages may be restored from backup".to_string(),
                command: "echo 'Note: Cleaned packages can be restored from /var/cache/pacman/pkg/ backups if made'".to_string(),
            },
        ];

        let mut analysis_parts = vec![
            "User reports package management issues.".to_string(),
            "Running comprehensive pacman diagnostics and repair sequence.".to_string(),
            format!("System has {:.1} GB free disk space.", disk_space_gb),
        ];

        if !has_internet {
            analysis_parts.push(
                "⚠️ WARNING: Internet connectivity not confirmed. Database sync will fail without network."
                    .to_string(),
            );
        }

        if disk_space_gb < 2.0 {
            analysis_parts
                .push("⚠️ WARNING: Low disk space. Package operations may fail.".to_string());
        }

        let analysis = analysis_parts.join(" ");

        let goals = vec![
            "Detect and remove stale pacman lock files".to_string(),
            "Synchronize package databases with mirrors".to_string(),
            "Clean old packages from cache to free space".to_string(),
            "Check for available updates and conflicts".to_string(),
            "Verify integrity of installed packages".to_string(),
            "Identify orphaned packages for potential removal".to_string(),
        ];

        let notes_for_user = "\
            This diagnostic and repair sequence will:\n\
            1. Remove stale pacman lock files (if safe to do so)\n\
            2. Sync package databases with mirrors\n\
            3. Clean old cached packages (keeps last 3 versions)\n\
            4. Check for system updates\n\
            5. Verify package file integrity\n\
            6. List orphaned packages\n\n\
            Common issues resolved:\n\
            • \"database lock\" errors\n\
            • \"failed to synchronize\" errors\n\
            • Package conflicts\n\
            • Cache corruption\n\n\
            If orphaned packages are found, you can remove them with:\n\
            sudo pacman -Rns $(pacman -Qtdq)\n\n\
            If package integrity issues are found, reinstall affected packages with:\n\
            sudo pacman -S --noconfirm <package-name>\n\n\
            Estimated time: 1-2 minutes\n\
            Cache cleanup may free: 100MB - 2GB"
            .to_string();

        // Metadata (Beta.151: Use PlanMeta for recipe tracking)
        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("packages.rs".to_string()),
        );
        other.insert(
            "disk_space_gb".to_string(),
            serde_json::Value::String(disk_space_gb.to_string()),
        );
        other.insert(
            "has_internet".to_string(),
            serde_json::Value::String(has_internet.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("package_repair".to_string()),
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
    fn test_matches_package_repair() {
        assert!(PackagesRecipe::matches_request("fix broken packages"));
        assert!(PackagesRecipe::matches_request("repair pacman"));
        assert!(PackagesRecipe::matches_request("package error"));
        assert!(PackagesRecipe::matches_request("pacman conflict"));
        assert!(PackagesRecipe::matches_request("fix package problems"));

        // Should not match
        assert!(!PackagesRecipe::matches_request("install a package"));
        assert!(!PackagesRecipe::matches_request(
            "what packages are installed"
        ));
    }

    #[test]
    fn test_build_packages_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("disk_free_gb".to_string(), "50.0".to_string());

        let plan = PackagesRecipe::build_plan(&telemetry).unwrap();

        // Verify structure
        assert_eq!(plan.goals.len(), 6);
        assert_eq!(plan.necessary_checks.len(), 4);
        assert_eq!(plan.command_plan.len(), 6);
        assert_eq!(plan.rollback_plan.len(), 1);

        // Verify key commands
        assert!(plan.command_plan[0].command.contains("db.lck"));
        assert!(plan.command_plan[1].command.contains("pacman -Syy"));
        assert!(plan.command_plan[2].command.contains("paccache"));

        // Verify metadata
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "package_repair");
        assert_eq!(plan.meta.llm_version, "deterministic_recipe_v1");
    }

    #[test]
    fn test_low_disk_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("disk_free_gb".to_string(), "1.0".to_string());

        let plan = PackagesRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("WARNING"));
        assert!(plan.analysis.contains("Low disk space"));
    }

    #[test]
    fn test_no_internet_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "false".to_string());
        telemetry.insert("disk_free_gb".to_string(), "50.0".to_string());

        let plan = PackagesRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("WARNING"));
        assert!(plan.analysis.contains("Internet connectivity"));
    }

    #[test]
    fn test_lock_removal_safety() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("disk_free_gb".to_string(), "50.0".to_string());

        let plan = PackagesRecipe::build_plan(&telemetry).unwrap();

        // Verify lock removal step checks for running pacman process
        let lock_step = &plan.command_plan[0];
        assert!(lock_step.command.contains("pgrep"));
        assert!(lock_step.command.contains("if"));
        assert_eq!(lock_step.risk_level, RiskLevel::Medium);
    }
}
