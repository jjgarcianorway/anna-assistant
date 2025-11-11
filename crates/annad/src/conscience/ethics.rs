//! Ethical evaluation engine for conscience layer
//!
//! Phase 1.1: Evaluates actions against safety, privacy, integrity, autonomy
//! Citation: [archwiki:System_maintenance]

use super::types::{EthicalScore, EthicsConfig, ReasoningNode, ReasoningTree};
use crate::sentinel::SentinelAction;
use anyhow::Result;
use tracing::{debug, warn};

/// Ethical evaluation engine
pub struct EthicsEngine {
    config: EthicsConfig,
}

impl EthicsEngine {
    /// Create new ethics engine with configuration
    pub fn new(config: EthicsConfig) -> Self {
        Self { config }
    }

    /// Evaluate action against ethical dimensions
    pub fn evaluate(&self, action: &SentinelAction) -> (EthicalScore, ReasoningTree) {
        debug!("Evaluating action: {:?}", action);

        let safety = self.evaluate_safety(action);
        let privacy = self.evaluate_privacy(action);
        let integrity = self.evaluate_integrity(action);
        let autonomy = self.evaluate_autonomy(action);

        let score = EthicalScore {
            safety: safety.0,
            privacy: privacy.0,
            integrity: integrity.0,
            autonomy: autonomy.0,
        };

        // Build reasoning tree from individual evaluations
        let reasoning = self.build_reasoning_tree(action, safety, privacy, integrity, autonomy);

        (score, reasoning)
    }

    /// Check if action passes ethical threshold
    pub fn is_ethical(&self, score: &EthicalScore) -> bool {
        score.is_ethical(self.config.ethical_threshold)
    }

    /// Evaluate safety dimension
    fn evaluate_safety(&self, action: &SentinelAction) -> (f64, String, Vec<String>) {
        match action {
            SentinelAction::None => (1.0, "No action taken".to_string(), vec![]),

            SentinelAction::RestartService { service } => (
                0.8,
                format!("Service restart is generally safe: {}", service),
                vec![
                    "Systemd will handle restart gracefully".to_string(),
                    "Service state is preserved".to_string(),
                    "Citation: [archwiki:Systemd#Using_units]".to_string(),
                ],
            ),

            SentinelAction::SyncDatabases => (
                0.95,
                "Database sync is safe read-only operation".to_string(),
                vec![
                    "pacman -Sy only updates package metadata".to_string(),
                    "No packages are installed or removed".to_string(),
                    "Citation: [archwiki:Pacman#Upgrading_packages]".to_string(),
                ],
            ),

            SentinelAction::SystemUpdate { dry_run } => {
                if *dry_run {
                    (
                        1.0,
                        "Dry-run update is completely safe".to_string(),
                        vec![
                            "No changes are made to the system".to_string(),
                            "Only simulation of update process".to_string(),
                        ],
                    )
                } else {
                    (
                        0.6,
                        "System update has moderate risk".to_string(),
                        vec![
                            "Package updates may introduce regressions".to_string(),
                            "Kernel updates require reboot".to_string(),
                            "Pre-transaction hooks mitigate risk".to_string(),
                            "Citation: [archwiki:System_maintenance#Upgrading_the_system]".to_string(),
                        ],
                    )
                }
            }

            SentinelAction::RunRepair { probe } => (
                0.7,
                format!("Repair probe execution has controlled risk: {}", probe),
                vec![
                    "Repair actions are wiki-validated".to_string(),
                    "Each probe has defined scope".to_string(),
                    "Changes are logged for audit".to_string(),
                ],
            ),

            SentinelAction::LogWarning { .. } => (
                1.0,
                "Logging is safe and non-destructive".to_string(),
                vec!["Only writes to log files".to_string()],
            ),

            SentinelAction::SendNotification { .. } => (
                1.0,
                "Notifications are safe and informational".to_string(),
                vec!["User notification via desktop environment".to_string()],
            ),
        }
    }

    /// Evaluate privacy dimension
    fn evaluate_privacy(&self, action: &SentinelAction) -> (f64, String, Vec<String>) {
        match action {
            SentinelAction::None => (1.0, "No privacy impact".to_string(), vec![]),

            SentinelAction::RestartService { .. }
            | SentinelAction::SyncDatabases
            | SentinelAction::RunRepair { .. } => (
                1.0,
                "System operations respect user data boundaries".to_string(),
                vec![
                    "No access to /home directories".to_string(),
                    "No reading of user files".to_string(),
                    "Operations limited to system scope".to_string(),
                ],
            ),

            SentinelAction::SystemUpdate { .. } => (
                1.0,
                "Package updates do not access user data".to_string(),
                vec![
                    "pacman operates on /usr, /etc, /var only".to_string(),
                    "Citation: [archwiki:Pacman#Usage]".to_string(),
                ],
            ),

            SentinelAction::LogWarning { message } => {
                // Check if message contains sensitive paths
                if message.contains("/home") || message.contains("password") {
                    (
                        0.5,
                        "Warning message may contain sensitive information".to_string(),
                        vec!["Manual review recommended for privacy".to_string()],
                    )
                } else {
                    (1.0, "Log message contains no sensitive data".to_string(), vec![])
                }
            }

            SentinelAction::SendNotification { title, body } => {
                // Check for sensitive information in notification
                let sensitive = title.contains("password")
                    || body.contains("password")
                    || title.contains("/home")
                    || body.contains("/home");

                if sensitive {
                    (
                        0.4,
                        "Notification may expose sensitive information".to_string(),
                        vec!["Contains potentially private paths or keywords".to_string()],
                    )
                } else {
                    (1.0, "Notification contains public information only".to_string(), vec![])
                }
            }
        }
    }

    /// Evaluate integrity dimension
    fn evaluate_integrity(&self, action: &SentinelAction) -> (f64, String, Vec<String>) {
        match action {
            SentinelAction::None => (1.0, "No integrity concerns".to_string(), vec![]),

            SentinelAction::RestartService { service } => (
                0.9,
                format!("Service restart is transparent and logged: {}", service),
                vec![
                    "Systemd journal records all service changes".to_string(),
                    "Action is observable by user via journalctl".to_string(),
                ],
            ),

            SentinelAction::SyncDatabases => (
                0.95,
                "Database sync is transparent operation".to_string(),
                vec![
                    "pacman logs all database operations".to_string(),
                    "User can verify sync via pacman -Qu".to_string(),
                ],
            ),

            SentinelAction::SystemUpdate { dry_run } => {
                if *dry_run {
                    (1.0, "Dry-run is honest simulation".to_string(), vec![])
                } else {
                    (
                        0.85,
                        "System update is transparent and auditable".to_string(),
                        vec![
                            "All package changes logged to pacman.log".to_string(),
                            "Pre/post transaction hooks visible".to_string(),
                            "Citation: [archwiki:Pacman#Viewing_logs]".to_string(),
                        ],
                    )
                }
            }

            SentinelAction::RunRepair { .. } => (
                0.8,
                "Repair actions are logged and wiki-cited".to_string(),
                vec![
                    "Each action includes Arch Wiki citation".to_string(),
                    "Changes recorded in audit trail".to_string(),
                ],
            ),

            SentinelAction::LogWarning { .. } | SentinelAction::SendNotification { .. } => {
                (1.0, "Informational actions are inherently transparent".to_string(), vec![])
            }
        }
    }

    /// Evaluate autonomy dimension
    fn evaluate_autonomy(&self, action: &SentinelAction) -> (f64, String, Vec<String>) {
        match action {
            SentinelAction::None => (1.0, "No impact on user control".to_string(), vec![]),

            SentinelAction::RestartService { .. } => (
                0.7,
                "Service restart is autonomous but reversible".to_string(),
                vec![
                    "User can stop/start service manually anytime".to_string(),
                    "Service state is under user control".to_string(),
                ],
            ),

            SentinelAction::SyncDatabases => (
                0.9,
                "Database sync preserves user control".to_string(),
                vec![
                    "User can still choose which packages to update".to_string(),
                    "No packages installed without consent".to_string(),
                ],
            ),

            SentinelAction::SystemUpdate { dry_run } => {
                if *dry_run {
                    (1.0, "Dry-run preserves full user control".to_string(), vec![])
                } else {
                    (
                        0.5,
                        "Auto-update reduces user control".to_string(),
                        vec![
                            "Packages updated without explicit consent".to_string(),
                            "Should require configuration opt-in".to_string(),
                            "User can disable auto_update in config".to_string(),
                        ],
                    )
                }
            }

            SentinelAction::RunRepair { .. } => (
                0.6,
                "Automated repair reduces user agency".to_string(),
                vec![
                    "Repairs applied without explicit request".to_string(),
                    "User can disable auto_repair_services in config".to_string(),
                ],
            ),

            SentinelAction::LogWarning { .. } | SentinelAction::SendNotification { .. } => (
                1.0,
                "Informational actions preserve full autonomy".to_string(),
                vec!["User retains complete control over system".to_string()],
            ),
        }
    }

    /// Build reasoning tree from dimension evaluations
    fn build_reasoning_tree(
        &self,
        action: &SentinelAction,
        safety: (f64, String, Vec<String>),
        privacy: (f64, String, Vec<String>),
        integrity: (f64, String, Vec<String>),
        autonomy: (f64, String, Vec<String>),
    ) -> ReasoningTree {
        let action_desc = format!("{:?}", action);

        let root = ReasoningNode {
            statement: format!("Ethical evaluation for action: {}", action_desc),
            evidence: vec![],
            confidence: 1.0,
            children: vec![
                ReasoningNode {
                    statement: format!("Safety: {}", safety.1),
                    evidence: safety.2,
                    confidence: safety.0,
                    children: vec![],
                },
                ReasoningNode {
                    statement: format!("Privacy: {}", privacy.1),
                    evidence: privacy.2,
                    confidence: privacy.0,
                    children: vec![],
                },
                ReasoningNode {
                    statement: format!("Integrity: {}", integrity.1),
                    evidence: integrity.2,
                    confidence: integrity.0,
                    children: vec![],
                },
                ReasoningNode {
                    statement: format!("Autonomy: {}", autonomy.1),
                    evidence: autonomy.2,
                    confidence: autonomy.0,
                    children: vec![],
                },
            ],
        };

        ReasoningTree { root }
    }

    /// Load ethics configuration from file
    pub async fn load_config() -> Result<EthicsConfig> {
        let config_path = "/etc/anna/ethics.yml";

        match tokio::fs::read_to_string(config_path).await {
            Ok(content) => {
                let config: EthicsConfig = serde_yaml::from_str(&content)?;
                debug!("Loaded ethics config from {}", config_path);
                Ok(config)
            }
            Err(_) => {
                warn!(
                    "Ethics config not found at {}, using defaults",
                    config_path
                );
                Ok(EthicsConfig::default())
            }
        }
    }

    /// Save ethics configuration to file
    pub async fn save_config(config: &EthicsConfig) -> Result<()> {
        let config_path = "/etc/anna/ethics.yml";

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(config_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let yaml = serde_yaml::to_string(config)?;
        tokio::fs::write(config_path, yaml).await?;

        debug!("Saved ethics config to {}", config_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_evaluation() {
        let engine = EthicsEngine::new(EthicsConfig::default());

        // Test safe actions
        let (score, _) = engine.evaluate(&SentinelAction::None);
        assert_eq!(score.safety, 1.0);

        let (score, _) = engine.evaluate(&SentinelAction::SyncDatabases);
        assert!(score.safety >= 0.9);

        // Test moderate risk actions
        let (score, _) = engine.evaluate(&SentinelAction::SystemUpdate { dry_run: false });
        assert!(score.safety < 0.9);
        assert!(score.safety >= 0.5);
    }

    #[test]
    fn test_privacy_evaluation() {
        let engine = EthicsEngine::new(EthicsConfig::default());

        // Test privacy-safe actions
        let (score, _) = engine.evaluate(&SentinelAction::SyncDatabases);
        assert_eq!(score.privacy, 1.0);

        // Test potentially sensitive notification
        let (score, _) = engine.evaluate(&SentinelAction::SendNotification {
            title: "Password required".to_string(),
            body: "Enter password".to_string(),
        });
        assert!(score.privacy < 1.0);
    }

    #[test]
    fn test_autonomy_evaluation() {
        let engine = EthicsEngine::new(EthicsConfig::default());

        // Dry-run preserves autonomy
        let (score, _) = engine.evaluate(&SentinelAction::SystemUpdate { dry_run: true });
        assert_eq!(score.autonomy, 1.0);

        // Auto-update reduces autonomy
        let (score, _) = engine.evaluate(&SentinelAction::SystemUpdate { dry_run: false });
        assert!(score.autonomy < 1.0);
    }

    #[test]
    fn test_ethical_threshold() {
        let engine = EthicsEngine::new(EthicsConfig::default());

        let high_score = EthicalScore {
            safety: 0.9,
            privacy: 0.9,
            integrity: 0.9,
            autonomy: 0.9,
        };
        assert!(engine.is_ethical(&high_score));

        let low_score = EthicalScore {
            safety: 0.3,
            privacy: 0.9,
            integrity: 0.9,
            autonomy: 0.9,
        };
        assert!(!engine.is_ethical(&low_score));
    }
}
