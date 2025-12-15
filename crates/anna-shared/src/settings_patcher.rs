// v0.0.662: Settings Patcher (Phase 238)
// Patcher for applying incremental changes to settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Patch operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PatchOperation {
    /// Add new key
    Add,
    /// Remove existing key
    Remove,
    /// Replace value
    #[default]
    Replace,
    /// Copy from another key
    Copy,
    /// Move to another key
    Move,
}

impl std::fmt::Display for PatchOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "add"),
            Self::Remove => write!(f, "remove"),
            Self::Replace => write!(f, "replace"),
            Self::Copy => write!(f, "copy"),
            Self::Move => write!(f, "move"),
        }
    }
}

/// Patch mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PatchMode {
    /// Strict (fail on errors)
    #[default]
    Strict,
    /// Lenient (skip errors)
    Lenient,
    /// Dry run
    DryRun,
    /// Atomic (all or nothing)
    Atomic,
}

impl std::fmt::Display for PatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Lenient => write!(f, "lenient"),
            Self::DryRun => write!(f, "dry_run"),
            Self::Atomic => write!(f, "atomic"),
        }
    }
}

/// Patcher config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatcherConfig {
    /// Patch mode
    pub mode: PatchMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Validate before apply
    pub validate_before: bool,
    /// Create backup before apply
    pub backup_before: bool,
}

impl PatcherConfig {
    /// Create new config
    pub fn new(mode: PatchMode) -> Self {
        Self {
            mode,
            category: None,
            validate_before: true,
            backup_before: true,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set validate before
    pub fn validate_before(mut self, validate: bool) -> Self {
        self.validate_before = validate;
        self
    }

    /// Set backup before
    pub fn backup_before(mut self, backup: bool) -> Self {
        self.backup_before = backup;
        self
    }
}

impl Default for PatcherConfig {
    fn default() -> Self {
        Self::new(PatchMode::Strict)
    }
}

/// Single patch entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchEntry {
    /// Operation
    pub operation: PatchOperation,
    /// Target key
    pub key: String,
    /// Value (for add/replace)
    pub value: Option<String>,
    /// Source key (for copy/move)
    pub source_key: Option<String>,
}

impl PatchEntry {
    /// Create add patch
    pub fn add(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Add,
            key: key.into(),
            value: Some(value.into()),
            source_key: None,
        }
    }

    /// Create remove patch
    pub fn remove(key: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Remove,
            key: key.into(),
            value: None,
            source_key: None,
        }
    }

    /// Create replace patch
    pub fn replace(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Replace,
            key: key.into(),
            value: Some(value.into()),
            source_key: None,
        }
    }

    /// Create copy patch
    pub fn copy(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Copy,
            key: to.into(),
            value: None,
            source_key: Some(from.into()),
        }
    }

    /// Create move patch
    pub fn move_key(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            operation: PatchOperation::Move,
            key: to.into(),
            value: None,
            source_key: Some(from.into()),
        }
    }
}

/// Patch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResult {
    /// Applied patches
    pub applied: Vec<String>,
    /// Skipped patches
    pub skipped: Vec<String>,
    /// Failed patches
    pub failed: Vec<String>,
    /// Patch mode used
    pub mode: PatchMode,
}

impl PatchResult {
    /// Create new result
    pub fn new(mode: PatchMode) -> Self {
        Self {
            applied: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
            mode,
        }
    }

    /// Add applied
    pub fn add_applied(&mut self, key: String) {
        self.applied.push(key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.skipped.push(key);
    }

    /// Add failed
    pub fn add_failed(&mut self, key: String) {
        self.failed.push(key);
    }

    /// Total applied
    pub fn total_applied(&self) -> usize {
        self.applied.len()
    }

    /// Has failures
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }

    /// Success
    pub fn success(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Patcher stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatcherStats {
    /// Total patches applied
    pub total_patches: usize,
    /// Total operations
    pub total_operations: usize,
    /// By operation
    pub by_operation: HashMap<String, usize>,
}

impl PatcherStats {
    /// Record patch
    pub fn record(&mut self, operation: PatchOperation, count: usize) {
        self.total_patches += 1;
        self.total_operations += count;
        *self.by_operation.entry(operation.to_string()).or_insert(0) += count;
    }

    /// Average operations per patch
    pub fn average_operations(&self) -> f64 {
        if self.total_patches == 0 {
            0.0
        } else {
            self.total_operations as f64 / self.total_patches as f64
        }
    }
}

/// Settings patcher
#[derive(Debug, Clone, Default)]
pub struct SettingsPatcher {
    /// Config
    config: PatcherConfig,
    /// Results
    results: Vec<PatchResult>,
    /// Stats
    stats: PatcherStats,
}

impl SettingsPatcher {
    /// Create new patcher
    pub fn new(config: PatcherConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: PatcherStats::default(),
        }
    }

    /// Apply patches to settings
    pub fn apply(
        &mut self,
        target: &mut HashMap<String, String>,
        patches: &[PatchEntry],
    ) -> PatchResult {
        let mut result = PatchResult::new(self.config.mode);
        let backup = target.clone();

        for patch in patches {
            let success = match patch.operation {
                PatchOperation::Add => {
                    if !target.contains_key(&patch.key) {
                        if let Some(value) = &patch.value {
                            if self.config.mode != PatchMode::DryRun {
                                target.insert(patch.key.clone(), value.clone());
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                PatchOperation::Remove => {
                    if target.contains_key(&patch.key) {
                        if self.config.mode != PatchMode::DryRun {
                            target.remove(&patch.key);
                        }
                        true
                    } else {
                        false
                    }
                }
                PatchOperation::Replace => {
                    if target.contains_key(&patch.key) {
                        if let Some(value) = &patch.value {
                            if self.config.mode != PatchMode::DryRun {
                                target.insert(patch.key.clone(), value.clone());
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                PatchOperation::Copy => {
                    if let Some(source_key) = &patch.source_key {
                        if let Some(value) = target.get(source_key).cloned() {
                            if self.config.mode != PatchMode::DryRun {
                                target.insert(patch.key.clone(), value);
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                PatchOperation::Move => {
                    if let Some(source_key) = &patch.source_key {
                        if let Some(value) = target.remove(source_key) {
                            if self.config.mode != PatchMode::DryRun {
                                target.insert(patch.key.clone(), value);
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            };

            if success {
                result.add_applied(patch.key.clone());
                self.stats.record(patch.operation, 1);
            } else {
                match self.config.mode {
                    PatchMode::Strict | PatchMode::Atomic => {
                        result.add_failed(patch.key.clone());
                    }
                    PatchMode::Lenient | PatchMode::DryRun => {
                        result.add_skipped(patch.key.clone());
                    }
                }
            }
        }

        // Rollback for atomic mode on failure
        if self.config.mode == PatchMode::Atomic && result.has_failures() {
            *target = backup;
        }

        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[PatchResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &PatcherStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

/// Settings patcher registry
#[derive(Debug, Clone, Default)]
pub struct SettingsPatcherRegistry {
    /// Patchers by ID
    patchers: HashMap<String, SettingsPatcher>,
}

impl SettingsPatcherRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register patcher
    pub fn register(&mut self, id: impl Into<String>, patcher: SettingsPatcher) {
        self.patchers.insert(id.into(), patcher);
    }

    /// Unregister patcher
    pub fn unregister(&mut self, id: &str) -> bool {
        self.patchers.remove(id).is_some()
    }

    /// Get patcher
    pub fn get(&self, id: &str) -> Option<&SettingsPatcher> {
        self.patchers.get(id)
    }

    /// Get patcher mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPatcher> {
        self.patchers.get_mut(id)
    }

    /// Patcher count
    pub fn count(&self) -> usize {
        self.patchers.len()
    }
}

/// Format patcher registry
pub fn format_patcher_registry(registry: &SettingsPatcherRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Patcher Registry:\n");
    output.push_str(&format!("  Patchers: {}\n", registry.count()));
    output
}

/// Check if query is about patcher
pub fn is_patcher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("patcher") || lower.contains("patch settings") || lower.contains("apply patch")
}

/// Fun fact about patcher
pub fn patcher_fun_fact() -> &'static str {
    "Anna's settings patchers apply changes incrementally!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_operation_display() {
        assert_eq!(format!("{}", PatchOperation::Add), "add");
        assert_eq!(format!("{}", PatchOperation::Remove), "remove");
    }

    #[test]
    fn test_patch_mode_display() {
        assert_eq!(format!("{}", PatchMode::Strict), "strict");
        assert_eq!(format!("{}", PatchMode::Lenient), "lenient");
    }

    #[test]
    fn test_config_new() {
        let c = PatcherConfig::new(PatchMode::Strict);
        assert!(c.validate_before);
    }

    #[test]
    fn test_config_builder() {
        let c = PatcherConfig::new(PatchMode::Atomic)
            .backup_before(false)
            .validate_before(false);
        assert!(!c.backup_before);
        assert!(!c.validate_before);
    }

    #[test]
    fn test_entry_add() {
        let e = PatchEntry::add("key", "value");
        assert_eq!(e.operation, PatchOperation::Add);
    }

    #[test]
    fn test_entry_remove() {
        let e = PatchEntry::remove("key");
        assert_eq!(e.operation, PatchOperation::Remove);
    }

    #[test]
    fn test_entry_replace() {
        let e = PatchEntry::replace("key", "new_value");
        assert_eq!(e.operation, PatchOperation::Replace);
    }

    #[test]
    fn test_result_new() {
        let r = PatchResult::new(PatchMode::Strict);
        assert_eq!(r.total_applied(), 0);
    }

    #[test]
    fn test_result_add_applied() {
        let mut r = PatchResult::new(PatchMode::Strict);
        r.add_applied("key".to_string());
        assert_eq!(r.total_applied(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = PatcherStats::default();
        s.record(PatchOperation::Add, 5);
        assert_eq!(s.total_patches, 1);
        assert_eq!(s.total_operations, 5);
    }

    #[test]
    fn test_patcher_new() {
        let p = SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict));
        assert_eq!(p.result_count(), 0);
    }

    #[test]
    fn test_patcher_apply_add() {
        let mut p = SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict));
        let mut target = HashMap::new();
        let patches = vec![PatchEntry::add("key1", "value1")];

        let r = p.apply(&mut target, &patches);
        assert_eq!(r.total_applied(), 1);
        assert_eq!(target.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_patcher_apply_remove() {
        let mut p = SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict));
        let mut target = HashMap::new();
        target.insert("key1".to_string(), "value1".to_string());
        let patches = vec![PatchEntry::remove("key1")];

        let r = p.apply(&mut target, &patches);
        assert_eq!(r.total_applied(), 1);
        assert!(!target.contains_key("key1"));
    }

    #[test]
    fn test_patcher_apply_replace() {
        let mut p = SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict));
        let mut target = HashMap::new();
        target.insert("key1".to_string(), "old".to_string());
        let patches = vec![PatchEntry::replace("key1", "new")];

        let r = p.apply(&mut target, &patches);
        assert_eq!(r.total_applied(), 1);
        assert_eq!(target.get("key1"), Some(&"new".to_string()));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsPatcherRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsPatcherRegistry::new();
        r.register("p1", SettingsPatcher::new(PatcherConfig::new(PatchMode::Strict)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_patcher_query() {
        assert!(is_patcher_query("settings patcher"));
        assert!(!is_patcher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = patcher_fun_fact();
        assert!(fact.contains("patcher"));
    }
}
