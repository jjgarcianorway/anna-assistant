//! Change Logging - Track all system modifications for rollback
//!
//! Phase 5.1: Conversational UX
//! Every system change made by Anna is logged as a Change Unit for transparency and rollback

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single system modification that can potentially be rolled back
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeUnit {
    /// Unique identifier
    pub id: String,

    /// Human-readable label (e.g., "install-kde-and-set-default-session")
    pub label: String,

    /// User's original request that triggered this change
    pub user_request: String,

    /// When the change started
    pub start_time: DateTime<Utc>,

    /// When the change completed
    pub end_time: Option<DateTime<Utc>>,

    /// Individual actions performed
    pub actions: Vec<ChangeAction>,

    /// Result status
    pub status: ChangeStatus,

    /// Optional notes about tradeoffs or limitations
    pub notes: Vec<String>,

    /// Metrics snapshot before change (for degradation tracking)
    pub metrics_before: Option<MetricsSnapshot>,

    /// Metrics snapshot after change
    pub metrics_after: Option<MetricsSnapshot>,
}

/// Status of a change unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeStatus {
    /// Change is in progress
    InProgress,

    /// Change completed successfully
    Success,

    /// Change partially succeeded (some actions failed)
    Partial,

    /// Change failed completely
    Failed,

    /// Change was rolled back
    RolledBack,
}

/// A single action within a change unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeAction {
    /// Type of action
    pub action_type: ActionType,

    /// When this action was performed
    pub timestamp: DateTime<Utc>,

    /// Description for user
    pub description: String,

    /// Whether this action succeeded
    pub success: bool,

    /// Rollback information
    pub rollback_info: Option<RollbackInfo>,
}

/// Type of action performed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionType {
    /// Command execution
    Command {
        command: String,
        args: Vec<String>,
        exit_code: Option<i32>,
        stdout: Option<String>,
        stderr: Option<String>,
    },

    /// File modification
    FileModify {
        path: PathBuf,
        backup_path: Option<PathBuf>,
        diff: Option<String>,
    },

    /// File creation
    FileCreate {
        path: PathBuf,
    },

    /// File deletion
    FileDelete {
        path: PathBuf,
        backup_path: Option<PathBuf>,
    },

    /// Package installation
    PackageInstall {
        packages: Vec<String>,
        size_mb: Option<f64>,
    },

    /// Package removal
    PackageRemove {
        packages: Vec<String>,
    },

    /// Service state change
    ServiceChange {
        service: String,
        old_state: String,
        new_state: String,
    },
}

/// Information needed to rollback an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    /// Can this action be rolled back?
    pub can_rollback: bool,

    /// Reason if rollback is not possible
    pub cannot_rollback_reason: Option<String>,

    /// Commands to execute for rollback
    pub rollback_commands: Vec<String>,

    /// Files to restore
    pub files_to_restore: Vec<(PathBuf, PathBuf)>, // (target, backup)
}

/// System metrics snapshot for degradation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,

    /// Boot time in seconds
    pub boot_time_secs: Option<f64>,

    /// Idle RAM usage in MB
    pub idle_ram_mb: Option<f64>,

    /// Average CPU usage (last 15 min)
    pub avg_cpu_percent: Option<f64>,

    /// Disk usage percentage
    pub disk_usage_percent: Option<f64>,

    /// Number of running services
    pub service_count: Option<usize>,

    /// Custom metrics
    pub custom: std::collections::HashMap<String, f64>,
}

impl ChangeUnit {
    /// Create a new change unit
    pub fn new(label: impl Into<String>, user_request: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            label: label.into(),
            user_request: user_request.into(),
            start_time: Utc::now(),
            end_time: None,
            actions: Vec::new(),
            status: ChangeStatus::InProgress,
            notes: Vec::new(),
            metrics_before: None,
            metrics_after: None,
        }
    }

    /// Add an action to this change unit
    pub fn add_action(&mut self, action: ChangeAction) {
        self.actions.push(action);
    }

    /// Mark this change as completed
    pub fn complete(&mut self, status: ChangeStatus) {
        self.end_time = Some(Utc::now());
        self.status = status;
    }

    /// Add a note about tradeoffs or limitations
    pub fn add_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// Set metrics before change
    pub fn set_metrics_before(&mut self, metrics: MetricsSnapshot) {
        self.metrics_before = Some(metrics);
    }

    /// Set metrics after change
    pub fn set_metrics_after(&mut self, metrics: MetricsSnapshot) {
        self.metrics_after = Some(metrics);
    }

    /// Check if this change can be rolled back
    pub fn can_rollback(&self) -> bool {
        if self.status != ChangeStatus::Success && self.status != ChangeStatus::Partial {
            return false;
        }

        self.actions.iter().all(|action| {
            action.rollback_info
                .as_ref()
                .map(|info| info.can_rollback)
                .unwrap_or(false)
        })
    }

    /// Get rollback limitations
    pub fn rollback_limitations(&self) -> Vec<String> {
        self.actions
            .iter()
            .filter_map(|action| {
                action.rollback_info.as_ref().and_then(|info| {
                    if !info.can_rollback {
                        info.cannot_rollback_reason.clone()
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

impl ChangeAction {
    /// Create a command action
    pub fn command(
        command: impl Into<String>,
        args: Vec<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            action_type: ActionType::Command {
                command: command.into(),
                args,
                exit_code: None,
                stdout: None,
                stderr: None,
            },
            timestamp: Utc::now(),
            description: description.into(),
            success: false,
            rollback_info: None,
        }
    }

    /// Create a file modification action
    pub fn file_modify(
        path: PathBuf,
        backup_path: PathBuf,
        description: impl Into<String>,
    ) -> Self {
        Self {
            action_type: ActionType::FileModify {
                path: path.clone(),
                backup_path: Some(backup_path.clone()),
                diff: None,
            },
            timestamp: Utc::now(),
            description: description.into(),
            success: false,
            rollback_info: Some(RollbackInfo {
                can_rollback: true,
                cannot_rollback_reason: None,
                rollback_commands: Vec::new(),
                files_to_restore: vec![(path, backup_path)],
            }),
        }
    }

    /// Create a package install action
    pub fn package_install(packages: Vec<String>, description: impl Into<String>) -> Self {
        let rollback_commands = vec![format!("pacman -Rns {}", packages.join(" "))];

        Self {
            action_type: ActionType::PackageInstall {
                packages: packages.clone(),
                size_mb: None,
            },
            timestamp: Utc::now(),
            description: description.into(),
            success: false,
            rollback_info: Some(RollbackInfo {
                can_rollback: true,
                cannot_rollback_reason: None,
                rollback_commands,
                files_to_restore: Vec::new(),
            }),
        }
    }
}

impl MetricsSnapshot {
    /// Create a new metrics snapshot with current timestamp
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            boot_time_secs: None,
            idle_ram_mb: None,
            avg_cpu_percent: None,
            disk_usage_percent: None,
            service_count: None,
            custom: std::collections::HashMap::new(),
        }
    }

    /// Calculate degradation compared to another snapshot
    pub fn degradation_vs(&self, other: &MetricsSnapshot) -> Vec<String> {
        let mut degradations = Vec::new();

        if let (Some(current), Some(previous)) = (self.boot_time_secs, other.boot_time_secs) {
            let diff = current - previous;
            if diff > 2.0 {
                degradations.push(format!("Boot time increased by {:.1}s", diff));
            }
        }

        if let (Some(current), Some(previous)) = (self.idle_ram_mb, other.idle_ram_mb) {
            let diff = current - previous;
            if diff > 100.0 {
                degradations.push(format!("Idle RAM usage increased by {:.0} MB", diff));
            }
        }

        if let (Some(current), Some(previous)) = (self.disk_usage_percent, other.disk_usage_percent) {
            let diff = current - previous;
            if diff > 5.0 {
                degradations.push(format!("Disk usage increased by {:.1}%", diff));
            }
        }

        degradations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_unit_creation() {
        let unit = ChangeUnit::new("test-change", "user request");
        assert_eq!(unit.label, "test-change");
        assert_eq!(unit.user_request, "user request");
        assert_eq!(unit.status, ChangeStatus::InProgress);
        assert!(unit.actions.is_empty());
    }

    #[test]
    fn test_rollback_check() {
        let mut unit = ChangeUnit::new("test", "request");

        // Add action that can be rolled back
        let mut action = ChangeAction::package_install(vec!["test-pkg".to_string()], "Install test");
        action.success = true;
        unit.add_action(action);

        unit.complete(ChangeStatus::Success);

        assert!(unit.can_rollback());
    }

    #[test]
    fn test_metrics_degradation() {
        let mut old = MetricsSnapshot::new();
        old.boot_time_secs = Some(10.0);
        old.idle_ram_mb = Some(1000.0);

        let mut new = MetricsSnapshot::new();
        new.boot_time_secs = Some(13.0);
        new.idle_ram_mb = Some(1200.0);

        let degradations = new.degradation_vs(&old);
        assert_eq!(degradations.len(), 2);
        assert!(degradations[0].contains("Boot time increased"));
        assert!(degradations[1].contains("Idle RAM usage increased"));
    }
}
