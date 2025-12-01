//! Service State v5.2.0 - Full Systemd Service Tracking
//!
//! Track complete service state including:
//! - Active state (running, stopped, failed)
//! - Enabled state (enabled, disabled, masked)
//! - Recent failures
//! - Resource usage
//! - Dependencies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Constants
// ============================================================================

/// Service state store path
pub const SERVICE_STATE_PATH: &str = "/var/lib/anna/knowledge/services_v5.json";

// ============================================================================
// Service Active State
// ============================================================================

/// Systemd active state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActiveState {
    Active,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Reloading,
    Unknown,
}

impl ActiveState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActiveState::Active => "active",
            ActiveState::Inactive => "inactive",
            ActiveState::Failed => "failed",
            ActiveState::Activating => "activating",
            ActiveState::Deactivating => "deactivating",
            ActiveState::Reloading => "reloading",
            ActiveState::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => ActiveState::Active,
            "inactive" => ActiveState::Inactive,
            "failed" => ActiveState::Failed,
            "activating" => ActiveState::Activating,
            "deactivating" => ActiveState::Deactivating,
            "reloading" => ActiveState::Reloading,
            _ => ActiveState::Unknown,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, ActiveState::Active | ActiveState::Reloading)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, ActiveState::Failed)
    }

    /// Color code for terminal display
    pub fn color_code(&self) -> &'static str {
        match self {
            ActiveState::Active => "green",
            ActiveState::Inactive => "dim",
            ActiveState::Failed => "red",
            ActiveState::Activating | ActiveState::Deactivating => "yellow",
            ActiveState::Reloading => "cyan",
            ActiveState::Unknown => "dim",
        }
    }
}

// ============================================================================
// Service Enabled State
// ============================================================================

/// Systemd unit file state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnabledState {
    Enabled,
    Disabled,
    Masked,
    Static,
    Generated,
    Alias,
    Indirect,
    Unknown,
}

impl EnabledState {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnabledState::Enabled => "enabled",
            EnabledState::Disabled => "disabled",
            EnabledState::Masked => "masked",
            EnabledState::Static => "static",
            EnabledState::Generated => "generated",
            EnabledState::Alias => "alias",
            EnabledState::Indirect => "indirect",
            EnabledState::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "enabled" | "enabled-runtime" => EnabledState::Enabled,
            "disabled" => EnabledState::Disabled,
            "masked" | "masked-runtime" => EnabledState::Masked,
            "static" => EnabledState::Static,
            "generated" => EnabledState::Generated,
            "alias" => EnabledState::Alias,
            "indirect" => EnabledState::Indirect,
            _ => EnabledState::Unknown,
        }
    }

    pub fn is_masked(&self) -> bool {
        matches!(self, EnabledState::Masked)
    }
}

// ============================================================================
// Service Sub State
// ============================================================================

/// Systemd sub-state (more detailed than active state)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubState {
    Running,
    Dead,
    Failed,
    Exited,
    Waiting,
    Start,
    Stop,
    Reload,
    AutoRestart,
    Unknown,
}

impl SubState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubState::Running => "running",
            SubState::Dead => "dead",
            SubState::Failed => "failed",
            SubState::Exited => "exited",
            SubState::Waiting => "waiting",
            SubState::Start => "start",
            SubState::Stop => "stop",
            SubState::Reload => "reload",
            SubState::AutoRestart => "auto-restart",
            SubState::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" => SubState::Running,
            "dead" => SubState::Dead,
            "failed" => SubState::Failed,
            "exited" => SubState::Exited,
            "waiting" => SubState::Waiting,
            "start" | "start-pre" | "start-post" => SubState::Start,
            "stop" | "stop-pre" | "stop-post" | "stop-sigterm" | "stop-sigkill" => SubState::Stop,
            "reload" => SubState::Reload,
            "auto-restart" => SubState::AutoRestart,
            _ => SubState::Unknown,
        }
    }
}

// ============================================================================
// Service State (full state for a single service)
// ============================================================================

/// Complete state for a systemd service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    /// Unit name (e.g., "nginx.service")
    pub unit_name: String,

    /// Active state
    pub active_state: ActiveState,

    /// Enabled state
    pub enabled_state: EnabledState,

    /// Sub-state (running, exited, etc.)
    pub sub_state: SubState,

    /// Main PID (if running)
    pub main_pid: Option<u32>,

    /// Memory usage (bytes)
    pub memory_bytes: Option<u64>,

    /// CPU time (microseconds)
    pub cpu_usec: Option<u64>,

    /// Number of restarts
    pub restart_count: u32,

    /// Last state change timestamp
    pub state_change_at: Option<u64>,

    /// Last failure timestamp
    pub last_failure_at: Option<u64>,

    /// Failure reason (if failed)
    pub failure_reason: Option<String>,

    /// Description from unit file
    pub description: Option<String>,

    /// Requires dependencies
    pub requires: Vec<String>,

    /// After dependencies
    pub after: Vec<String>,

    /// First indexed timestamp
    pub first_indexed_at: u64,

    /// Last indexed timestamp
    pub last_indexed_at: u64,
}

impl ServiceState {
    pub fn new(unit_name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            unit_name: unit_name.to_string(),
            active_state: ActiveState::Unknown,
            enabled_state: EnabledState::Unknown,
            sub_state: SubState::Unknown,
            main_pid: None,
            memory_bytes: None,
            cpu_usec: None,
            restart_count: 0,
            state_change_at: None,
            last_failure_at: None,
            failure_reason: None,
            description: None,
            requires: Vec::new(),
            after: Vec::new(),
            first_indexed_at: now,
            last_indexed_at: now,
        }
    }

    /// Query service state from systemctl
    pub fn query(unit_name: &str) -> Option<Self> {
        let mut state = Self::new(unit_name);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Query properties
        let output = Command::new("systemctl")
            .args([
                "show",
                unit_name,
                "--property=ActiveState,SubState,UnitFileState,MainPID,MemoryCurrent,CPUUsageNSec,NRestarts,StateChangeTimestamp,Description,Requires,After",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "ActiveState" => state.active_state = ActiveState::from_str(value),
                    "SubState" => state.sub_state = SubState::from_str(value),
                    "UnitFileState" => state.enabled_state = EnabledState::from_str(value),
                    "MainPID" => state.main_pid = value.parse().ok().filter(|&p| p > 0),
                    "MemoryCurrent" => {
                        if value != "[not set]" {
                            state.memory_bytes = value.parse().ok();
                        }
                    }
                    "CPUUsageNSec" => {
                        if value != "[not set]" {
                            state.cpu_usec = value.parse::<u64>().ok().map(|n| n / 1000);
                        }
                    }
                    "NRestarts" => {
                        state.restart_count = value.parse().unwrap_or(0);
                    }
                    "StateChangeTimestamp" => {
                        // Parse systemd timestamp format
                        if !value.is_empty() && value != "n/a" {
                            state.state_change_at = Some(now); // Simplified
                        }
                    }
                    "Description" => {
                        if !value.is_empty() {
                            state.description = Some(value.to_string());
                        }
                    }
                    "Requires" => {
                        if !value.is_empty() {
                            state.requires =
                                value.split_whitespace().map(String::from).collect();
                        }
                    }
                    "After" => {
                        if !value.is_empty() {
                            state.after = value.split_whitespace().map(String::from).collect();
                        }
                    }
                    _ => {}
                }
            }
        }

        // Track failure
        if state.active_state.is_failed() {
            state.last_failure_at = Some(now);

            // Get failure reason
            if let Ok(result) = Command::new("systemctl")
                .args(["status", unit_name])
                .output()
            {
                let status_output = String::from_utf8_lossy(&result.stdout);
                for line in status_output.lines() {
                    if line.contains("Result:") || line.contains("Status:") {
                        state.failure_reason = Some(line.trim().to_string());
                        break;
                    }
                }
            }
        }

        state.last_indexed_at = now;
        Some(state)
    }

    /// Format memory for display
    pub fn format_memory(&self) -> String {
        match self.memory_bytes {
            Some(bytes) if bytes >= 1_073_741_824 => {
                format!("{:.1} GiB", bytes as f64 / 1_073_741_824.0)
            }
            Some(bytes) if bytes >= 1_048_576 => {
                format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
            }
            Some(bytes) if bytes >= 1024 => format!("{:.1} KiB", bytes as f64 / 1024.0),
            Some(bytes) => format!("{} B", bytes),
            None => "-".to_string(),
        }
    }

    /// Format CPU time for display
    pub fn format_cpu(&self) -> String {
        match self.cpu_usec {
            Some(usec) if usec >= 1_000_000 => format!("{:.2}s", usec as f64 / 1_000_000.0),
            Some(usec) if usec >= 1_000 => format!("{:.2}ms", usec as f64 / 1_000.0),
            Some(usec) => format!("{}us", usec),
            None => "-".to_string(),
        }
    }

    /// Get status summary string
    pub fn status_summary(&self) -> String {
        format!(
            "{} ({}) [{}]",
            self.active_state.as_str(),
            self.sub_state.as_str(),
            self.enabled_state.as_str()
        )
    }
}

// ============================================================================
// Service Index (global service database)
// ============================================================================

/// Global service state index
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceIndex {
    /// Services by unit name
    pub services: HashMap<String, ServiceState>,

    /// Count of running services
    pub running_count: usize,

    /// Count of failed services
    pub failed_count: usize,

    /// Count of masked services
    pub masked_count: usize,

    /// Last scan timestamp
    pub last_scan_at: u64,

    /// Created at timestamp
    pub created_at: u64,
}

impl ServiceIndex {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            services: HashMap::new(),
            running_count: 0,
            failed_count: 0,
            masked_count: 0,
            last_scan_at: now,
            created_at: now,
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(SERVICE_STATE_PATH) {
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(SERVICE_STATE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(SERVICE_STATE_PATH, json)
    }

    /// Update a service state
    pub fn update(&mut self, state: ServiceState) {
        self.services.insert(state.unit_name.clone(), state);
        self.recalculate_counts();
    }

    /// Query and update a specific service
    pub fn query_and_update(&mut self, unit_name: &str) -> Option<&ServiceState> {
        if let Some(state) = ServiceState::query(unit_name) {
            self.services.insert(unit_name.to_string(), state);
            self.recalculate_counts();
        }
        self.services.get(unit_name)
    }

    /// Scan all services and update states
    pub fn scan_all(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Get list of all services
        let output = Command::new("systemctl")
            .args(["list-units", "--type=service", "--all", "--no-legend", "--no-pager"])
            .output();

        if let Ok(result) = output {
            let stdout = String::from_utf8_lossy(&result.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let unit_name = parts[0];
                    if let Some(state) = ServiceState::query(unit_name) {
                        self.services.insert(unit_name.to_string(), state);
                    }
                }
            }
        }

        self.recalculate_counts();
        self.last_scan_at = now;
    }

    /// Recalculate summary counts
    fn recalculate_counts(&mut self) {
        self.running_count = self
            .services
            .values()
            .filter(|s| s.active_state.is_running())
            .count();
        self.failed_count = self
            .services
            .values()
            .filter(|s| s.active_state.is_failed())
            .count();
        self.masked_count = self
            .services
            .values()
            .filter(|s| s.enabled_state.is_masked())
            .count();
    }

    /// Get failed services
    pub fn get_failed(&self) -> Vec<&ServiceState> {
        self.services
            .values()
            .filter(|s| s.active_state.is_failed())
            .collect()
    }

    /// Get running services
    pub fn get_running(&self) -> Vec<&ServiceState> {
        self.services
            .values()
            .filter(|s| s.active_state.is_running())
            .collect()
    }

    /// Get masked services
    pub fn get_masked(&self) -> Vec<&ServiceState> {
        self.services
            .values()
            .filter(|s| s.enabled_state.is_masked())
            .collect()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.services.clear();
        self.running_count = 0;
        self.failed_count = 0;
        self.masked_count = 0;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_scan_at = now;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_state_parsing() {
        assert_eq!(ActiveState::from_str("active"), ActiveState::Active);
        assert_eq!(ActiveState::from_str("failed"), ActiveState::Failed);
        assert_eq!(ActiveState::from_str("inactive"), ActiveState::Inactive);
        assert_eq!(ActiveState::from_str("ACTIVE"), ActiveState::Active);
    }

    #[test]
    fn test_enabled_state_parsing() {
        assert_eq!(EnabledState::from_str("enabled"), EnabledState::Enabled);
        assert_eq!(EnabledState::from_str("disabled"), EnabledState::Disabled);
        assert_eq!(EnabledState::from_str("masked"), EnabledState::Masked);
        assert_eq!(EnabledState::from_str("static"), EnabledState::Static);
    }

    #[test]
    fn test_sub_state_parsing() {
        assert_eq!(SubState::from_str("running"), SubState::Running);
        assert_eq!(SubState::from_str("dead"), SubState::Dead);
        assert_eq!(SubState::from_str("failed"), SubState::Failed);
        assert_eq!(SubState::from_str("exited"), SubState::Exited);
    }

    #[test]
    fn test_service_state_new() {
        let state = ServiceState::new("test.service");
        assert_eq!(state.unit_name, "test.service");
        assert_eq!(state.active_state, ActiveState::Unknown);
        assert_eq!(state.enabled_state, EnabledState::Unknown);
    }

    #[test]
    fn test_format_memory() {
        let mut state = ServiceState::new("test.service");

        state.memory_bytes = Some(1024);
        assert_eq!(state.format_memory(), "1.0 KiB");

        state.memory_bytes = Some(1_048_576);
        assert_eq!(state.format_memory(), "1.0 MiB");

        state.memory_bytes = Some(1_073_741_824);
        assert_eq!(state.format_memory(), "1.0 GiB");
    }

    #[test]
    fn test_service_index() {
        let mut index = ServiceIndex::new();

        let mut state1 = ServiceState::new("running.service");
        state1.active_state = ActiveState::Active;

        let mut state2 = ServiceState::new("failed.service");
        state2.active_state = ActiveState::Failed;

        index.update(state1);
        index.update(state2);

        assert_eq!(index.running_count, 1);
        assert_eq!(index.failed_count, 1);
        assert_eq!(index.services.len(), 2);
    }

    #[test]
    fn test_status_summary() {
        let mut state = ServiceState::new("test.service");
        state.active_state = ActiveState::Active;
        state.sub_state = SubState::Running;
        state.enabled_state = EnabledState::Enabled;

        assert_eq!(state.status_summary(), "active (running) [enabled]");
    }
}
