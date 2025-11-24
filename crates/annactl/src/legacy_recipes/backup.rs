// Beta.157: Backup Solutions Recipe
// Handles installation and configuration of backup tools (rsync, borg backup)

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct BackupRecipe;

#[derive(Debug, PartialEq)]
enum BackupOperation {
    Install,      // Install backup tools (rsync, borg)
    CheckStatus,  // Verify tool installation and backup status
    SetupBorg,    // Initialize borg repository
    CreateBackup, // Create backup with rsync or borg
}

impl BackupOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();

        // Check status (highest priority)
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || input_lower.contains("list")
            || input_lower.contains("show")
        {
            return BackupOperation::CheckStatus;
        }

        // Create backup
        if input_lower.contains("create")
            || input_lower.contains("make")
            || input_lower.contains("run")
            || input_lower.contains("start")
            || input_lower.contains("perform")
            || input_lower.contains("execute")
        {
            return BackupOperation::CreateBackup;
        }

        // Setup borg repository
        if input_lower.contains("setup borg")
            || input_lower.contains("init")
            || input_lower.contains("initialize")
            || input_lower.contains("configure borg")
            || (input_lower.contains("borg") && input_lower.contains("repo"))
        {
            return BackupOperation::SetupBorg;
        }

        // Default to install
        BackupOperation::Install
    }
}

impl BackupRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        let has_backup_context = input_lower.contains("backup")
            || input_lower.contains("rsync")
            || input_lower.contains("borg")
            || input_lower.contains("borgbackup")
            || (input_lower.contains("sync") && input_lower.contains("file"))
            || input_lower.contains("archive")
            || (input_lower.contains("snapshot") && input_lower.contains("data"));

        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("create")
            || input_lower.contains("make")
            || input_lower.contains("run")
            || input_lower.contains("start")
            || input_lower.contains("init")
            || input_lower.contains("add")
            || input_lower.contains("set")
            || input_lower.contains("perform")
            || input_lower.contains("execute");

        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_backup_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = BackupOperation::detect(user_input);

        match operation {
            BackupOperation::Install => Self::build_install_plan(telemetry),
            BackupOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            BackupOperation::SetupBorg => Self::build_setup_borg_plan(telemetry),
            BackupOperation::CreateBackup => Self::build_create_backup_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-pacman".to_string(),
                description: "Verify pacman is available".to_string(),
                command: "which pacman".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-tools".to_string(),
                description: "Check which backup tools are already installed".to_string(),
                command: "pacman -Q rsync borg 2>/dev/null || true".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-rsync".to_string(),
                description: "Install rsync (fast incremental file transfer)".to_string(),
                command: "sudo pacman -S --needed --noconfirm rsync".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-rsync".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-borg".to_string(),
                description: "Install borg (deduplicating archiver with encryption)".to_string(),
                command: "sudo pacman -S --needed --noconfirm borg".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-borg".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: "Verify all tools are installed".to_string(),
                command: "pacman -Q rsync borg".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-rsync".to_string(),
                description: "Remove rsync".to_string(),
                command: "sudo pacman -Rns --noconfirm rsync".to_string(),
            },
            RollbackStep {
                id: "remove-borg".to_string(),
                description: "Remove borg".to_string(),
                command: "sudo pacman -Rns --noconfirm borg".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("backup.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tools".to_string(), serde_json::json!("rsync, borg"));

        Ok(ActionPlan {
            analysis: "Installing two complementary backup tools: rsync for fast incremental file transfer and borg for encrypted deduplicating archives.".to_string(),
            goals: vec![
                "Install rsync for file synchronization and incremental backups".to_string(),
                "Install borg for encrypted, deduplicated backup archives".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Backup tools installed:\n\nrsync: Fast incremental file transfer\n- Great for syncing directories\n- No compression or encryption by default\n- Usage: rsync -av source/ destination/\n\nborg: Deduplicating archiver with encryption\n- Compressed and encrypted backups\n- Space-efficient (only stores changes)\n- Requires repository initialization\n- Usage: borg create /path/to/repo::archive /path/to/backup".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("backup_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-rsync".to_string(),
                description: "Check if rsync is installed".to_string(),
                command: "which rsync".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-borg".to_string(),
                description: "Check if borg is installed".to_string(),
                command: "which borg".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "list-backup-tools".to_string(),
                description: "List all installed backup tools".to_string(),
                command: "pacman -Q rsync borg 2>/dev/null || echo 'No backup tools installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-tool-versions".to_string(),
                description: "Show version information for installed tools".to_string(),
                command: "echo '--- rsync ---' && rsync --version 2>/dev/null | head -1 || echo 'Not installed'; echo '--- borg ---' && borg --version 2>/dev/null || echo 'Not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("backup.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking which backup tools are currently installed and their versions.".to_string(),
            goals: vec![
                "Verify rsync installation status".to_string(),
                "Verify borg installation status".to_string(),
                "Show version information".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "This will show which backup tools (rsync, borg) are currently installed on your system and their versions.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("backup_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_setup_borg_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-borg-installed".to_string(),
                description: "Verify borg is installed".to_string(),
                command: "which borg".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-init-instructions".to_string(),
                description: "Show borg repository initialization instructions".to_string(),
                command: "echo 'To initialize a borg repository, run:\n\nborg init --encryption=repokey /path/to/repo\n\nEncryption modes:\n- repokey: Encryption key stored in repo (recommended)\n- keyfile: Encryption key stored in ~/.config/borg/keys/\n- none: No encryption (not recommended)\n\nExample:\nborg init --encryption=repokey ~/backups/borg-repo'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("backup.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("SetupBorg"));

        Ok(ActionPlan {
            analysis: "Providing instructions for initializing a borg backup repository with encryption.".to_string(),
            goals: vec![
                "Explain borg repository initialization".to_string(),
                "Describe encryption options".to_string(),
                "Provide example commands".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Borg requires a repository to be initialized before creating backups.\n\nSteps:\n1. Choose a location for your backup repository (e.g., ~/backups/borg-repo or external drive)\n2. Initialize with: borg init --encryption=repokey /path/to/repo\n3. Set a strong passphrase (CRITICAL - you cannot recover backups without it!)\n4. Create backups with: borg create /path/to/repo::archive-name /data/to/backup\n\nNote: Anna cannot run the initialization interactively (requires passphrase input). Run the borg init command manually in your terminal.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("backup_setup_borg".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_create_backup_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-backup-tools".to_string(),
                description: "Check which backup tools are available".to_string(),
                command: "pacman -Q rsync borg 2>/dev/null || true".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-backup-examples".to_string(),
                description: "Show backup command examples".to_string(),
                command: "echo '=== Backup Examples ===\n\n--- rsync (Simple file sync) ---\nrsync -av --progress /source/path/ /backup/destination/\n\nOptions:\n  -a: Archive mode (preserves permissions, timestamps)\n  -v: Verbose output\n  --progress: Show transfer progress\n  --delete: Delete files in destination not in source\n\nExample:\nrsync -av --progress ~/Documents/ /mnt/backup/Documents/\n\n--- borg (Encrypted archive) ---\nborg create /path/to/repo::$(date +%Y-%m-%d) /data/to/backup\n\nExample:\nborg create ~/backups/borg-repo::$(date +%Y-%m-%d) ~/Documents ~/Pictures\n\nList archives:\nborg list ~/backups/borg-repo\n\nRestore:\nborg extract ~/backups/borg-repo::2025-11-20'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("backup.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CreateBackup"));

        Ok(ActionPlan {
            analysis: "Providing examples and guidance for creating backups with rsync and borg.".to_string(),
            goals: vec![
                "Show rsync backup examples".to_string(),
                "Show borg backup examples".to_string(),
                "Explain backup options and usage".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Anna cannot run backup commands automatically (they require your specific source/destination paths).\n\nChoose a backup method:\n\nrsync: Best for simple file syncing\n- Fast and efficient\n- No compression or encryption\n- Good for local backups or NAS\n\nborg: Best for long-term archives\n- Encrypted and compressed\n- Space-efficient (deduplication)\n- Requires repository setup first\n\nRun the appropriate command manually with your specific paths.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("backup_create_backup".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_backup_keywords() {
        assert!(BackupRecipe::matches_request("install backup tools"));
        assert!(BackupRecipe::matches_request("setup rsync"));
        assert!(BackupRecipe::matches_request("install borg"));
        assert!(BackupRecipe::matches_request("configure borgbackup"));
    }

    #[test]
    fn test_matches_backup_actions() {
        assert!(BackupRecipe::matches_request("create backup"));
        assert!(BackupRecipe::matches_request("make rsync backup"));
        assert!(BackupRecipe::matches_request("run borg backup"));
    }

    #[test]
    fn test_does_not_match_info_queries() {
        assert!(!BackupRecipe::matches_request("what is borg"));
        assert!(!BackupRecipe::matches_request("tell me about rsync"));
        assert!(!BackupRecipe::matches_request("explain backup"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            BackupOperation::detect("install backup tools"),
            BackupOperation::Install
        );
        assert_eq!(
            BackupOperation::detect("check backup status"),
            BackupOperation::CheckStatus
        );
        assert_eq!(
            BackupOperation::detect("setup borg repository"),
            BackupOperation::SetupBorg
        );
        assert_eq!(
            BackupOperation::detect("create borg backup"),
            BackupOperation::CreateBackup
        );
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install backup tools".to_string());

        let plan = BackupRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("rsync")));
        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("borg")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check backup tools".to_string());

        let plan = BackupRecipe::build_check_status_plan(&telemetry).unwrap();

        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_setup_borg_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "setup borg".to_string());

        let plan = BackupRecipe::build_setup_borg_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("borg") || g.to_lowercase().contains("init") || g.to_lowercase().contains("encryption")));
        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
    }

    #[test]
    fn test_create_backup_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "create backup".to_string());

        let plan = BackupRecipe::build_create_backup_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("backup") || g.to_lowercase().contains("example")));
        assert!(!plan.command_plan.is_empty());
    }
}
