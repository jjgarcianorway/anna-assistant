// v0.0.607: Settings Deployer (Phase 183)
// Deploy settings configurations to targets

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Deploy target type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployTarget {
    /// Local system
    Local,
    /// Remote system
    Remote,
    /// User profile
    Profile,
    /// Application
    Application,
    /// Service
    Service,
}

impl std::fmt::Display for DeployTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Remote => write!(f, "remote"),
            Self::Profile => write!(f, "profile"),
            Self::Application => write!(f, "application"),
            Self::Service => write!(f, "service"),
        }
    }
}

/// Deploy status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStatus {
    /// Pending
    Pending,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Rolled back
    RolledBack,
}

impl std::fmt::Display for DeployStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// Deploy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Target type
    pub target: DeployTarget,
    /// Target address
    pub address: String,
    /// Categories to deploy
    pub categories: Vec<SettingsCategory>,
    /// Dry run
    pub dry_run: bool,
    /// Backup before deploy
    pub backup: bool,
}

impl DeployConfig {
    /// Create new config
    pub fn new(id: impl Into<String>, target: DeployTarget) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            target,
            address: String::new(),
            categories: Vec::new(),
            dry_run: false,
            backup: true,
        }
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set address
    pub fn address(mut self, addr: impl Into<String>) -> Self {
        self.address = addr.into();
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set dry run
    pub fn dry_run(mut self, dry: bool) -> Self {
        self.dry_run = dry;
        self
    }

    /// Set backup
    pub fn backup(mut self, backup: bool) -> Self {
        self.backup = backup;
        self
    }
}

/// Deploy result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployResult {
    /// Config ID
    pub config_id: String,
    /// Status
    pub status: DeployStatus,
    /// Items deployed
    pub items_deployed: usize,
    /// Items failed
    pub items_failed: usize,
    /// Error message
    pub error: Option<String>,
    /// Duration ms
    pub duration_ms: u64,
}

impl DeployResult {
    /// Create success result
    pub fn success(config_id: impl Into<String>, items: usize) -> Self {
        Self {
            config_id: config_id.into(),
            status: DeployStatus::Completed,
            items_deployed: items,
            items_failed: 0,
            error: None,
            duration_ms: 0,
        }
    }

    /// Create failure result
    pub fn failure(config_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            config_id: config_id.into(),
            status: DeployStatus::Failed,
            items_deployed: 0,
            items_failed: 0,
            error: Some(error.into()),
            duration_ms: 0,
        }
    }

    /// Set duration
    pub fn duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Is success
    pub fn is_success(&self) -> bool {
        self.status == DeployStatus::Completed
    }
}

/// Deploy history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployHistory {
    /// Timestamp
    pub timestamp: u64,
    /// Config
    pub config: DeployConfig,
    /// Result
    pub result: DeployResult,
}

/// Settings deployer
#[derive(Debug, Clone, Default)]
pub struct SettingsDeployer {
    /// Configurations
    configs: HashMap<String, DeployConfig>,
    /// History
    history: Vec<DeployHistory>,
    /// Max history
    max_history: usize,
}

impl SettingsDeployer {
    /// Create new deployer
    pub fn new() -> Self {
        Self {
            max_history: 100,
            ..Default::default()
        }
    }

    /// Add config
    pub fn add_config(&mut self, config: DeployConfig) {
        self.configs.insert(config.id.clone(), config);
    }

    /// Remove config
    pub fn remove_config(&mut self, id: &str) -> Option<DeployConfig> {
        self.configs.remove(id)
    }

    /// Get config
    pub fn get_config(&self, id: &str) -> Option<&DeployConfig> {
        self.configs.get(id)
    }

    /// Record deployment
    pub fn record(&mut self, config: DeployConfig, result: DeployResult) {
        self.history.push(DeployHistory {
            timestamp: 0,
            config,
            result,
        });
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get history
    pub fn history(&self) -> &[DeployHistory] {
        &self.history
    }

    /// Config count
    pub fn config_count(&self) -> usize {
        self.configs.len()
    }

    /// History count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.history.is_empty() {
            1.0
        } else {
            let success = self.history.iter().filter(|h| h.result.is_success()).count();
            success as f64 / self.history.len() as f64
        }
    }
}

/// Format deployer
pub fn format_deployer(deployer: &SettingsDeployer) -> String {
    let mut output = String::new();
    output.push_str("Settings Deployer:\n");
    output.push_str(&format!("  Configs: {}\n", deployer.config_count()));
    output.push_str(&format!("  History: {}\n", deployer.history_count()));
    output.push_str(&format!("  Success rate: {:.1}%\n", deployer.success_rate() * 100.0));
    output
}

/// Check if query is about deployer
pub fn is_deployer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("deploy")
        || lower.contains("push settings")
        || lower.contains("apply settings")
}

/// Fun fact about deployer
pub fn deployer_fun_fact() -> &'static str {
    "Anna can deploy your settings to local or remote targets with rollback support!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_display() {
        assert_eq!(format!("{}", DeployTarget::Local), "local");
        assert_eq!(format!("{}", DeployTarget::Remote), "remote");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", DeployStatus::Completed), "completed");
        assert_eq!(format!("{}", DeployStatus::Failed), "failed");
    }

    #[test]
    fn test_config_new() {
        let c = DeployConfig::new("d1", DeployTarget::Local);
        assert!(c.backup);
    }

    #[test]
    fn test_config_builder() {
        let c = DeployConfig::new("d1", DeployTarget::Remote)
            .name("Test")
            .address("host:22")
            .dry_run(true);
        assert!(c.dry_run);
    }

    #[test]
    fn test_result_success() {
        let r = DeployResult::success("d1", 10);
        assert!(r.is_success());
    }

    #[test]
    fn test_result_failure() {
        let r = DeployResult::failure("d1", "connection failed");
        assert!(!r.is_success());
    }

    #[test]
    fn test_deployer_new() {
        let d = SettingsDeployer::new();
        assert_eq!(d.config_count(), 0);
    }

    #[test]
    fn test_deployer_add_config() {
        let mut d = SettingsDeployer::new();
        d.add_config(DeployConfig::new("d1", DeployTarget::Local));
        assert_eq!(d.config_count(), 1);
    }

    #[test]
    fn test_deployer_record() {
        let mut d = SettingsDeployer::new();
        let config = DeployConfig::new("d1", DeployTarget::Local);
        let result = DeployResult::success("d1", 5);
        d.record(config, result);
        assert_eq!(d.history_count(), 1);
    }

    #[test]
    fn test_is_deployer_query() {
        assert!(is_deployer_query("deploy settings"));
        assert!(!is_deployer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = deployer_fun_fact();
        assert!(fact.contains("deploy"));
    }
}
