//! Sentinel state persistence and caching
//!
//! Phase 1.0: State management with /var/lib/anna/state.json
//! Citation: [archwiki:System_maintenance]

use super::types::{SentinelConfig, SentinelState};
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info};

const STATE_DIR: &str = "/var/lib/anna";
const STATE_FILE: &str = "state.json";
const CONFIG_FILE: &str = "config.json";

/// Load sentinel state from disk
pub async fn load_state() -> Result<SentinelState> {
    let state_path = Path::new(STATE_DIR).join(STATE_FILE);

    if !state_path.exists() {
        info!("No existing state found, creating default");
        return Ok(SentinelState::default());
    }

    let mut file = tokio::fs::File::open(&state_path)
        .await
        .context("Failed to open state file")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .await
        .context("Failed to read state file")?;

    let state: SentinelState =
        serde_json::from_str(&contents).context("Failed to parse state JSON")?;

    info!("Loaded sentinel state version {}", state.version);
    Ok(state)
}

/// Save sentinel state to disk
pub async fn save_state(state: &SentinelState) -> Result<()> {
    let state_dir = Path::new(STATE_DIR);
    create_dir_all(state_dir)
        .await
        .context("Failed to create state directory")?;

    let state_path = state_dir.join(STATE_FILE);
    let state_json = serde_json::to_string_pretty(state).context("Failed to serialize state")?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&state_path)
        .await
        .context("Failed to open state file for writing")?;

    file.write_all(state_json.as_bytes())
        .await
        .context("Failed to write state file")?;

    file.sync_all().await.context("Failed to sync state file")?;

    // Beta.95: Changed to debug! to reduce log spam (saves happen every minute)
    debug!("Saved sentinel state version {}", state.version);
    Ok(())
}

/// Load sentinel configuration from disk
pub async fn load_config() -> Result<SentinelConfig> {
    let config_path = Path::new(STATE_DIR).join(CONFIG_FILE);

    if !config_path.exists() {
        info!("No existing configuration found, creating default");
        let default_config = SentinelConfig::default();
        save_config(&default_config).await?;
        return Ok(default_config);
    }

    let mut file = tokio::fs::File::open(&config_path)
        .await
        .context("Failed to open config file")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .await
        .context("Failed to read config file")?;

    let config: SentinelConfig =
        serde_json::from_str(&contents).context("Failed to parse config JSON")?;

    info!(
        "Loaded sentinel configuration (autonomous_mode={})",
        config.autonomous_mode
    );
    Ok(config)
}

/// Save sentinel configuration to disk
pub async fn save_config(config: &SentinelConfig) -> Result<()> {
    let state_dir = Path::new(STATE_DIR);
    create_dir_all(state_dir)
        .await
        .context("Failed to create state directory")?;

    let config_path = state_dir.join(CONFIG_FILE);
    let config_json = serde_json::to_string_pretty(config).context("Failed to serialize config")?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&config_path)
        .await
        .context("Failed to open config file for writing")?;

    file.write_all(config_json.as_bytes())
        .await
        .context("Failed to write config file")?;

    file.sync_all()
        .await
        .context("Failed to sync config file")?;

    info!("Saved sentinel configuration");
    Ok(())
}

/// Calculate state diff between two states
pub fn calculate_diff(old: &SentinelState, new: &SentinelState) -> StateDiff {
    StateDiff {
        version_delta: new.version.saturating_sub(old.version),
        system_state_changed: old.system_state != new.system_state,
        health_status_changed: old.last_health.status != new.last_health.status,
        failed_services_delta: new.last_health.failed_services as i32
            - old.last_health.failed_services as i32,
        available_updates_delta: new.last_health.available_updates as i32
            - old.last_health.available_updates as i32,
        log_issues_delta: new.last_health.log_issues as i32 - old.last_health.log_issues as i32,
        error_rate_delta: new.error_rate - old.error_rate,
        drift_index_delta: new.drift_index - old.drift_index,
    }
}

/// State diff result
#[derive(Debug, Clone)]
pub struct StateDiff {
    pub version_delta: u64,
    pub system_state_changed: bool,
    pub health_status_changed: bool,
    pub failed_services_delta: i32,
    pub available_updates_delta: i32,
    pub log_issues_delta: i32,
    pub error_rate_delta: f64,
    pub drift_index_delta: f64,
}

impl StateDiff {
    /// Check if diff indicates degradation
    pub fn is_degradation(&self) -> bool {
        self.failed_services_delta > 0
            || self.log_issues_delta > 0
            || self.error_rate_delta > 0.1
            || self.drift_index_delta > 0.1
    }

    /// Check if diff indicates improvement
    pub fn is_improvement(&self) -> bool {
        self.failed_services_delta < 0
            || self.log_issues_delta < 0
            || self.error_rate_delta < -0.1
            || self.drift_index_delta < -0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_degradation() {
        let mut old = SentinelState::default();
        old.last_health.failed_services = 0;
        old.error_rate = 0.0;

        let mut new = old.clone();
        new.last_health.failed_services = 2;
        new.error_rate = 0.5;

        let diff = calculate_diff(&old, &new);
        assert!(diff.is_degradation());
        assert!(!diff.is_improvement());
    }

    #[test]
    fn test_diff_improvement() {
        let mut old = SentinelState::default();
        old.last_health.failed_services = 2;
        old.error_rate = 0.5;

        let mut new = old.clone();
        new.last_health.failed_services = 0;
        new.error_rate = 0.0;

        let diff = calculate_diff(&old, &new);
        assert!(diff.is_improvement());
        assert!(!diff.is_degradation());
    }
}
