// v0.0.734: Settings Entente (Phase 310)
// Informal understanding for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entente type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EntenteType {
    /// Cordiale entente
    #[default]
    Cordiale,
    /// Strategic entente
    Strategic,
    /// Commercial entente
    Commercial,
    /// Cultural entente
    Cultural,
}

impl std::fmt::Display for EntenteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cordiale => write!(f, "cordiale"),
            Self::Strategic => write!(f, "strategic"),
            Self::Commercial => write!(f, "commercial"),
            Self::Cultural => write!(f, "cultural"),
        }
    }
}

/// Entente status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EntenteStatus {
    /// Informal status
    #[default]
    Informal,
    /// Formalized status
    Formalized,
    /// Active status
    Active,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for EntenteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Informal => write!(f, "informal"),
            Self::Formalized => write!(f, "formalized"),
            Self::Active => write!(f, "active"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}

/// Entente config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntenteConfig {
    /// Name
    pub name: String,
    /// Entente type
    pub entente_type: EntenteType,
    /// Status
    pub status: EntenteStatus,
    /// Max understandings
    pub max_understandings: usize,
}

impl EntenteConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entente_type: EntenteType::Cordiale,
            status: EntenteStatus::Informal,
            max_understandings: 100,
        }
    }

    /// Set type
    pub fn entente_type(mut self, et: EntenteType) -> Self {
        self.entente_type = et;
        self
    }

    /// Set status
    pub fn status(mut self, s: EntenteStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max understandings
    pub fn max_understandings(mut self, max: usize) -> Self {
        self.max_understandings = max;
        self
    }
}

impl Default for EntenteConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Entente understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntenteUnderstanding {
    /// Understanding ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Point number
    pub point: u32,
    /// Tacit
    pub tacit: bool,
}

impl EntenteUnderstanding {
    /// Create new understanding
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            point: 0,
            tacit: true,
        }
    }

    /// Set point
    pub fn point(mut self, p: u32) -> Self {
        self.point = p;
        self
    }

    /// Make tacit
    pub fn make_tacit(&mut self) {
        self.tacit = true;
    }

    /// Make explicit
    pub fn make_explicit(&mut self) {
        self.tacit = false;
    }
}

/// Entente partner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntentePartner {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Understanding ID
    pub understanding_id: String,
}

impl EntentePartner {
    /// Create new partner
    pub fn new(key: impl Into<String>, name: impl Into<String>, understanding_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            understanding_id: understanding_id.into(),
        }
    }
}

/// Entente stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntenteStats {
    /// Total understandings
    pub total_understandings: usize,
    /// Tacit understandings
    pub tacit: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl EntenteStats {
    /// Update from understandings
    pub fn update(&mut self, understandings: &[EntenteUnderstanding], entente_type: EntenteType) {
        self.total_understandings = understandings.len();
        self.tacit = understandings.iter().filter(|u| u.tacit).count();
        *self.by_type.entry(entente_type.to_string()).or_insert(0) += 1;
    }

    /// Tacit rate
    pub fn tacit_rate(&self) -> f64 {
        if self.total_understandings == 0 { 0.0 } else { self.tacit as f64 / self.total_understandings as f64 * 100.0 }
    }
}

/// Settings entente
#[derive(Debug, Clone, Default)]
pub struct SettingsEntente {
    /// Config
    config: EntenteConfig,
    /// Understandings
    understandings: Vec<EntenteUnderstanding>,
    /// Partners
    partners: Vec<EntentePartner>,
    /// Stats
    stats: EntenteStats,
}

impl SettingsEntente {
    /// Create new entente system
    pub fn new(config: EntenteConfig) -> Self {
        Self {
            config,
            understandings: Vec::new(),
            partners: Vec::new(),
            stats: EntenteStats::default(),
        }
    }

    /// Add understanding
    pub fn add_understanding(&mut self, understanding: EntenteUnderstanding) -> bool {
        if self.understandings.len() >= self.config.max_understandings {
            return false;
        }
        self.understandings.push(understanding);
        self.update_stats();
        true
    }

    /// Get understanding
    pub fn get_understanding(&self, id: &str) -> Option<&EntenteUnderstanding> {
        self.understandings.iter().find(|u| u.id == id)
    }

    /// Get understanding mut
    pub fn get_understanding_mut(&mut self, id: &str) -> Option<&mut EntenteUnderstanding> {
        self.understandings.iter_mut().find(|u| u.id == id)
    }

    /// Add partner
    pub fn add_partner(&mut self, partner: EntentePartner) {
        self.partners.push(partner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.understandings, self.config.entente_type);
    }

    /// Get stats
    pub fn stats(&self) -> &EntenteStats {
        &self.stats
    }

    /// Understanding count
    pub fn understanding_count(&self) -> usize {
        self.understandings.len()
    }
}

/// Entente registry
#[derive(Debug, Clone, Default)]
pub struct EntenteRegistry {
    /// Ententes by ID
    ententes: HashMap<String, SettingsEntente>,
}

impl EntenteRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register entente
    pub fn register(&mut self, id: impl Into<String>, entente: SettingsEntente) {
        self.ententes.insert(id.into(), entente);
    }

    /// Unregister entente
    pub fn unregister(&mut self, id: &str) -> bool {
        self.ententes.remove(id).is_some()
    }

    /// Get entente
    pub fn get(&self, id: &str) -> Option<&SettingsEntente> {
        self.ententes.get(id)
    }

    /// Get entente mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsEntente> {
        self.ententes.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.ententes.len()
    }
}

/// Format entente registry
pub fn format_entente_registry(registry: &EntenteRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Entente Registry:\n");
    output.push_str(&format!("  Ententes: {}\n", registry.count()));
    output
}

/// Check if query is about entente
pub fn is_entente_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings entente") || lower.contains("entente settings") || lower.contains("informal understanding")
}

/// Fun fact about entente
pub fn entente_fun_fact() -> &'static str {
    "Anna's settings entente establishes informal governance understandings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entente_type_display() {
        assert_eq!(format!("{}", EntenteType::Cordiale), "cordiale");
        assert_eq!(format!("{}", EntenteType::Strategic), "strategic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", EntenteStatus::Informal), "informal");
        assert_eq!(format!("{}", EntenteStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = EntenteConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = EntenteConfig::new("test")
            .entente_type(EntenteType::Strategic)
            .status(EntenteStatus::Formalized);
        assert_eq!(c.entente_type, EntenteType::Strategic);
        assert_eq!(c.status, EntenteStatus::Formalized);
    }

    #[test]
    fn test_understanding_new() {
        let u = EntenteUnderstanding::new("u1", "Title", "Content");
        assert_eq!(u.id, "u1");
    }

    #[test]
    fn test_understanding_builder() {
        let u = EntenteUnderstanding::new("u1", "Title", "Content")
            .point(1);
        assert_eq!(u.point, 1);
    }

    #[test]
    fn test_understanding_tacit() {
        let mut u = EntenteUnderstanding::new("u1", "Title", "Content");
        u.make_explicit();
        assert!(!u.tacit);
        u.make_tacit();
        assert!(u.tacit);
    }

    #[test]
    fn test_partner_new() {
        let p = EntentePartner::new("key", "name", "u1");
        assert_eq!(p.understanding_id, "u1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = EntenteStats::default();
        let understanding = EntenteUnderstanding::new("u1", "Title", "Content");
        s.update(&[understanding], EntenteType::Cordiale);
        assert_eq!(s.total_understandings, 1);
        assert_eq!(s.tacit, 1);
    }

    #[test]
    fn test_entente_new() {
        let e = SettingsEntente::new(EntenteConfig::default());
        assert_eq!(e.understanding_count(), 0);
    }

    #[test]
    fn test_entente_add_understanding() {
        let mut e = SettingsEntente::new(EntenteConfig::default());
        e.add_understanding(EntenteUnderstanding::new("u1", "Title", "Content"));
        assert_eq!(e.understanding_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = EntenteRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = EntenteRegistry::new();
        r.register("e1", SettingsEntente::new(EntenteConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_entente_query() {
        assert!(is_entente_query("settings entente"));
        assert!(!is_entente_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = entente_fun_fact();
        assert!(fact.contains("entente"));
    }
}
