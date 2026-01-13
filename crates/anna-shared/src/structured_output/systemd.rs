//! Systemd structured output - parse systemctl JSON output.

use super::ParseResult;
use serde::{Deserialize, Serialize};

/// Service info from `systemctl show --output=json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Unit name
    #[serde(rename = "Id")]
    pub id: Option<String>,
    /// Unit description
    #[serde(rename = "Description")]
    pub description: Option<String>,
    /// Load state
    #[serde(rename = "LoadState")]
    pub load_state: Option<String>,
    /// Active state
    #[serde(rename = "ActiveState")]
    pub active_state: Option<String>,
    /// Sub state
    #[serde(rename = "SubState")]
    pub sub_state: Option<String>,
    /// Unit file state (enabled, disabled, etc.)
    #[serde(rename = "UnitFileState")]
    pub unit_file_state: Option<String>,
    /// Main process PID
    #[serde(rename = "MainPID")]
    pub main_pid: Option<u32>,
    /// Exec start command
    #[serde(rename = "ExecStart")]
    pub exec_start: Option<String>,
    /// Restart policy
    #[serde(rename = "Restart")]
    pub restart: Option<String>,
    /// Memory current usage
    #[serde(rename = "MemoryCurrent")]
    pub memory_current: Option<u64>,
    /// CPU usage (nanoseconds)
    #[serde(rename = "CPUUsageNSec")]
    pub cpu_usage_nsec: Option<u64>,
    /// Timestamp of last state change
    #[serde(rename = "StateChangeTimestamp")]
    pub state_change_timestamp: Option<String>,
    /// Wants (dependencies)
    #[serde(rename = "Wants")]
    pub wants: Option<String>,
    /// Required by
    #[serde(rename = "RequiredBy")]
    pub required_by: Option<String>,
    /// Wanted by
    #[serde(rename = "WantedBy")]
    pub wanted_by: Option<String>,
}

impl ServiceInfo {
    /// Get the service state
    pub fn state(&self) -> ServiceState {
        match self.active_state.as_deref() {
            Some("active") => ServiceState::Active,
            Some("inactive") => ServiceState::Inactive,
            Some("failed") => ServiceState::Failed,
            Some("activating") => ServiceState::Activating,
            Some("deactivating") => ServiceState::Deactivating,
            _ => ServiceState::Unknown,
        }
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        matches!(self.state(), ServiceState::Active)
    }

    /// Check if service is enabled
    pub fn is_enabled(&self) -> bool {
        matches!(
            self.unit_file_state.as_deref(),
            Some("enabled") | Some("enabled-runtime")
        )
    }

    /// Check if service has failed
    pub fn is_failed(&self) -> bool {
        matches!(self.state(), ServiceState::Failed)
    }

    /// Get memory usage in MB
    pub fn memory_mb(&self) -> Option<f64> {
        self.memory_current.map(|b| b as f64 / (1024.0 * 1024.0))
    }

    /// Get dependencies as a list
    pub fn dependencies(&self) -> Vec<&str> {
        self.wants
            .as_deref()
            .map(|s| s.split_whitespace().collect())
            .unwrap_or_default()
    }
}

/// Service state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Unknown,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceState::Active => write!(f, "active"),
            ServiceState::Inactive => write!(f, "inactive"),
            ServiceState::Failed => write!(f, "failed"),
            ServiceState::Activating => write!(f, "activating"),
            ServiceState::Deactivating => write!(f, "deactivating"),
            ServiceState::Unknown => write!(f, "unknown"),
        }
    }
}

/// Parse `systemctl show --output=json <unit>` output
pub fn parse_systemctl_output(output: &str) -> ParseResult<ServiceInfo> {
    super::parse_json(output)
}

/// Unit list entry from `systemctl list-units --output=json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitListEntry {
    /// Unit name
    pub unit: String,
    /// Load state
    pub load: String,
    /// Active state
    pub active: String,
    /// Sub state
    pub sub: String,
    /// Description
    pub description: String,
}

/// Parse `systemctl list-units --output=json` output
pub fn parse_list_units_output(output: &str) -> ParseResult<Vec<UnitListEntry>> {
    super::parse_json(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_service_info() {
        let json = r#"{
            "Id": "sshd.service",
            "Description": "OpenSSH Daemon",
            "LoadState": "loaded",
            "ActiveState": "active",
            "SubState": "running",
            "UnitFileState": "enabled",
            "MainPID": 1234,
            "MemoryCurrent": 10485760
        }"#;

        let result = parse_systemctl_output(json);
        assert!(result.is_ok());

        let info = result.ok().unwrap();
        assert_eq!(info.id, Some("sshd.service".to_string()));
        assert!(info.is_running());
        assert!(info.is_enabled());
        assert_eq!(info.memory_mb(), Some(10.0));
    }

    #[test]
    fn test_service_state() {
        let info = ServiceInfo {
            id: Some("test.service".to_string()),
            description: None,
            load_state: Some("loaded".to_string()),
            active_state: Some("failed".to_string()),
            sub_state: None,
            unit_file_state: Some("disabled".to_string()),
            main_pid: None,
            exec_start: None,
            restart: None,
            memory_current: None,
            cpu_usage_nsec: None,
            state_change_timestamp: None,
            wants: None,
            required_by: None,
            wanted_by: None,
        };

        assert!(info.is_failed());
        assert!(!info.is_enabled());
        assert!(!info.is_running());
    }
}
