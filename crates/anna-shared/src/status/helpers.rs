//! Helper tools, backup status, and learning status.

use serde::{Deserialize, Serialize};

use crate::deps;

/// v0.3.24: Backup status for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupStatus {
    /// Backup directory path
    pub directory: String,
    /// Number of backups stored
    pub backup_count: usize,
    /// Last backup timestamp (RFC3339 or "none")
    pub last_backup: Option<String>,
    /// Total size of all backups in bytes
    pub total_size_bytes: u64,
    /// Retention policy description
    pub retention_policy: String,
}

/// v0.3.27: Skill learning status for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningStatus {
    /// Whether learning mode is enabled
    pub enabled: bool,
    /// Skills in candidate tier (sandbox only)
    pub candidate_skills: usize,
    /// Skills in probation tier (host with verification)
    pub probation_skills: usize,
    /// Skills in trusted tier (normal use)
    pub trusted_skills: usize,
    /// Total promotions
    pub promotions: usize,
    /// Total demotions
    pub demotions: usize,
    /// Failed experiments (negative knowledge)
    pub failed_experiments: usize,
}

/// v0.3.21: Helper tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperInfo {
    /// Tool/command name
    pub name: String,
    /// Description
    pub description: String,
    /// Whether the tool is installed
    pub installed: bool,
    /// Installation source
    pub source: HelperSource,
}

/// v0.3.21: Where a helper was installed from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HelperSource {
    #[default]
    Unknown,
    /// Installed by user before Anna
    User,
    /// Installed by Anna
    Anna,
    /// System package (pre-installed)
    System,
}

impl std::fmt::Display for HelperSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelperSource::Unknown => write!(f, "unknown"),
            HelperSource::User => write!(f, "user"),
            HelperSource::Anna => write!(f, "anna"),
            HelperSource::System => write!(f, "system"),
        }
    }
}

impl HelperInfo {
    /// Check all diagnostic tools and their sources
    pub fn check_all() -> Vec<Self> {
        let anna_installed = deps::read_installed_packages().unwrap_or_default();

        deps::DIAGNOSTIC_TOOLS
            .iter()
            .map(|(name, desc)| {
                let installed = deps::command_exists(name);
                let source = if anna_installed.contains(&name.to_string()) {
                    HelperSource::Anna
                } else if installed {
                    // Was installed before Anna tracked it
                    HelperSource::User
                } else {
                    HelperSource::Unknown
                };

                Self {
                    name: name.to_string(),
                    description: desc.to_string(),
                    installed,
                    source,
                }
            })
            .collect()
    }
}
