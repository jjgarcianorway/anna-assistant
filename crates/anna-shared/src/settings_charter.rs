// v0.0.724: Settings Charter (Phase 300)
// Foundational charter for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Charter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CharterType {
    /// Founding charter
    #[default]
    Founding,
    /// Corporate charter
    Corporate,
    /// Municipal charter
    Municipal,
    /// Royal charter
    Royal,
}

impl std::fmt::Display for CharterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Founding => write!(f, "founding"),
            Self::Corporate => write!(f, "corporate"),
            Self::Municipal => write!(f, "municipal"),
            Self::Royal => write!(f, "royal"),
        }
    }
}

/// Charter status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CharterStatus {
    /// Draft status
    #[default]
    Draft,
    /// Ratified status
    Ratified,
    /// Amended status
    Amended,
    /// Revoked status
    Revoked,
}

impl std::fmt::Display for CharterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Ratified => write!(f, "ratified"),
            Self::Amended => write!(f, "amended"),
            Self::Revoked => write!(f, "revoked"),
        }
    }
}

/// Charter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharterConfig {
    /// Name
    pub name: String,
    /// Charter type
    pub charter_type: CharterType,
    /// Status
    pub status: CharterStatus,
    /// Max provisions
    pub max_provisions: usize,
}

impl CharterConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            charter_type: CharterType::Founding,
            status: CharterStatus::Draft,
            max_provisions: 150,
        }
    }

    /// Set type
    pub fn charter_type(mut self, ct: CharterType) -> Self {
        self.charter_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CharterStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max provisions
    pub fn max_provisions(mut self, max: usize) -> Self {
        self.max_provisions = max;
        self
    }
}

impl Default for CharterConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Charter provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharterProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: String,
    /// Active
    pub active: bool,
}

impl CharterProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: String::new(),
            active: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: impl Into<String>) -> Self {
        self.section = s.into();
        self
    }

    /// Activate provision
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate provision
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Charter amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharterAmendment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Provision ID
    pub provision_id: String,
}

impl CharterAmendment {
    /// Create new amendment
    pub fn new(key: impl Into<String>, value: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            provision_id: provision_id.into(),
        }
    }
}

/// Charter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharterStats {
    /// Total provisions
    pub total_provisions: usize,
    /// Active provisions
    pub active: usize,
    /// Ratified count
    pub ratified_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CharterStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[CharterProvision], charter_type: CharterType) {
        self.total_provisions = provisions.len();
        self.active = provisions.iter().filter(|p| p.active).count();
        *self.by_type.entry(charter_type.to_string()).or_insert(0) += 1;
    }

    /// Active rate
    pub fn active_rate(&self) -> f64 {
        if self.total_provisions == 0 { 0.0 } else { self.active as f64 / self.total_provisions as f64 * 100.0 }
    }
}

/// Settings charter
#[derive(Debug, Clone, Default)]
pub struct SettingsCharter {
    /// Config
    config: CharterConfig,
    /// Provisions
    provisions: Vec<CharterProvision>,
    /// Amendments
    amendments: Vec<CharterAmendment>,
    /// Stats
    stats: CharterStats,
}

impl SettingsCharter {
    /// Create new charter system
    pub fn new(config: CharterConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            amendments: Vec::new(),
            stats: CharterStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: CharterProvision) -> bool {
        if self.provisions.len() >= self.config.max_provisions {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&CharterProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut CharterProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add amendment
    pub fn add_amendment(&mut self, amendment: CharterAmendment) {
        self.amendments.push(amendment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.charter_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CharterStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}

/// Charter registry
#[derive(Debug, Clone, Default)]
pub struct CharterRegistry {
    /// Charters by ID
    charters: HashMap<String, SettingsCharter>,
}

impl CharterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register charter
    pub fn register(&mut self, id: impl Into<String>, charter: SettingsCharter) {
        self.charters.insert(id.into(), charter);
    }

    /// Unregister charter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.charters.remove(id).is_some()
    }

    /// Get charter
    pub fn get(&self, id: &str) -> Option<&SettingsCharter> {
        self.charters.get(id)
    }

    /// Get charter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCharter> {
        self.charters.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.charters.len()
    }
}

/// Format charter registry
pub fn format_charter_registry(registry: &CharterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Charter Registry:\n");
    output.push_str(&format!("  Charters: {}\n", registry.count()));
    output
}

/// Check if query is about charter
pub fn is_charter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings charter") || lower.contains("charter settings") || lower.contains("foundational document")
}

/// Fun fact about charter
pub fn charter_fun_fact() -> &'static str {
    "Anna's settings charter establishes foundational governance principles!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charter_type_display() {
        assert_eq!(format!("{}", CharterType::Founding), "founding");
        assert_eq!(format!("{}", CharterType::Royal), "royal");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CharterStatus::Draft), "draft");
        assert_eq!(format!("{}", CharterStatus::Ratified), "ratified");
    }

    #[test]
    fn test_config_new() {
        let c = CharterConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CharterConfig::new("test")
            .charter_type(CharterType::Municipal)
            .status(CharterStatus::Amended);
        assert_eq!(c.charter_type, CharterType::Municipal);
        assert_eq!(c.status, CharterStatus::Amended);
    }

    #[test]
    fn test_provision_new() {
        let p = CharterProvision::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = CharterProvision::new("p1", "Title", "Content")
            .section("Section 1");
        assert_eq!(p.section, "Section 1");
    }

    #[test]
    fn test_provision_activate_deactivate() {
        let mut p = CharterProvision::new("p1", "Title", "Content");
        p.deactivate();
        assert!(!p.active);
        p.activate();
        assert!(p.active);
    }

    #[test]
    fn test_amendment_new() {
        let a = CharterAmendment::new("key", "value", "p1");
        assert_eq!(a.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CharterStats::default();
        let provision = CharterProvision::new("p1", "Title", "Content");
        s.update(&[provision], CharterType::Founding);
        assert_eq!(s.total_provisions, 1);
        assert_eq!(s.active, 1);
    }

    #[test]
    fn test_charter_new() {
        let c = SettingsCharter::new(CharterConfig::default());
        assert_eq!(c.provision_count(), 0);
    }

    #[test]
    fn test_charter_add_provision() {
        let mut c = SettingsCharter::new(CharterConfig::default());
        c.add_provision(CharterProvision::new("p1", "Title", "Content"));
        assert_eq!(c.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CharterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CharterRegistry::new();
        r.register("c1", SettingsCharter::new(CharterConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_charter_query() {
        assert!(is_charter_query("settings charter"));
        assert!(!is_charter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = charter_fun_fact();
        assert!(fact.contains("charter"));
    }
}
