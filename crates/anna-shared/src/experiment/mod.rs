//! Experiment Manager - Sandbox testing before live execution.
//!
//! Before mutating the system:
//! - Check if the command can be sandboxed (namespaces, containers, VMs)
//! - Predict side effects from package hooks, service restarts, etc.
//! - Score risk vs. information gain

pub mod sandbox;
pub mod predictor;
pub mod scoring;

pub use sandbox::*;
pub use predictor::*;
pub use scoring::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An experiment to be run before live execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// Unique experiment ID
    pub id: String,
    /// Commands to test
    pub commands: Vec<String>,
    /// Sandbox environment to use
    pub sandbox: SandboxType,
    /// Predicted side effects
    pub predictions: Vec<SideEffect>,
    /// Risk vs information gain score
    pub score: ExperimentScore,
    /// Experiment status
    pub status: ExperimentStatus,
    /// Results if completed
    pub results: Option<ExperimentResults>,
    /// When the experiment was created
    pub created_at: String,
}

/// Types of sandbox environments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SandboxType {
    #[default]
    /// No sandbox - run directly (for read-only commands)
    None,
    /// Dry-run mode (command supports --dry-run or similar)
    DryRun,
    /// Filesystem namespace isolation
    FilesystemNamespace,
    /// Full namespace isolation (mount, pid, net)
    FullNamespace,
    /// Bubblewrap sandbox
    Bubblewrap,
    /// Systemd-nspawn container
    NspawnContainer,
    /// Docker container
    Docker,
    /// Podman container
    Podman,
    /// Virtual machine
    VirtualMachine,
}

impl SandboxType {
    /// Check if this sandbox type is available on the system
    pub fn is_available(&self) -> bool {
        match self {
            SandboxType::None | SandboxType::DryRun => true,
            SandboxType::FilesystemNamespace | SandboxType::FullNamespace => {
                // Check for unshare capability
                std::process::Command::new("unshare")
                    .arg("--help")
                    .output()
                    .is_ok()
            }
            SandboxType::Bubblewrap => command_exists("bwrap"),
            SandboxType::NspawnContainer => command_exists("systemd-nspawn"),
            SandboxType::Docker => command_exists("docker"),
            SandboxType::Podman => command_exists("podman"),
            SandboxType::VirtualMachine => false, // Complex setup, disabled by default
        }
    }

    /// Get isolation level (0-10)
    pub fn isolation_level(&self) -> u8 {
        match self {
            SandboxType::None => 0,
            SandboxType::DryRun => 2,
            SandboxType::FilesystemNamespace => 4,
            SandboxType::FullNamespace => 6,
            SandboxType::Bubblewrap => 7,
            SandboxType::NspawnContainer => 8,
            SandboxType::Docker | SandboxType::Podman => 8,
            SandboxType::VirtualMachine => 10,
        }
    }

    /// Overhead level (0-10, higher = more overhead)
    pub fn overhead(&self) -> u8 {
        match self {
            SandboxType::None => 0,
            SandboxType::DryRun => 1,
            SandboxType::FilesystemNamespace => 2,
            SandboxType::FullNamespace => 3,
            SandboxType::Bubblewrap => 3,
            SandboxType::NspawnContainer => 5,
            SandboxType::Docker | SandboxType::Podman => 5,
            SandboxType::VirtualMachine => 10,
        }
    }
}

/// Experiment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Results of a completed experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    /// Exit codes for each command
    pub exit_codes: Vec<i32>,
    /// Stdout for each command
    pub stdout: Vec<String>,
    /// Stderr for each command
    pub stderr: Vec<String>,
    /// Files that would be modified
    pub modified_files: Vec<String>,
    /// Packages that would be installed/removed
    pub package_changes: Vec<PackageChange>,
    /// Services that would be affected
    pub service_changes: Vec<ServiceChange>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Was the experiment safe?
    pub safe: bool,
    /// Recommendation based on results
    pub recommendation: ExperimentRecommendation,
}

/// Package change from experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageChange {
    pub name: String,
    pub action: PackageAction,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageAction {
    Install,
    Remove,
    Upgrade,
    Downgrade,
}

/// Service change from experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceChange {
    pub name: String,
    pub action: ServiceAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
}

/// Recommendation based on experiment results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentRecommendation {
    /// Safe to proceed
    Proceed,
    /// Proceed with caution
    ProceedWithCaution,
    /// Review changes first
    ReviewFirst,
    /// Do not proceed
    DoNotProceed,
}

/// The experiment manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentManager {
    /// Active experiments
    pub experiments: HashMap<String, Experiment>,
    /// Completed experiments (limited history)
    pub history: Vec<Experiment>,
    /// Maximum history size
    pub max_history: usize,
    /// Default sandbox preference
    pub default_sandbox: SandboxType,
}

impl ExperimentManager {
    /// Create a new experiment manager
    pub fn new() -> Self {
        Self {
            experiments: HashMap::new(),
            history: Vec::new(),
            max_history: 100,
            default_sandbox: SandboxType::None,
        }
    }

    /// Create an experiment for commands
    pub fn create_experiment(&mut self, commands: &[String]) -> Experiment {
        let id = uuid::Uuid::new_v4().to_string();

        // Analyze commands to choose sandbox
        let sandbox = select_sandbox(commands);

        // Predict side effects
        let predictions = predict_side_effects(commands);

        // Calculate score
        let score = calculate_experiment_score(commands, &predictions, sandbox);

        let experiment = Experiment {
            id: id.clone(),
            commands: commands.to_vec(),
            sandbox,
            predictions,
            score,
            status: ExperimentStatus::Pending,
            results: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.experiments.insert(id, experiment.clone());
        experiment
    }

    /// Get an experiment by ID
    pub fn get(&self, id: &str) -> Option<&Experiment> {
        self.experiments.get(id)
    }

    /// Mark experiment as completed
    pub fn complete_experiment(&mut self, id: &str, results: ExperimentResults) {
        if let Some(exp) = self.experiments.get_mut(id) {
            exp.status = ExperimentStatus::Completed;
            exp.results = Some(results);

            // Move to history
            let exp = exp.clone();
            self.experiments.remove(id);
            self.history.push(exp);

            // Trim history
            while self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }
    }

    /// Cancel an experiment
    pub fn cancel_experiment(&mut self, id: &str) {
        if let Some(exp) = self.experiments.get_mut(id) {
            exp.status = ExperimentStatus::Cancelled;
        }
    }

    /// Get all pending experiments
    pub fn pending_experiments(&self) -> Vec<&Experiment> {
        self.experiments
            .values()
            .filter(|e| e.status == ExperimentStatus::Pending)
            .collect()
    }

    /// Check if we should experiment before running
    pub fn should_experiment(&self, commands: &[String]) -> bool {
        for cmd in commands {
            let risk = estimate_command_risk(cmd);
            if risk > 0.3 {
                return true;
            }
        }
        false
    }
}

/// Check if a command exists
fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

impl Experiment {
    /// Check if this experiment requires elevated privileges
    pub fn requires_root(&self) -> bool {
        for cmd in &self.commands {
            if cmd.starts_with("sudo ")
                || cmd.starts_with("pacman -S")
                || cmd.starts_with("pacman -R")
                || cmd.contains("systemctl")
            {
                return true;
            }
        }
        false
    }

    /// Get a summary of the experiment
    pub fn summary(&self) -> String {
        format!(
            "{} command(s), sandbox: {:?}, risk: {:.2}, gain: {:.2}",
            self.commands.len(),
            self.sandbox,
            self.score.risk_score,
            self.score.information_gain
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_availability() {
        // None and DryRun should always be available
        assert!(SandboxType::None.is_available());
        assert!(SandboxType::DryRun.is_available());
    }

    #[test]
    fn test_experiment_creation() {
        let mut manager = ExperimentManager::new();
        let exp = manager.create_experiment(&["ls -la".to_string()]);

        assert_eq!(exp.status, ExperimentStatus::Pending);
        assert!(manager.get(&exp.id).is_some());
    }

    #[test]
    fn test_isolation_levels() {
        assert!(SandboxType::VirtualMachine.isolation_level() > SandboxType::Docker.isolation_level());
        assert!(SandboxType::Docker.isolation_level() > SandboxType::None.isolation_level());
    }
}
