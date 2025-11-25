//! Episode Recorder - Convert execution into ActionEpisodes
//!
//! v6.50.0: Bridge between executor and episode storage

use crate::action_episodes::{
    ActionKind, ActionRecord, EpisodeBuilder, EpisodeTags, ExecutionStatus,
    infer_tags_from_plan_and_answer,
};
use crate::executor_core::ExecutionResult;
use crate::planner_core::{CommandPlan, PlannedCommand};
use anyhow::Result;

/// Records commands as they execute and builds an ActionEpisode
pub struct EpisodeRecorder {
    builder: EpisodeBuilder,
    all_succeeded: bool,
}

impl EpisodeRecorder {
    /// Create new recorder for a user query
    pub fn new(user_question: &str) -> Self {
        Self {
            builder: EpisodeBuilder::new(user_question),
            all_succeeded: true,
        }
    }

    /// Record a command execution
    pub fn record_command(
        &mut self,
        planned: &PlannedCommand,
        result: &ExecutionResult,
        backup_paths: Vec<String>,
    ) {
        // Determine ActionKind from command
        let full_command = format!("{} {}", planned.command, planned.args.join(" "));
        let kind = classify_action_kind(&full_command, planned);

        // Detect files touched
        let files_touched = detect_files_touched(&full_command, planned);

        // Create action record
        let action = ActionRecord {
            id: 0, // Will be set by builder
            kind,
            command: full_command,
            cwd: std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string()),
            files_touched,
            backup_paths,
            notes: Some(planned.purpose.clone()),
        };

        self.builder.add_action(action);

        // Track success
        if !result.success {
            self.all_succeeded = false;
        }
    }

    /// Finish recording and build the episode
    pub fn finish(
        mut self,
        plan: &CommandPlan,
        answer_summary: &str,
        user_question: &str,
    ) -> crate::action_episodes::ActionEpisode {
        // Infer tags
        let tags = infer_tags_from_plan_and_answer(
            user_question,
            &plan.reasoning,
            answer_summary,
        );

        // Build episode
        let mut episode = self
            .builder
            .with_final_answer_summary(answer_summary)
            .with_tags(tags)
            .build();

        // Set execution status
        episode.execution_status = if self.all_succeeded {
            ExecutionStatus::Executed
        } else {
            ExecutionStatus::PartiallyExecuted
        };

        episode
    }
}

/// Classify the type of action from the command
fn classify_action_kind(command: &str, planned: &PlannedCommand) -> ActionKind {
    let cmd_lower = command.to_lowercase();

    // Package operations
    if cmd_lower.contains("pacman -s")
        || cmd_lower.contains("yay -s")
        || cmd_lower.contains("paru -s")
    {
        return ActionKind::InstallPackages;
    }

    if cmd_lower.contains("pacman -r")
        || cmd_lower.contains("yay -r")
        || cmd_lower.contains("paru -r")
    {
        return ActionKind::RemovePackages;
    }

    // Service operations
    if cmd_lower.contains("systemctl enable") {
        return ActionKind::EnableServices;
    }

    if cmd_lower.contains("systemctl disable") {
        return ActionKind::DisableServices;
    }

    if cmd_lower.contains("systemctl start") {
        return ActionKind::StartServices;
    }

    if cmd_lower.contains("systemctl stop") {
        return ActionKind::StopServices;
    }

    // File operations
    if planned.writes_files {
        // Check if file exists to determine edit vs create
        if cmd_lower.contains("touch") || cmd_lower.contains("mkdir") {
            return ActionKind::CreateFile;
        }

        if cmd_lower.contains("rm ") || cmd_lower.contains("rm -") {
            return ActionKind::DeleteFile;
        }

        if cmd_lower.contains("mv ") {
            return ActionKind::MoveFile;
        }

        // Default to edit for other write operations
        return ActionKind::EditFile;
    }

    // Default: inspection/read-only
    ActionKind::RunCommand
}

/// Detect which files are touched by a command
fn detect_files_touched(command: &str, planned: &PlannedCommand) -> Vec<String> {
    let mut files = Vec::new();

    // Extract file paths from common patterns
    if command.contains("~/.") {
        // Extract ~/.something patterns
        for part in command.split_whitespace() {
            if part.starts_with("~/.") {
                // Expand ~ to home directory
                if let Some(home) = std::env::var_os("HOME") {
                    let expanded = part.replace("~", &home.to_string_lossy());
                    files.push(expanded);
                } else {
                    files.push(part.to_string());
                }
            }
        }
    }

    if command.contains("/etc/") {
        // Extract /etc/ paths
        for part in command.split_whitespace() {
            if part.contains("/etc/") {
                files.push(part.to_string());
            }
        }
    }

    // If we didn't detect any files but command writes, note that
    if files.is_empty() && planned.writes_files {
        files.push("(files modified)".to_string());
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner_core::StepRiskLevel;

    fn make_test_command(cmd: &str, args: Vec<&str>, writes: bool) -> PlannedCommand {
        PlannedCommand {
            command: cmd.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            purpose: "test".to_string(),
            requires_tools: vec![],
            risk_level: StepRiskLevel::ReadOnly,
            writes_files: writes,
            requires_root: false,
            expected_outcome: None,
            validation_hint: None,
        }
    }

    #[test]
    fn test_classify_package_install() {
        let cmd = make_test_command("pacman", vec!["-S", "vim"], false);
        let kind = classify_action_kind("pacman -S vim", &cmd);
        assert_eq!(kind, ActionKind::InstallPackages);
    }

    #[test]
    fn test_classify_package_remove() {
        let cmd = make_test_command("pacman", vec!["-R", "vim"], false);
        let kind = classify_action_kind("pacman -R vim", &cmd);
        assert_eq!(kind, ActionKind::RemovePackages);
    }

    #[test]
    fn test_classify_service_enable() {
        let cmd = make_test_command("systemctl", vec!["enable", "sshd"], false);
        let kind = classify_action_kind("systemctl enable sshd", &cmd);
        assert_eq!(kind, ActionKind::EnableServices);
    }

    #[test]
    fn test_classify_file_edit() {
        let cmd = make_test_command("vim", vec!["~/.vimrc"], true);
        let kind = classify_action_kind("vim ~/.vimrc", &cmd);
        assert_eq!(kind, ActionKind::EditFile);
    }

    #[test]
    fn test_classify_file_delete() {
        let cmd = make_test_command("rm", vec!["/tmp/test"], true);
        let kind = classify_action_kind("rm /tmp/test", &cmd);
        assert_eq!(kind, ActionKind::DeleteFile);
    }

    #[test]
    fn test_classify_read_only() {
        let cmd = make_test_command("ls", vec!["-la"], false);
        let kind = classify_action_kind("ls -la", &cmd);
        assert_eq!(kind, ActionKind::RunCommand);
    }

    #[test]
    fn test_detect_files_home() {
        let cmd = make_test_command("vim", vec!["~/.vimrc"], true);
        let files = detect_files_touched("vim ~/.vimrc", &cmd);
        assert!(!files.is_empty());
        assert!(files[0].contains(".vimrc"));
    }

    #[test]
    fn test_detect_files_etc() {
        let cmd = make_test_command("vim", vec!["/etc/ssh/sshd_config"], true);
        let files = detect_files_touched("vim /etc/ssh/sshd_config", &cmd);
        assert!(files.iter().any(|f| f.contains("/etc/ssh")));
    }
}
