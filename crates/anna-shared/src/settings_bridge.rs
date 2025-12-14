// v0.0.629: Settings Bridge (Phase 205)
// Bridge layer for connecting different settings systems

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Bridge direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BridgeDirection {
    /// Unidirectional - source to target only
    #[default]
    Unidirectional,
    /// Bidirectional - both ways
    Bidirectional,
    /// Source only - read from source
    SourceOnly,
    /// Target only - write to target
    TargetOnly,
}

impl std::fmt::Display for BridgeDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unidirectional => write!(f, "unidirectional"),
            Self::Bidirectional => write!(f, "bidirectional"),
            Self::SourceOnly => write!(f, "source_only"),
            Self::TargetOnly => write!(f, "target_only"),
        }
    }
}

/// Bridge state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BridgeState {
    /// Idle
    #[default]
    Idle,
    /// Syncing
    Syncing,
    /// Active
    Active,
    /// Paused
    Paused,
    /// Error
    Error,
}

impl std::fmt::Display for BridgeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Syncing => write!(f, "syncing"),
            Self::Active => write!(f, "active"),
            Self::Paused => write!(f, "paused"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Bridge endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeEndpoint {
    /// Name
    pub name: String,
    /// Endpoint type
    pub endpoint_type: String,
    /// Connected
    pub connected: bool,
}

impl BridgeEndpoint {
    /// Create new endpoint
    pub fn new(name: impl Into<String>, endpoint_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            endpoint_type: endpoint_type.into(),
            connected: false,
        }
    }

    /// Mark connected
    pub fn connect(&mut self) {
        self.connected = true;
    }

    /// Mark disconnected
    pub fn disconnect(&mut self) {
        self.connected = false;
    }
}

/// Bridge mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMapping {
    /// Source key
    pub source_key: String,
    /// Target key
    pub target_key: String,
    /// Transform (optional)
    pub transform: Option<String>,
    /// Enabled
    pub enabled: bool,
}

impl BridgeMapping {
    /// Create new mapping
    pub fn new(source_key: impl Into<String>, target_key: impl Into<String>) -> Self {
        Self {
            source_key: source_key.into(),
            target_key: target_key.into(),
            transform: None,
            enabled: true,
        }
    }

    /// Set transform
    pub fn transform(mut self, transform: impl Into<String>) -> Self {
        self.transform = Some(transform.into());
        self
    }

    /// Set enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BridgeStats {
    /// Sync count
    pub sync_count: usize,
    /// Transfer count
    pub transfer_count: usize,
    /// Error count
    pub error_count: usize,
    /// Last sync timestamp
    pub last_sync: u64,
}

impl BridgeStats {
    /// Record sync
    pub fn record_sync(&mut self, timestamp: u64) {
        self.sync_count += 1;
        self.last_sync = timestamp;
    }

    /// Record transfer
    pub fn record_transfer(&mut self) {
        self.transfer_count += 1;
    }

    /// Record error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }
}

/// Settings bridge
#[derive(Debug, Clone, Default)]
pub struct SettingsBridge {
    /// Direction
    direction: BridgeDirection,
    /// State
    state: BridgeState,
    /// Source endpoint
    source: Option<BridgeEndpoint>,
    /// Target endpoint
    target: Option<BridgeEndpoint>,
    /// Mappings
    mappings: Vec<BridgeMapping>,
    /// Statistics
    stats: BridgeStats,
}

impl SettingsBridge {
    /// Create new bridge
    pub fn new() -> Self {
        Self::default()
    }

    /// Get direction
    pub fn direction(&self) -> BridgeDirection {
        self.direction
    }

    /// Set direction
    pub fn set_direction(&mut self, direction: BridgeDirection) {
        self.direction = direction;
    }

    /// Get state
    pub fn state(&self) -> BridgeState {
        self.state
    }

    /// Set source
    pub fn set_source(&mut self, endpoint: BridgeEndpoint) {
        self.source = Some(endpoint);
    }

    /// Set target
    pub fn set_target(&mut self, endpoint: BridgeEndpoint) {
        self.target = Some(endpoint);
    }

    /// Add mapping
    pub fn add_mapping(&mut self, mapping: BridgeMapping) {
        self.mappings.push(mapping);
    }

    /// Start bridge
    pub fn start(&mut self) {
        if self.source.as_ref().map(|s| s.connected).unwrap_or(false)
            && self.target.as_ref().map(|t| t.connected).unwrap_or(false)
        {
            self.state = BridgeState::Active;
        }
    }

    /// Pause bridge
    pub fn pause(&mut self) {
        self.state = BridgeState::Paused;
    }

    /// Stop bridge
    pub fn stop(&mut self) {
        self.state = BridgeState::Idle;
    }

    /// Sync
    pub fn sync(&mut self, timestamp: u64) {
        if self.state == BridgeState::Active {
            self.state = BridgeState::Syncing;
            self.stats.record_sync(timestamp);
            self.state = BridgeState::Active;
        }
    }

    /// Mapping count
    pub fn mapping_count(&self) -> usize {
        self.mappings.len()
    }

    /// Get stats
    pub fn stats(&self) -> &BridgeStats {
        &self.stats
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.state == BridgeState::Active
    }
}

/// Format bridge
pub fn format_bridge(bridge: &SettingsBridge) -> String {
    let mut output = String::new();
    output.push_str("Settings Bridge:\n");
    output.push_str(&format!("  Direction: {}\n", bridge.direction()));
    output.push_str(&format!("  State: {}\n", bridge.state()));
    output.push_str(&format!("  Mappings: {}\n", bridge.mapping_count()));
    output.push_str(&format!("  Syncs: {}\n", bridge.stats().sync_count));
    output
}

/// Check if query is about bridge
pub fn is_bridge_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("bridge")
        || lower.contains("settings bridge")
        || lower.contains("connect settings")
}

/// Fun fact about bridge
pub fn bridge_fun_fact() -> &'static str {
    "Anna's settings bridge connects different settings systems seamlessly!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_display() {
        assert_eq!(format!("{}", BridgeDirection::Unidirectional), "unidirectional");
        assert_eq!(format!("{}", BridgeDirection::Bidirectional), "bidirectional");
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", BridgeState::Active), "active");
        assert_eq!(format!("{}", BridgeState::Paused), "paused");
    }

    #[test]
    fn test_endpoint_new() {
        let e = BridgeEndpoint::new("src", "file");
        assert!(!e.connected);
    }

    #[test]
    fn test_endpoint_connect() {
        let mut e = BridgeEndpoint::new("src", "file");
        e.connect();
        assert!(e.connected);
    }

    #[test]
    fn test_mapping_new() {
        let m = BridgeMapping::new("src.key", "tgt.key");
        assert!(m.enabled);
    }

    #[test]
    fn test_mapping_transform() {
        let m = BridgeMapping::new("src.key", "tgt.key")
            .transform("uppercase");
        assert!(m.transform.is_some());
    }

    #[test]
    fn test_stats_record() {
        let mut s = BridgeStats::default();
        s.record_sync(100);
        assert_eq!(s.sync_count, 1);
    }

    #[test]
    fn test_bridge_new() {
        let b = SettingsBridge::new();
        assert!(!b.is_active());
    }

    #[test]
    fn test_bridge_add_mapping() {
        let mut b = SettingsBridge::new();
        b.add_mapping(BridgeMapping::new("src", "tgt"));
        assert_eq!(b.mapping_count(), 1);
    }

    #[test]
    fn test_bridge_start() {
        let mut b = SettingsBridge::new();
        let mut src = BridgeEndpoint::new("src", "file");
        let mut tgt = BridgeEndpoint::new("tgt", "file");
        src.connect();
        tgt.connect();
        b.set_source(src);
        b.set_target(tgt);
        b.start();
        assert!(b.is_active());
    }

    #[test]
    fn test_is_bridge_query() {
        assert!(is_bridge_query("settings bridge"));
        assert!(!is_bridge_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = bridge_fun_fact();
        assert!(fact.contains("bridge"));
    }
}
