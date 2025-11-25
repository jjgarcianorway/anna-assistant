//! Action Episodes - Episodic Action Log for rollback capability
//!
//! v6.49.0: "If Anna did it, Anna can undo it"

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of action performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionKind {
    EditFile,
    CreateFile,
    DeleteFile,
    MoveFile,
    InstallPackages,
    RemovePackages,
    EnableServices,
    DisableServices,
    StartServices,
    StopServices,
    RunCommand, // read-only / inspection
}

/// Rollback capability for an episode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RollbackCapability {
    Full,    // can be cleanly reversed
    Partial, // some parts reversible
    None,    // cannot be safely reversed
}

/// Single action within an episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: i64, // local to episode
    pub kind: ActionKind,
    pub command: String,            // exact shell command executed
    pub cwd: Option<String>,        // working directory
    pub files_touched: Vec<String>, // paths touched
    pub backup_paths: Vec<String>,  // backup file paths created
    pub notes: Option<String>,      // short explanation
}

/// Tags for categorizing episodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpisodeTags {
    pub topics: Vec<String>,     // ["vim", "config", "editor"]
    pub domain: Option<String>,  // "audio", "network", "packages", etc
}

/// Post-validation result from LLM
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostValidation {
    pub satisfaction_score: f32,      // 0.0 - 1.0
    pub summary: String,               // 1-3 sentences
    pub residual_concerns: Vec<String>, // Short list of potential issues
    pub suggested_checks: Vec<String>, // At most 3 further commands
}

/// Episode execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    PlannedOnly,      // Plan generated but not executed
    Executed,         // Successfully executed
    PartiallyExecuted, // Some commands failed
    RolledBack,       // This episode was rolled back
}

/// Complete episode of actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEpisode {
    pub episode_id: i64,
    pub created_at: DateTime<Utc>,
    pub user_question: String,
    pub final_answer_summary: String,
    pub tags: EpisodeTags,
    pub actions: Vec<ActionRecord>,
    pub rollback_capability: RollbackCapability,
    pub execution_status: ExecutionStatus,
    pub post_validation: Option<PostValidation>,
    pub rolled_back_episode_id: Option<i64>, // If this is a rollback, link to original
}

/// Builder for constructing episodes
pub struct EpisodeBuilder {
    user_question: String,
    final_answer_summary: Option<String>,
    tags: Option<EpisodeTags>,
    actions: Vec<ActionRecord>,
    next_action_id: i64,
}

impl EpisodeBuilder {
    /// Create new episode builder
    pub fn new(user_question: &str) -> Self {
        Self {
            user_question: user_question.to_string(),
            final_answer_summary: None,
            tags: None,
            actions: Vec::new(),
            next_action_id: 1,
        }
    }

    /// Set final answer summary
    pub fn with_final_answer_summary(mut self, summary: &str) -> Self {
        self.final_answer_summary = Some(summary.to_string());
        self
    }

    /// Set tags
    pub fn with_tags(mut self, tags: EpisodeTags) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Add an action
    pub fn add_action(&mut self, mut action: ActionRecord) {
        action.id = self.next_action_id;
        self.next_action_id += 1;
        self.actions.push(action);
    }

    /// Infer rollback capability from actions
    pub fn infer_rollback_capability(&self) -> RollbackCapability {
        if self.actions.is_empty() {
            return RollbackCapability::None;
        }

        for action in &self.actions {
            match action.kind {
                ActionKind::DeleteFile => {
                    // DeleteFile with no backup is unsafe
                    if action.backup_paths.is_empty() {
                        return RollbackCapability::None;
                    }
                }
                ActionKind::RunCommand => {
                    // Read-only commands don't affect rollback
                }
                _ => {}
            }
        }

        // If we have any state-changing actions, assume Full
        let has_state_change = self.actions.iter().any(|a| {
            !matches!(a.kind, ActionKind::RunCommand)
        });

        if has_state_change {
            RollbackCapability::Full
        } else {
            RollbackCapability::None
        }
    }

    /// Build the episode
    pub fn build(self) -> ActionEpisode {
        let rollback_capability = self.infer_rollback_capability();

        ActionEpisode {
            episode_id: 0, // Will be set by database
            created_at: Utc::now(),
            user_question: self.user_question,
            final_answer_summary: self.final_answer_summary.unwrap_or_default(),
            tags: self.tags.unwrap_or_else(|| EpisodeTags {
                topics: vec![],
                domain: None,
            }),
            actions: self.actions,
            rollback_capability,
            execution_status: ExecutionStatus::PlannedOnly,
            post_validation: None,
            rolled_back_episode_id: None,
        }
    }
}

/// Infer tags from plan and answer
pub fn infer_tags_from_plan_and_answer(
    user_question: &str,
    plan_summary: &str,
    interpreter_summary: &str,
) -> EpisodeTags {
    let combined = format!(
        "{} {} {}",
        user_question.to_lowercase(),
        plan_summary.to_lowercase(),
        interpreter_summary.to_lowercase()
    );

    let mut topics = Vec::new();
    let mut domain = None;

    // Editor-related
    if combined.contains("vim") || combined.contains(".vimrc") {
        topics.push("vim".to_string());
        topics.push("editor".to_string());
        domain = Some("editor".to_string());
    } else if combined.contains("neovim") || combined.contains("nvim") {
        topics.push("neovim".to_string());
        topics.push("editor".to_string());
        domain = Some("editor".to_string());
    } else if combined.contains("emacs") {
        topics.push("emacs".to_string());
        topics.push("editor".to_string());
        domain = Some("editor".to_string());
    }

    // Network-related
    if combined.contains("ssh") {
        topics.push("ssh".to_string());
        topics.push("network".to_string());
        domain = Some("network".to_string());
    }

    if combined.contains("firewall") || combined.contains("ufw") || combined.contains("iptables") {
        topics.push("firewall".to_string());
        topics.push("security".to_string());
        domain = Some("network".to_string());
    }

    // Audio-related
    if combined.contains("audio") || combined.contains("pipewire")
        || combined.contains("pulseaudio") || combined.contains("alsa") {
        topics.push("audio".to_string());
        domain = Some("audio".to_string());
    }

    // Package management
    if combined.contains("pacman") || combined.contains("yay")
        || combined.contains("install") || combined.contains("remove") {
        if combined.contains("install") {
            topics.push("packages".to_string());
            topics.push("install".to_string());
        }
        if combined.contains("remove") {
            topics.push("packages".to_string());
            topics.push("remove".to_string());
        }
        if domain.is_none() {
            domain = Some("packages".to_string());
        }
    }

    // Services
    if combined.contains("systemctl") || combined.contains("service") {
        topics.push("services".to_string());
        if domain.is_none() {
            domain = Some("system".to_string());
        }
    }

    // Desktop environment / Window manager
    if combined.contains("hyprland") || combined.contains("i3")
        || combined.contains("sway") || combined.contains("kde") || combined.contains("gnome") {
        topics.push("wm".to_string());
        topics.push("desktop".to_string());
        domain = Some("desktop".to_string());
    }

    // Configuration
    if combined.contains("config") || combined.contains("configuration") {
        topics.push("config".to_string());
    }

    EpisodeTags { topics, domain }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episode_builder_basic() {
        let mut builder = EpisodeBuilder::new("make my vim use 4 spaces");

        builder.add_action(ActionRecord {
            id: 0,
            kind: ActionKind::EditFile,
            command: "cp ~/.vimrc.anna-backup ~/.vimrc".to_string(),
            cwd: Some("/home/user".to_string()),
            files_touched: vec!["/home/user/.vimrc".to_string()],
            backup_paths: vec!["/home/user/.vimrc.anna-backup".to_string()],
            notes: Some("updated vim config".to_string()),
        });

        let episode = builder
            .with_final_answer_summary("Updated vim to use 4 spaces")
            .with_tags(EpisodeTags {
                topics: vec!["vim".to_string(), "editor".to_string()],
                domain: Some("editor".to_string()),
            })
            .build();

        assert_eq!(episode.user_question, "make my vim use 4 spaces");
        assert_eq!(episode.actions.len(), 1);
        assert_eq!(episode.actions[0].id, 1); // Builder assigns ID
        assert_eq!(episode.rollback_capability, RollbackCapability::Full);
    }

    #[test]
    fn test_rollback_capability_with_backups() {
        let mut builder = EpisodeBuilder::new("test");

        builder.add_action(ActionRecord {
            id: 0,
            kind: ActionKind::EditFile,
            command: "edit file".to_string(),
            cwd: None,
            files_touched: vec!["/test.txt".to_string()],
            backup_paths: vec!["/test.txt.bak".to_string()],
            notes: None,
        });

        let capability = builder.infer_rollback_capability();
        assert_eq!(capability, RollbackCapability::Full);
    }

    #[test]
    fn test_rollback_capability_delete_no_backup() {
        let mut builder = EpisodeBuilder::new("test");

        builder.add_action(ActionRecord {
            id: 0,
            kind: ActionKind::DeleteFile,
            command: "rm /test.txt".to_string(),
            cwd: None,
            files_touched: vec!["/test.txt".to_string()],
            backup_paths: vec![], // No backup!
            notes: None,
        });

        let capability = builder.infer_rollback_capability();
        assert_eq!(capability, RollbackCapability::None);
    }

    #[test]
    fn test_tag_inference_vim() {
        let tags = infer_tags_from_plan_and_answer(
            "make my vim use 4 spaces",
            "edit .vimrc to set tabstop=4",
            "Updated vim configuration",
        );

        assert!(tags.topics.contains(&"vim".to_string()));
        assert!(tags.topics.contains(&"editor".to_string()));
        assert_eq!(tags.domain, Some("editor".to_string()));
    }

    #[test]
    fn test_tag_inference_ssh() {
        let tags = infer_tags_from_plan_and_answer(
            "harden my ssh configuration",
            "update /etc/ssh/sshd_config",
            "Applied SSH hardening",
        );

        assert!(tags.topics.contains(&"ssh".to_string()));
        assert!(tags.topics.contains(&"network".to_string()));
        assert_eq!(tags.domain, Some("network".to_string()));
    }

    #[test]
    fn test_tag_inference_audio() {
        let tags = infer_tags_from_plan_and_answer(
            "fix my audio setup",
            "configure pipewire",
            "Configured audio system",
        );

        assert!(tags.topics.contains(&"audio".to_string()));
        assert_eq!(tags.domain, Some("audio".to_string()));
    }

    #[test]
    fn test_tag_inference_packages() {
        let tags = infer_tags_from_plan_and_answer(
            "install docker",
            "yay -S docker",
            "Installed docker package",
        );

        assert!(tags.topics.contains(&"packages".to_string()));
        assert!(tags.topics.contains(&"install".to_string()));
        assert_eq!(tags.domain, Some("packages".to_string()));
    }
}
