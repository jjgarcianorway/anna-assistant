// v0.0.628: Settings Adapter (Phase 204)
// Adapter layer for integrating different settings sources

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Adapter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AdapterType {
    /// File adapter
    #[default]
    File,
    /// Environment adapter
    Environment,
    /// Memory adapter
    Memory,
    /// Remote adapter
    Remote,
    /// Database adapter
    Database,
}

impl std::fmt::Display for AdapterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Environment => write!(f, "environment"),
            Self::Memory => write!(f, "memory"),
            Self::Remote => write!(f, "remote"),
            Self::Database => write!(f, "database"),
        }
    }
}

/// Adapter status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AdapterStatus {
    /// Connected
    #[default]
    Connected,
    /// Disconnected
    Disconnected,
    /// Connecting
    Connecting,
    /// Error
    Error,
    /// Disabled
    Disabled,
}

impl std::fmt::Display for AdapterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connected => write!(f, "connected"),
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Error => write!(f, "error"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

/// Adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Adapter type
    pub adapter_type: AdapterType,
    /// Name
    pub name: String,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Read-only
    pub read_only: bool,
    /// Enabled
    pub enabled: bool,
}

impl AdapterConfig {
    /// Create new config
    pub fn new(adapter_type: AdapterType, name: impl Into<String>) -> Self {
        Self {
            adapter_type,
            name: name.into(),
            priority: 100,
            read_only: false,
            enabled: true,
        }
    }

    /// Set priority
    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Set enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Adapter instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInstance {
    /// Configuration
    pub config: AdapterConfig,
    /// Status
    pub status: AdapterStatus,
    /// Last sync timestamp
    pub last_sync: u64,
    /// Error message
    pub error: Option<String>,
}

impl AdapterInstance {
    /// Create new instance
    pub fn new(config: AdapterConfig) -> Self {
        Self {
            config,
            status: AdapterStatus::Disconnected,
            last_sync: 0,
            error: None,
        }
    }

    /// Connect
    pub fn connect(&mut self) {
        self.status = AdapterStatus::Connected;
        self.error = None;
    }

    /// Disconnect
    pub fn disconnect(&mut self) {
        self.status = AdapterStatus::Disconnected;
    }

    /// Set error
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.status = AdapterStatus::Error;
        self.error = Some(error.into());
    }

    /// Mark synced
    pub fn mark_synced(&mut self, timestamp: u64) {
        self.last_sync = timestamp;
    }

    /// Is available
    pub fn is_available(&self) -> bool {
        self.config.enabled && self.status == AdapterStatus::Connected
    }
}

/// Adapter statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdapterStats {
    /// Read count
    pub read_count: usize,
    /// Write count
    pub write_count: usize,
    /// Error count
    pub error_count: usize,
    /// Sync count
    pub sync_count: usize,
}

impl AdapterStats {
    /// Record read
    pub fn record_read(&mut self) {
        self.read_count += 1;
    }

    /// Record write
    pub fn record_write(&mut self) {
        self.write_count += 1;
    }

    /// Record error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Record sync
    pub fn record_sync(&mut self) {
        self.sync_count += 1;
    }
}

/// Settings adapter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsAdapterRegistry {
    /// Adapters by name
    adapters: HashMap<String, AdapterInstance>,
    /// Statistics by adapter
    stats: HashMap<String, AdapterStats>,
}

impl SettingsAdapterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register adapter
    pub fn register(&mut self, config: AdapterConfig) {
        let name = config.name.clone();
        self.adapters.insert(name.clone(), AdapterInstance::new(config));
        self.stats.insert(name, AdapterStats::default());
    }

    /// Unregister adapter
    pub fn unregister(&mut self, name: &str) -> bool {
        self.stats.remove(name);
        self.adapters.remove(name).is_some()
    }

    /// Get adapter
    pub fn get(&self, name: &str) -> Option<&AdapterInstance> {
        self.adapters.get(name)
    }

    /// Get adapter mut
    pub fn get_mut(&mut self, name: &str) -> Option<&mut AdapterInstance> {
        self.adapters.get_mut(name)
    }

    /// List available adapters
    pub fn list_available(&self) -> Vec<&AdapterInstance> {
        self.adapters.values().filter(|a| a.is_available()).collect()
    }

    /// Get stats
    pub fn get_stats(&self, name: &str) -> Option<&AdapterStats> {
        self.stats.get(name)
    }

    /// Record operation
    pub fn record_read(&mut self, name: &str) {
        if let Some(stats) = self.stats.get_mut(name) {
            stats.record_read();
        }
    }

    /// Adapter count
    pub fn count(&self) -> usize {
        self.adapters.len()
    }

    /// Available count
    pub fn available_count(&self) -> usize {
        self.adapters.values().filter(|a| a.is_available()).count()
    }
}

/// Format adapter registry
pub fn format_adapter_registry(registry: &SettingsAdapterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Adapter Registry:\n");
    output.push_str(&format!("  Adapters: {}\n", registry.count()));
    output.push_str(&format!("  Available: {}\n", registry.available_count()));
    output
}

/// Check if query is about adapter
pub fn is_adapter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("adapter")
        || lower.contains("settings adapter")
        || lower.contains("source adapter")
}

/// Fun fact about adapter
pub fn adapter_fun_fact() -> &'static str {
    "Anna's settings adapters allow integrating settings from multiple sources!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_type_display() {
        assert_eq!(format!("{}", AdapterType::File), "file");
        assert_eq!(format!("{}", AdapterType::Memory), "memory");
    }

    #[test]
    fn test_adapter_status_display() {
        assert_eq!(format!("{}", AdapterStatus::Connected), "connected");
        assert_eq!(format!("{}", AdapterStatus::Error), "error");
    }

    #[test]
    fn test_config_new() {
        let c = AdapterConfig::new(AdapterType::File, "file1");
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = AdapterConfig::new(AdapterType::File, "file1")
            .priority(50)
            .read_only(true);
        assert!(c.read_only);
    }

    #[test]
    fn test_instance_new() {
        let c = AdapterConfig::new(AdapterType::File, "file1");
        let i = AdapterInstance::new(c);
        assert!(!i.is_available());
    }

    #[test]
    fn test_instance_connect() {
        let c = AdapterConfig::new(AdapterType::File, "file1");
        let mut i = AdapterInstance::new(c);
        i.connect();
        assert!(i.is_available());
    }

    #[test]
    fn test_stats_record() {
        let mut s = AdapterStats::default();
        s.record_read();
        s.record_write();
        assert_eq!(s.read_count, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsAdapterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsAdapterRegistry::new();
        r.register(AdapterConfig::new(AdapterType::File, "file1"));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_registry_get() {
        let mut r = SettingsAdapterRegistry::new();
        r.register(AdapterConfig::new(AdapterType::File, "file1"));
        assert!(r.get("file1").is_some());
    }

    #[test]
    fn test_is_adapter_query() {
        assert!(is_adapter_query("settings adapter"));
        assert!(!is_adapter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = adapter_fun_fact();
        assert!(fact.contains("adapter"));
    }
}
