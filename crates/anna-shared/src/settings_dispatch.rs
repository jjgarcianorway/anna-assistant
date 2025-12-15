// v0.0.714: Settings Dispatch (Phase 290)
// Dispatching settings changes to targets

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dispatch type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DispatchType {
    /// Immediate dispatch
    #[default]
    Immediate,
    /// Scheduled dispatch
    Scheduled,
    /// Batch dispatch
    Batch,
    /// Conditional dispatch
    Conditional,
}

impl std::fmt::Display for DispatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate => write!(f, "immediate"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Batch => write!(f, "batch"),
            Self::Conditional => write!(f, "conditional"),
        }
    }
}

/// Dispatch status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DispatchStatus {
    /// Pending
    #[default]
    Pending,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

impl std::fmt::Display for DispatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Dispatch config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchConfig {
    /// Name
    pub name: String,
    /// Dispatch type
    pub dispatch_type: DispatchType,
    /// Retry count
    pub retry_count: usize,
    /// Max dispatches
    pub max_dispatches: usize,
}

impl DispatchConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dispatch_type: DispatchType::Immediate,
            retry_count: 3,
            max_dispatches: 500,
        }
    }

    /// Set type
    pub fn dispatch_type(mut self, dt: DispatchType) -> Self {
        self.dispatch_type = dt;
        self
    }

    /// Set retry count
    pub fn retry_count(mut self, rc: usize) -> Self {
        self.retry_count = rc;
        self
    }

    /// Set max dispatches
    pub fn max_dispatches(mut self, max: usize) -> Self {
        self.max_dispatches = max;
        self
    }
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Dispatch item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchItem {
    /// Item ID
    pub id: String,
    /// Target
    pub target: String,
    /// Payload
    pub payload: String,
    /// Status
    pub status: DispatchStatus,
    /// Attempts
    pub attempts: usize,
}

impl DispatchItem {
    /// Create new item
    pub fn new(id: impl Into<String>, target: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            payload: payload.into(),
            status: DispatchStatus::Pending,
            attempts: 0,
        }
    }

    /// Start dispatch
    pub fn start(&mut self) {
        self.status = DispatchStatus::InProgress;
        self.attempts += 1;
    }

    /// Complete dispatch
    pub fn complete(&mut self) {
        self.status = DispatchStatus::Completed;
    }

    /// Mark failed
    pub fn fail(&mut self) {
        self.status = DispatchStatus::Failed;
    }
}

/// Dispatch metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchMetadata {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Item ID
    pub item_id: String,
}

impl DispatchMetadata {
    /// Create new metadata
    pub fn new(key: impl Into<String>, value: impl Into<String>, item_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            item_id: item_id.into(),
        }
    }
}

/// Dispatch stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DispatchStats {
    /// Total dispatches
    pub total_dispatches: usize,
    /// Completed dispatches
    pub completed: usize,
    /// Failed dispatches
    pub failed: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DispatchStats {
    /// Update from items
    pub fn update(&mut self, items: &[DispatchItem], dispatch_type: DispatchType) {
        self.total_dispatches = items.len();
        self.completed = items.iter().filter(|i| i.status == DispatchStatus::Completed).count();
        self.failed = items.iter().filter(|i| i.status == DispatchStatus::Failed).count();
        *self.by_type.entry(dispatch_type.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_dispatches == 0 { 0.0 } else { self.completed as f64 / self.total_dispatches as f64 * 100.0 }
    }
}

/// Settings dispatch
#[derive(Debug, Clone, Default)]
pub struct SettingsDispatch {
    /// Config
    config: DispatchConfig,
    /// Items
    items: Vec<DispatchItem>,
    /// Metadata
    metadata: Vec<DispatchMetadata>,
    /// Stats
    stats: DispatchStats,
}

impl SettingsDispatch {
    /// Create new dispatch system
    pub fn new(config: DispatchConfig) -> Self {
        Self {
            config,
            items: Vec::new(),
            metadata: Vec::new(),
            stats: DispatchStats::default(),
        }
    }

    /// Add item
    pub fn add_item(&mut self, item: DispatchItem) -> bool {
        if self.items.len() >= self.config.max_dispatches {
            return false;
        }
        self.items.push(item);
        self.update_stats();
        true
    }

    /// Get item
    pub fn get_item(&self, id: &str) -> Option<&DispatchItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Get item mut
    pub fn get_item_mut(&mut self, id: &str) -> Option<&mut DispatchItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Add metadata
    pub fn add_metadata(&mut self, meta: DispatchMetadata) {
        self.metadata.push(meta);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.items, self.config.dispatch_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DispatchStats {
        &self.stats
    }

    /// Item count
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

/// Dispatch registry
#[derive(Debug, Clone, Default)]
pub struct DispatchRegistry {
    /// Dispatches by ID
    dispatches: HashMap<String, SettingsDispatch>,
}

impl DispatchRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register dispatch
    pub fn register(&mut self, id: impl Into<String>, dispatch: SettingsDispatch) {
        self.dispatches.insert(id.into(), dispatch);
    }

    /// Unregister dispatch
    pub fn unregister(&mut self, id: &str) -> bool {
        self.dispatches.remove(id).is_some()
    }

    /// Get dispatch
    pub fn get(&self, id: &str) -> Option<&SettingsDispatch> {
        self.dispatches.get(id)
    }

    /// Get dispatch mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDispatch> {
        self.dispatches.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.dispatches.len()
    }
}

/// Format dispatch registry
pub fn format_dispatch_registry(registry: &DispatchRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Dispatch Registry:\n");
    output.push_str(&format!("  Dispatches: {}\n", registry.count()));
    output
}

/// Check if query is about dispatch
pub fn is_dispatch_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings dispatch") || lower.contains("dispatch settings") || lower.contains("send settings")
}

/// Fun fact about dispatch
pub fn dispatch_fun_fact() -> &'static str {
    "Anna's settings dispatch delivers configuration changes to their targets reliably!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_type_display() {
        assert_eq!(format!("{}", DispatchType::Immediate), "immediate");
        assert_eq!(format!("{}", DispatchType::Batch), "batch");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", DispatchStatus::Pending), "pending");
        assert_eq!(format!("{}", DispatchStatus::Completed), "completed");
    }

    #[test]
    fn test_config_new() {
        let c = DispatchConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DispatchConfig::new("test")
            .dispatch_type(DispatchType::Scheduled)
            .retry_count(5);
        assert_eq!(c.dispatch_type, DispatchType::Scheduled);
        assert_eq!(c.retry_count, 5);
    }

    #[test]
    fn test_item_new() {
        let i = DispatchItem::new("i1", "target", "payload");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_item_lifecycle() {
        let mut i = DispatchItem::new("i1", "target", "payload");
        i.start();
        assert_eq!(i.status, DispatchStatus::InProgress);
        assert_eq!(i.attempts, 1);
        i.complete();
        assert_eq!(i.status, DispatchStatus::Completed);
    }

    #[test]
    fn test_item_fail() {
        let mut i = DispatchItem::new("i1", "target", "payload");
        i.fail();
        assert_eq!(i.status, DispatchStatus::Failed);
    }

    #[test]
    fn test_metadata_new() {
        let m = DispatchMetadata::new("key", "value", "i1");
        assert_eq!(m.item_id, "i1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DispatchStats::default();
        let mut item = DispatchItem::new("i1", "target", "payload");
        item.complete();
        s.update(&[item], DispatchType::Immediate);
        assert_eq!(s.total_dispatches, 1);
        assert_eq!(s.completed, 1);
    }

    #[test]
    fn test_dispatch_new() {
        let d = SettingsDispatch::new(DispatchConfig::default());
        assert_eq!(d.item_count(), 0);
    }

    #[test]
    fn test_dispatch_add_item() {
        let mut d = SettingsDispatch::new(DispatchConfig::default());
        d.add_item(DispatchItem::new("i1", "target", "payload"));
        assert_eq!(d.item_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DispatchRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DispatchRegistry::new();
        r.register("d1", SettingsDispatch::new(DispatchConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_dispatch_query() {
        assert!(is_dispatch_query("settings dispatch"));
        assert!(!is_dispatch_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = dispatch_fun_fact();
        assert!(fact.contains("dispatch"));
    }
}
