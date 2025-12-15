// v0.0.719: Settings Edict (Phase 295)
// Formal edicts for settings enforcement

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Edict type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EdictType {
    /// Royal edict
    #[default]
    Royal,
    /// Imperial edict
    Imperial,
    /// Sovereign edict
    Sovereign,
    /// Administrative edict
    Administrative,
}

impl std::fmt::Display for EdictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Royal => write!(f, "royal"),
            Self::Imperial => write!(f, "imperial"),
            Self::Sovereign => write!(f, "sovereign"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// Edict status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EdictStatus {
    /// Draft status
    #[default]
    Draft,
    /// Proclaimed status
    Proclaimed,
    /// Active status
    Active,
    /// Revoked status
    Revoked,
}

impl std::fmt::Display for EdictStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Proclaimed => write!(f, "proclaimed"),
            Self::Active => write!(f, "active"),
            Self::Revoked => write!(f, "revoked"),
        }
    }
}

/// Edict config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdictConfig {
    /// Name
    pub name: String,
    /// Edict type
    pub edict_type: EdictType,
    /// Default status
    pub default_status: EdictStatus,
    /// Max edicts
    pub max_edicts: usize,
}

impl EdictConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            edict_type: EdictType::Royal,
            default_status: EdictStatus::Draft,
            max_edicts: 100,
        }
    }

    /// Set type
    pub fn edict_type(mut self, et: EdictType) -> Self {
        self.edict_type = et;
        self
    }

    /// Set default status
    pub fn default_status(mut self, ds: EdictStatus) -> Self {
        self.default_status = ds;
        self
    }

    /// Set max edicts
    pub fn max_edicts(mut self, max: usize) -> Self {
        self.max_edicts = max;
        self
    }
}

impl Default for EdictConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Edict proclamation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdictProclamation {
    /// Proclamation ID
    pub id: String,
    /// Title
    pub title: String,
    /// Decree
    pub decree: String,
    /// Status
    pub status: EdictStatus,
    /// Seal
    pub seal: String,
}

impl EdictProclamation {
    /// Create new proclamation
    pub fn new(id: impl Into<String>, title: impl Into<String>, decree: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            decree: decree.into(),
            status: EdictStatus::Draft,
            seal: String::new(),
        }
    }

    /// Set seal
    pub fn seal(mut self, s: impl Into<String>) -> Self {
        self.seal = s.into();
        self
    }

    /// Proclaim edict
    pub fn proclaim(&mut self) {
        self.status = EdictStatus::Proclaimed;
    }

    /// Activate edict
    pub fn activate(&mut self) {
        self.status = EdictStatus::Active;
    }

    /// Revoke edict
    pub fn revoke(&mut self) {
        self.status = EdictStatus::Revoked;
    }
}

/// Edict annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdictAnnotation {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Proclamation ID
    pub proclamation_id: String,
}

impl EdictAnnotation {
    /// Create new annotation
    pub fn new(key: impl Into<String>, value: impl Into<String>, proclamation_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            proclamation_id: proclamation_id.into(),
        }
    }
}

/// Edict stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdictStats {
    /// Total edicts
    pub total_edicts: usize,
    /// Active edicts
    pub active: usize,
    /// Revoked edicts
    pub revoked: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl EdictStats {
    /// Update from proclamations
    pub fn update(&mut self, proclamations: &[EdictProclamation], edict_type: EdictType) {
        self.total_edicts = proclamations.len();
        self.active = proclamations.iter().filter(|p| p.status == EdictStatus::Active).count();
        self.revoked = proclamations.iter().filter(|p| p.status == EdictStatus::Revoked).count();
        *self.by_type.entry(edict_type.to_string()).or_insert(0) += 1;
    }

    /// Active rate
    pub fn active_rate(&self) -> f64 {
        if self.total_edicts == 0 { 0.0 } else { self.active as f64 / self.total_edicts as f64 * 100.0 }
    }
}

/// Settings edict
#[derive(Debug, Clone, Default)]
pub struct SettingsEdict {
    /// Config
    config: EdictConfig,
    /// Proclamations
    proclamations: Vec<EdictProclamation>,
    /// Annotations
    annotations: Vec<EdictAnnotation>,
    /// Stats
    stats: EdictStats,
}

impl SettingsEdict {
    /// Create new edict system
    pub fn new(config: EdictConfig) -> Self {
        Self {
            config,
            proclamations: Vec::new(),
            annotations: Vec::new(),
            stats: EdictStats::default(),
        }
    }

    /// Add proclamation
    pub fn add_proclamation(&mut self, proclamation: EdictProclamation) -> bool {
        if self.proclamations.len() >= self.config.max_edicts {
            return false;
        }
        self.proclamations.push(proclamation);
        self.update_stats();
        true
    }

    /// Get proclamation
    pub fn get_proclamation(&self, id: &str) -> Option<&EdictProclamation> {
        self.proclamations.iter().find(|p| p.id == id)
    }

    /// Get proclamation mut
    pub fn get_proclamation_mut(&mut self, id: &str) -> Option<&mut EdictProclamation> {
        self.proclamations.iter_mut().find(|p| p.id == id)
    }

    /// Add annotation
    pub fn add_annotation(&mut self, annotation: EdictAnnotation) {
        self.annotations.push(annotation);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.proclamations, self.config.edict_type);
    }

    /// Get stats
    pub fn stats(&self) -> &EdictStats {
        &self.stats
    }

    /// Proclamation count
    pub fn proclamation_count(&self) -> usize {
        self.proclamations.len()
    }
}

/// Edict registry
#[derive(Debug, Clone, Default)]
pub struct EdictRegistry {
    /// Edicts by ID
    edicts: HashMap<String, SettingsEdict>,
}

impl EdictRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register edict
    pub fn register(&mut self, id: impl Into<String>, edict: SettingsEdict) {
        self.edicts.insert(id.into(), edict);
    }

    /// Unregister edict
    pub fn unregister(&mut self, id: &str) -> bool {
        self.edicts.remove(id).is_some()
    }

    /// Get edict
    pub fn get(&self, id: &str) -> Option<&SettingsEdict> {
        self.edicts.get(id)
    }

    /// Get edict mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsEdict> {
        self.edicts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.edicts.len()
    }
}

/// Format edict registry
pub fn format_edict_registry(registry: &EdictRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Edict Registry:\n");
    output.push_str(&format!("  Edicts: {}\n", registry.count()));
    output
}

/// Check if query is about edict
pub fn is_edict_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings edict") || lower.contains("edict settings") || lower.contains("royal edict")
}

/// Fun fact about edict
pub fn edict_fun_fact() -> &'static str {
    "Anna's settings edict issues formal proclamations for configuration governance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edict_type_display() {
        assert_eq!(format!("{}", EdictType::Royal), "royal");
        assert_eq!(format!("{}", EdictType::Imperial), "imperial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", EdictStatus::Draft), "draft");
        assert_eq!(format!("{}", EdictStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = EdictConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = EdictConfig::new("test")
            .edict_type(EdictType::Sovereign)
            .default_status(EdictStatus::Proclaimed);
        assert_eq!(c.edict_type, EdictType::Sovereign);
        assert_eq!(c.default_status, EdictStatus::Proclaimed);
    }

    #[test]
    fn test_proclamation_new() {
        let p = EdictProclamation::new("p1", "Title", "Decree");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_proclamation_builder() {
        let p = EdictProclamation::new("p1", "Title", "Decree")
            .seal("ROYAL_SEAL");
        assert_eq!(p.seal, "ROYAL_SEAL");
    }

    #[test]
    fn test_proclamation_lifecycle() {
        let mut p = EdictProclamation::new("p1", "Title", "Decree");
        p.proclaim();
        assert_eq!(p.status, EdictStatus::Proclaimed);
        p.activate();
        assert_eq!(p.status, EdictStatus::Active);
        p.revoke();
        assert_eq!(p.status, EdictStatus::Revoked);
    }

    #[test]
    fn test_annotation_new() {
        let a = EdictAnnotation::new("key", "value", "p1");
        assert_eq!(a.proclamation_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = EdictStats::default();
        let mut proc = EdictProclamation::new("p1", "Title", "Decree");
        proc.activate();
        s.update(&[proc], EdictType::Royal);
        assert_eq!(s.total_edicts, 1);
        assert_eq!(s.active, 1);
    }

    #[test]
    fn test_edict_new() {
        let e = SettingsEdict::new(EdictConfig::default());
        assert_eq!(e.proclamation_count(), 0);
    }

    #[test]
    fn test_edict_add_proclamation() {
        let mut e = SettingsEdict::new(EdictConfig::default());
        e.add_proclamation(EdictProclamation::new("p1", "Title", "Decree"));
        assert_eq!(e.proclamation_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = EdictRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = EdictRegistry::new();
        r.register("e1", SettingsEdict::new(EdictConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_edict_query() {
        assert!(is_edict_query("settings edict"));
        assert!(!is_edict_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = edict_fun_fact();
        assert!(fact.contains("edict"));
    }
}
