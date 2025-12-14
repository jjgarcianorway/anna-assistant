// v0.0.631: Settings Provider (Phase 207)
// Provider abstraction for settings sources

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProviderType {
    /// Local provider
    #[default]
    Local,
    /// Remote provider
    Remote,
    /// Cloud provider
    Cloud,
    /// Hybrid provider
    Hybrid,
    /// Custom provider
    Custom,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Remote => write!(f, "remote"),
            Self::Cloud => write!(f, "cloud"),
            Self::Hybrid => write!(f, "hybrid"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Provider capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderCapability {
    /// Read capability
    Read,
    /// Write capability
    Write,
    /// Delete capability
    Delete,
    /// List capability
    List,
    /// Watch capability
    Watch,
    /// Sync capability
    Sync,
}

impl std::fmt::Display for ProviderCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Delete => write!(f, "delete"),
            Self::List => write!(f, "list"),
            Self::Watch => write!(f, "watch"),
            Self::Sync => write!(f, "sync"),
        }
    }
}

/// Provider info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Name
    pub name: String,
    /// Provider type
    pub provider_type: ProviderType,
    /// Capabilities
    pub capabilities: Vec<ProviderCapability>,
    /// Priority
    pub priority: u32,
    /// Enabled
    pub enabled: bool,
}

impl ProviderInfo {
    /// Create new provider info
    pub fn new(name: impl Into<String>, provider_type: ProviderType) -> Self {
        Self {
            name: name.into(),
            provider_type,
            capabilities: Vec::new(),
            priority: 100,
            enabled: true,
        }
    }

    /// Add capability
    pub fn capability(mut self, cap: ProviderCapability) -> Self {
        self.capabilities.push(cap);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Has capability
    pub fn has_capability(&self, cap: ProviderCapability) -> bool {
        self.capabilities.contains(&cap)
    }

    /// Can read
    pub fn can_read(&self) -> bool {
        self.has_capability(ProviderCapability::Read)
    }

    /// Can write
    pub fn can_write(&self) -> bool {
        self.has_capability(ProviderCapability::Write)
    }
}

/// Provider status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    /// Available
    pub available: bool,
    /// Healthy
    pub healthy: bool,
    /// Last check timestamp
    pub last_check: u64,
    /// Error message
    pub error: Option<String>,
}

impl ProviderStatus {
    /// Create new status
    pub fn new() -> Self {
        Self {
            available: false,
            healthy: false,
            last_check: 0,
            error: None,
        }
    }

    /// Mark available
    pub fn mark_available(&mut self, timestamp: u64) {
        self.available = true;
        self.healthy = true;
        self.last_check = timestamp;
        self.error = None;
    }

    /// Mark unavailable
    pub fn mark_unavailable(&mut self, timestamp: u64, error: impl Into<String>) {
        self.available = false;
        self.healthy = false;
        self.last_check = timestamp;
        self.error = Some(error.into());
    }
}

impl Default for ProviderStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderStats {
    /// Read count
    pub read_count: usize,
    /// Write count
    pub write_count: usize,
    /// Error count
    pub error_count: usize,
    /// Latency sum ms
    pub latency_sum_ms: u64,
}

impl ProviderStats {
    /// Record operation
    pub fn record(&mut self, is_read: bool, latency_ms: u64, success: bool) {
        if is_read {
            self.read_count += 1;
        } else {
            self.write_count += 1;
        }
        if !success {
            self.error_count += 1;
        }
        self.latency_sum_ms += latency_ms;
    }

    /// Average latency
    pub fn avg_latency_ms(&self) -> u64 {
        let total = self.read_count + self.write_count;
        if total == 0 {
            0
        } else {
            self.latency_sum_ms / total as u64
        }
    }
}

/// Settings provider registry
#[derive(Debug, Clone, Default)]
pub struct SettingsProviderRegistry {
    /// Providers by name
    providers: HashMap<String, ProviderInfo>,
    /// Status by name
    status: HashMap<String, ProviderStatus>,
    /// Statistics by name
    stats: HashMap<String, ProviderStats>,
}

impl SettingsProviderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register provider
    pub fn register(&mut self, info: ProviderInfo) {
        let name = info.name.clone();
        self.providers.insert(name.clone(), info);
        self.status.insert(name.clone(), ProviderStatus::new());
        self.stats.insert(name, ProviderStats::default());
    }

    /// Unregister provider
    pub fn unregister(&mut self, name: &str) -> bool {
        self.stats.remove(name);
        self.status.remove(name);
        self.providers.remove(name).is_some()
    }

    /// Get provider
    pub fn get(&self, name: &str) -> Option<&ProviderInfo> {
        self.providers.get(name)
    }

    /// Get status
    pub fn get_status(&self, name: &str) -> Option<&ProviderStatus> {
        self.status.get(name)
    }

    /// Update status
    pub fn update_status(&mut self, name: &str, available: bool, timestamp: u64) {
        if let Some(status) = self.status.get_mut(name) {
            if available {
                status.mark_available(timestamp);
            } else {
                status.mark_unavailable(timestamp, "Provider unavailable");
            }
        }
    }

    /// List by capability
    pub fn list_by_capability(&self, cap: ProviderCapability) -> Vec<&ProviderInfo> {
        self.providers.values().filter(|p| p.has_capability(cap)).collect()
    }

    /// Provider count
    pub fn count(&self) -> usize {
        self.providers.len()
    }

    /// Available count
    pub fn available_count(&self) -> usize {
        self.status.values().filter(|s| s.available).count()
    }
}

/// Format provider registry
pub fn format_provider_registry(registry: &SettingsProviderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Provider Registry:\n");
    output.push_str(&format!("  Providers: {}\n", registry.count()));
    output.push_str(&format!("  Available: {}\n", registry.available_count()));
    output
}

/// Check if query is about provider
pub fn is_provider_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("provider")
        || lower.contains("settings provider")
        || lower.contains("data provider")
}

/// Fun fact about provider
pub fn provider_fun_fact() -> &'static str {
    "Anna's settings providers abstract away the complexity of different data sources!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_display() {
        assert_eq!(format!("{}", ProviderType::Local), "local");
        assert_eq!(format!("{}", ProviderType::Cloud), "cloud");
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(format!("{}", ProviderCapability::Read), "read");
        assert_eq!(format!("{}", ProviderCapability::Write), "write");
    }

    #[test]
    fn test_info_new() {
        let i = ProviderInfo::new("p1", ProviderType::Local);
        assert!(i.enabled);
    }

    #[test]
    fn test_info_capability() {
        let i = ProviderInfo::new("p1", ProviderType::Local)
            .capability(ProviderCapability::Read);
        assert!(i.can_read());
    }

    #[test]
    fn test_status_new() {
        let s = ProviderStatus::new();
        assert!(!s.available);
    }

    #[test]
    fn test_status_available() {
        let mut s = ProviderStatus::new();
        s.mark_available(100);
        assert!(s.available);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ProviderStats::default();
        s.record(true, 50, true);
        assert_eq!(s.read_count, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsProviderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsProviderRegistry::new();
        r.register(ProviderInfo::new("p1", ProviderType::Local));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_registry_get() {
        let mut r = SettingsProviderRegistry::new();
        r.register(ProviderInfo::new("p1", ProviderType::Local));
        assert!(r.get("p1").is_some());
    }

    #[test]
    fn test_is_provider_query() {
        assert!(is_provider_query("settings provider"));
        assert!(!is_provider_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = provider_fun_fact();
        assert!(fact.contains("provider"));
    }
}
