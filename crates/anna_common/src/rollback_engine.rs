//! Rollback Engine - Build rollback plans from action episodes
//!
//! v6.49.0: Compute inverse commands for safe rollback

use crate::action_episodes::{ActionEpisode, ActionKind, ActionRecord, RollbackCapability};

/// Single rollback action with inverse commands
#[derive(Debug, Clone)]
pub struct RollbackAction {
    pub original: ActionRecord,
    pub inverse_commands: Vec<String>,
}

/// Complete rollback plan
#[derive(Debug, Clone)]
pub struct RollbackPlan {
    pub episode: ActionEpisode,
    pub actions: Vec<RollbackAction>,
    pub capability: RollbackCapability,
    pub summary: String,
}

/// Build rollback plan from episode
pub fn build_rollback_plan(episode: &ActionEpisode) -> RollbackPlan {
    let mut actions = Vec::new();
    let mut has_full_inverse = true;
    let mut file_count = 0;
    let mut package_count = 0;
    let mut service_count = 0;

    for action in &episode.actions {
        let inverse_commands = compute_inverse_commands(action);

        if inverse_commands.is_empty() && !matches!(action.kind, ActionKind::RunCommand) {
            has_full_inverse = false;
        }

        // Count actions by type
        match action.kind {
            ActionKind::EditFile | ActionKind::CreateFile | ActionKind::DeleteFile | ActionKind::MoveFile => {
                if !inverse_commands.is_empty() {
                    file_count += 1;
                }
            }
            ActionKind::InstallPackages | ActionKind::RemovePackages => {
                if !inverse_commands.is_empty() {
                    package_count += 1;
                }
            }
            ActionKind::EnableServices | ActionKind::DisableServices
            | ActionKind::StartServices | ActionKind::StopServices => {
                if !inverse_commands.is_empty() {
                    service_count += 1;
                }
            }
            ActionKind::RunCommand => {}
        }

        actions.push(RollbackAction {
            original: action.clone(),
            inverse_commands,
        });
    }

    // Determine capability
    let capability = if episode.rollback_capability == RollbackCapability::None {
        RollbackCapability::None
    } else if has_full_inverse {
        RollbackCapability::Full
    } else {
        RollbackCapability::Partial
    };

    // Build summary
    let mut summary_parts = Vec::new();

    if file_count > 0 {
        summary_parts.push(format!(
            "Restore {} file backup{}",
            file_count,
            if file_count == 1 { "" } else { "s" }
        ));
    }

    if package_count > 0 {
        summary_parts.push(format!(
            "undo {} package change{}",
            package_count,
            if package_count == 1 { "" } else { "s" }
        ));
    }

    if service_count > 0 {
        summary_parts.push(format!(
            "revert {} service change{}",
            service_count,
            if service_count == 1 { "" } else { "s" }
        ));
    }

    let summary = if summary_parts.is_empty() {
        "No reversible actions found".to_string()
    } else {
        summary_parts.join(" and ")
    };

    RollbackPlan {
        episode: episode.clone(),
        actions,
        capability,
        summary,
    }
}

/// Compute inverse commands for an action
fn compute_inverse_commands(action: &ActionRecord) -> Vec<String> {
    match action.kind {
        ActionKind::EditFile | ActionKind::CreateFile => {
            // Restore from backup
            if action.backup_paths.len() == 1 && action.files_touched.len() == 1 {
                vec![format!(
                    "cp {} {}",
                    action.backup_paths[0], action.files_touched[0]
                )]
            } else if !action.backup_paths.is_empty() && !action.files_touched.is_empty() {
                // Multiple files - generate multiple cp commands
                action
                    .backup_paths
                    .iter()
                    .zip(action.files_touched.iter())
                    .map(|(backup, target)| format!("cp {} {}", backup, target))
                    .collect()
            } else {
                vec![]
            }
        }

        ActionKind::DeleteFile => {
            // Restore from backup if available
            if !action.backup_paths.is_empty() && !action.files_touched.is_empty() {
                action
                    .backup_paths
                    .iter()
                    .zip(action.files_touched.iter())
                    .map(|(backup, target)| format!("cp {} {}", backup, target))
                    .collect()
            } else {
                vec![]
            }
        }

        ActionKind::MoveFile => {
            // Reverse the move
            if action.files_touched.len() >= 2 {
                vec![format!(
                    "mv {} {}",
                    action.files_touched[1], action.files_touched[0]
                )]
            } else {
                vec![]
            }
        }

        ActionKind::InstallPackages => {
            // Remove installed packages
            extract_package_commands(&action.command, "install")
        }

        ActionKind::RemovePackages => {
            // Reinstall removed packages
            extract_package_commands(&action.command, "remove")
        }

        ActionKind::EnableServices => {
            // Disable services
            extract_service_commands(&action.command, "enable")
        }

        ActionKind::DisableServices => {
            // Enable services
            extract_service_commands(&action.command, "disable")
        }

        ActionKind::StartServices => {
            // Stop services
            extract_service_commands(&action.command, "start")
        }

        ActionKind::StopServices => {
            // Start services
            extract_service_commands(&action.command, "stop")
        }

        ActionKind::RunCommand => {
            // Read-only commands have no inverse
            vec![]
        }
    }
}

/// Extract package commands
fn extract_package_commands(command: &str, operation: &str) -> Vec<String> {
    // Parse package manager commands
    if operation == "install" {
        // Original was install, reverse is remove
        if command.contains("pacman -S") || command.contains("yay -S") {
            let tool = if command.contains("yay") { "yay" } else { "pacman" };
            if let Some(packages) = extract_packages_from_command(command) {
                return vec![format!("{} -Rns {}", tool, packages)];
            }
        }
    } else if operation == "remove" {
        // Original was remove, reverse is install
        if command.contains("pacman -R") || command.contains("yay -R") {
            let tool = if command.contains("yay") { "yay" } else { "pacman" };
            if let Some(packages) = extract_packages_from_command(command) {
                return vec![format!("{} -S {}", tool, packages)];
            }
        }
    }

    vec![]
}

/// Extract service commands
fn extract_service_commands(command: &str, operation: &str) -> Vec<String> {
    // Parse systemctl commands
    if !command.contains("systemctl") {
        return vec![];
    }

    let inverse_op = match operation {
        "enable" => "disable",
        "disable" => "enable",
        "start" => "stop",
        "stop" => "start",
        _ => return vec![],
    };

    // Extract service names
    if let Some(services) = extract_services_from_command(command) {
        vec![format!("systemctl {} {}", inverse_op, services)]
    } else {
        vec![]
    }
}

/// Extract package names from command
fn extract_packages_from_command(command: &str) -> Option<String> {
    // Simple extraction: everything after -S or -R
    let parts: Vec<&str> = command.split_whitespace().collect();

    for (i, part) in parts.iter().enumerate() {
        if part.starts_with("-S") || part.starts_with("-R") {
            // Collect remaining parts as package names
            let packages: Vec<&str> = parts[i + 1..].iter().copied().collect();
            if !packages.is_empty() {
                return Some(packages.join(" "));
            }
        }
    }

    None
}

/// Extract service names from command
fn extract_services_from_command(command: &str) -> Option<String> {
    // Extract service names after enable/disable/start/stop
    let parts: Vec<&str> = command.split_whitespace().collect();

    for (i, part) in parts.iter().enumerate() {
        if matches!(*part, "enable" | "disable" | "start" | "stop") {
            // Collect remaining parts as service names
            let services: Vec<&str> = parts[i + 1..].iter().copied().collect();
            if !services.is_empty() {
                return Some(services.join(" "));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::action_episodes::EpisodeTags;

    fn make_test_episode() -> ActionEpisode {
        ActionEpisode {
            episode_id: 1,
            created_at: Utc::now(),
            user_question: "test".to_string(),
            final_answer_summary: "test".to_string(),
            tags: EpisodeTags {
                topics: vec![],
                domain: None,
            },
            actions: vec![],
            rollback_capability: RollbackCapability::Full,
            execution_status: crate::action_episodes::ExecutionStatus::PlannedOnly,
            post_validation: None,
            rolled_back_episode_id: None,
        }
    }

    #[test]
    fn test_file_edit_with_backup() {
        let mut episode = make_test_episode();
        episode.actions.push(ActionRecord {
            id: 1,
            kind: ActionKind::EditFile,
            command: "edit ~/.vimrc".to_string(),
            cwd: None,
            files_touched: vec!["/home/user/.vimrc".to_string()],
            backup_paths: vec!["/home/user/.vimrc.anna-backup".to_string()],
            notes: Some("vim config".to_string()),
        });

        let plan = build_rollback_plan(&episode);

        assert_eq!(plan.capability, RollbackCapability::Full);
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].inverse_commands.len(), 1);
        assert_eq!(
            plan.actions[0].inverse_commands[0],
            "cp /home/user/.vimrc.anna-backup /home/user/.vimrc"
        );
        assert!(plan.summary.contains("1 file backup"));
    }

    #[test]
    fn test_file_edit_missing_backup() {
        let mut episode = make_test_episode();
        episode.actions.push(ActionRecord {
            id: 1,
            kind: ActionKind::EditFile,
            command: "edit file".to_string(),
            cwd: None,
            files_touched: vec!["/test.txt".to_string()],
            backup_paths: vec![], // No backup!
            notes: None,
        });

        let plan = build_rollback_plan(&episode);

        assert_eq!(plan.capability, RollbackCapability::Partial);
        assert_eq!(plan.actions[0].inverse_commands.len(), 0);
    }

    #[test]
    fn test_package_install() {
        let mut episode = make_test_episode();
        episode.actions.push(ActionRecord {
            id: 1,
            kind: ActionKind::InstallPackages,
            command: "yay -S docker vim".to_string(),
            cwd: None,
            files_touched: vec![],
            backup_paths: vec![],
            notes: Some("installed packages".to_string()),
        });

        let plan = build_rollback_plan(&episode);

        assert_eq!(plan.capability, RollbackCapability::Full);
        assert_eq!(plan.actions[0].inverse_commands.len(), 1);
        assert_eq!(plan.actions[0].inverse_commands[0], "yay -Rns docker vim");
        assert!(plan.summary.contains("1 package change"));
    }

    #[test]
    fn test_package_remove() {
        let mut episode = make_test_episode();
        episode.actions.push(ActionRecord {
            id: 1,
            kind: ActionKind::RemovePackages,
            command: "pacman -Rns firefox".to_string(),
            cwd: None,
            files_touched: vec![],
            backup_paths: vec![],
            notes: Some("removed package".to_string()),
        });

        let plan = build_rollback_plan(&episode);

        assert_eq!(plan.capability, RollbackCapability::Full);
        assert_eq!(plan.actions[0].inverse_commands.len(), 1);
        assert_eq!(plan.actions[0].inverse_commands[0], "pacman -S firefox");
    }

    #[test]
    fn test_service_enable() {
        let mut episode = make_test_episode();
        episode.actions.push(ActionRecord {
            id: 1,
            kind: ActionKind::EnableServices,
            command: "systemctl enable sshd".to_string(),
            cwd: None,
            files_touched: vec![],
            backup_paths: vec![],
            notes: Some("enabled sshd".to_string()),
        });

        let plan = build_rollback_plan(&episode);

        assert_eq!(plan.capability, RollbackCapability::Full);
        assert_eq!(plan.actions[0].inverse_commands.len(), 1);
        assert_eq!(plan.actions[0].inverse_commands[0], "systemctl disable sshd");
        assert!(plan.summary.contains("1 service change"));
    }

    #[test]
    fn test_run_command_no_inverse() {
        let mut episode = make_test_episode();
        episode.actions.push(ActionRecord {
            id: 1,
            kind: ActionKind::RunCommand,
            command: "ls -la".to_string(),
            cwd: None,
            files_touched: vec![],
            backup_paths: vec![],
            notes: Some("inspection".to_string()),
        });

        let plan = build_rollback_plan(&episode);

        // RunCommand doesn't affect capability if it's the only action
        assert_eq!(plan.actions[0].inverse_commands.len(), 0);
        assert_eq!(plan.summary, "No reversible actions found");
    }

    #[test]
    fn test_mixed_episode() {
        let mut episode = make_test_episode();

        episode.actions.push(ActionRecord {
            id: 1,
            kind: ActionKind::EditFile,
            command: "edit file".to_string(),
            cwd: None,
            files_touched: vec!["/file.txt".to_string()],
            backup_paths: vec!["/file.txt.bak".to_string()],
            notes: None,
        });

        episode.actions.push(ActionRecord {
            id: 2,
            kind: ActionKind::InstallPackages,
            command: "yay -S vim".to_string(),
            cwd: None,
            files_touched: vec![],
            backup_paths: vec![],
            notes: None,
        });

        let plan = build_rollback_plan(&episode);

        assert_eq!(plan.capability, RollbackCapability::Full);
        assert_eq!(plan.actions.len(), 2);
        assert!(plan.summary.contains("file backup"));
        assert!(plan.summary.contains("package change"));
    }
}
