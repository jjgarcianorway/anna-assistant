// v0.0.720: Settings Decree (Phase 296)
// Official decrees for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Decree type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DecreeType {
    /// Executive decree
    #[default]
    Executive,
    /// Legislative decree
    Legislative,
    /// Judicial decree
    Judicial,
    /// Emergency decree
    Emergency,
}

impl std::fmt::Display for DecreeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executive => write!(f, "executive"),
            Self::Legislative => write!(f, "legislative"),
            Self::Judicial => write!(f, "judicial"),
            Self::Emergency => write!(f, "emergency"),
        }
    }
}

/// Decree binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DecreeBinding {
    /// Mandatory binding
    #[default]
    Mandatory,
    /// Recommended binding
    Recommended,
    /// Voluntary binding
    Voluntary,
    /// Advisory binding
    Advisory,
}

impl std::fmt::Display for DecreeBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mandatory => write!(f, "mandatory"),
            Self::Recommended => write!(f, "recommended"),
            Self::Voluntary => write!(f, "voluntary"),
            Self::Advisory => write!(f, "advisory"),
        }
    }
}

/// Decree config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreeConfig {
    /// Name
    pub name: String,
    /// Decree type
    pub decree_type: DecreeType,
    /// Binding level
    pub binding: DecreeBinding,
    /// Max decrees
    pub max_decrees: usize,
}

impl DecreeConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            decree_type: DecreeType::Executive,
            binding: DecreeBinding::Mandatory,
            max_decrees: 100,
        }
    }

    /// Set type
    pub fn decree_type(mut self, dt: DecreeType) -> Self {
        self.decree_type = dt;
        self
    }

    /// Set binding
    pub fn binding(mut self, b: DecreeBinding) -> Self {
        self.binding = b;
        self
    }

    /// Set max decrees
    pub fn max_decrees(mut self, max: usize) -> Self {
        self.max_decrees = max;
        self
    }
}

impl Default for DecreeConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Decree ruling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreeRuling {
    /// Ruling ID
    pub id: String,
    /// Title
    pub title: String,
    /// Text
    pub text: String,
    /// Binding
    pub binding: DecreeBinding,
    /// In force
    pub in_force: bool,
}

impl DecreeRuling {
    /// Create new ruling
    pub fn new(id: impl Into<String>, title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            text: text.into(),
            binding: DecreeBinding::Mandatory,
            in_force: false,
        }
    }

    /// Set binding
    pub fn binding(mut self, b: DecreeBinding) -> Self {
        self.binding = b;
        self
    }

    /// Put in force
    pub fn enact(&mut self) {
        self.in_force = true;
    }

    /// Remove from force
    pub fn repeal(&mut self) {
        self.in_force = false;
    }
}

/// Decree clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreeClause {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Ruling ID
    pub ruling_id: String,
}

impl DecreeClause {
    /// Create new clause
    pub fn new(key: impl Into<String>, value: impl Into<String>, ruling_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            ruling_id: ruling_id.into(),
        }
    }
}

/// Decree stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecreeStats {
    /// Total decrees
    pub total_decrees: usize,
    /// In force
    pub in_force: usize,
    /// Emergency decrees
    pub emergency_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DecreeStats {
    /// Update from rulings
    pub fn update(&mut self, rulings: &[DecreeRuling], decree_type: DecreeType) {
        self.total_decrees = rulings.len();
        self.in_force = rulings.iter().filter(|r| r.in_force).count();
        if decree_type == DecreeType::Emergency {
            self.emergency_count = rulings.len();
        }
        *self.by_type.entry(decree_type.to_string()).or_insert(0) += 1;
    }

    /// In force rate
    pub fn in_force_rate(&self) -> f64 {
        if self.total_decrees == 0 { 0.0 } else { self.in_force as f64 / self.total_decrees as f64 * 100.0 }
    }
}

/// Settings decree
#[derive(Debug, Clone, Default)]
pub struct SettingsDecree {
    /// Config
    config: DecreeConfig,
    /// Rulings
    rulings: Vec<DecreeRuling>,
    /// Clauses
    clauses: Vec<DecreeClause>,
    /// Stats
    stats: DecreeStats,
}

impl SettingsDecree {
    /// Create new decree system
    pub fn new(config: DecreeConfig) -> Self {
        Self {
            config,
            rulings: Vec::new(),
            clauses: Vec::new(),
            stats: DecreeStats::default(),
        }
    }

    /// Add ruling
    pub fn add_ruling(&mut self, ruling: DecreeRuling) -> bool {
        if self.rulings.len() >= self.config.max_decrees {
            return false;
        }
        self.rulings.push(ruling);
        self.update_stats();
        true
    }

    /// Get ruling
    pub fn get_ruling(&self, id: &str) -> Option<&DecreeRuling> {
        self.rulings.iter().find(|r| r.id == id)
    }

    /// Get ruling mut
    pub fn get_ruling_mut(&mut self, id: &str) -> Option<&mut DecreeRuling> {
        self.rulings.iter_mut().find(|r| r.id == id)
    }

    /// Add clause
    pub fn add_clause(&mut self, clause: DecreeClause) {
        self.clauses.push(clause);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.rulings, self.config.decree_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DecreeStats {
        &self.stats
    }

    /// Ruling count
    pub fn ruling_count(&self) -> usize {
        self.rulings.len()
    }
}

/// Decree registry
#[derive(Debug, Clone, Default)]
pub struct DecreeRegistry {
    /// Decrees by ID
    decrees: HashMap<String, SettingsDecree>,
}

impl DecreeRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register decree
    pub fn register(&mut self, id: impl Into<String>, decree: SettingsDecree) {
        self.decrees.insert(id.into(), decree);
    }

    /// Unregister decree
    pub fn unregister(&mut self, id: &str) -> bool {
        self.decrees.remove(id).is_some()
    }

    /// Get decree
    pub fn get(&self, id: &str) -> Option<&SettingsDecree> {
        self.decrees.get(id)
    }

    /// Get decree mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDecree> {
        self.decrees.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.decrees.len()
    }
}

/// Format decree registry
pub fn format_decree_registry(registry: &DecreeRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Decree Registry:\n");
    output.push_str(&format!("  Decrees: {}\n", registry.count()));
    output
}

/// Check if query is about decree
pub fn is_decree_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings decree") || lower.contains("decree settings") || lower.contains("executive decree")
}

/// Fun fact about decree
pub fn decree_fun_fact() -> &'static str {
    "Anna's settings decree issues official rulings for configuration governance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decree_type_display() {
        assert_eq!(format!("{}", DecreeType::Executive), "executive");
        assert_eq!(format!("{}", DecreeType::Emergency), "emergency");
    }

    #[test]
    fn test_binding_display() {
        assert_eq!(format!("{}", DecreeBinding::Mandatory), "mandatory");
        assert_eq!(format!("{}", DecreeBinding::Advisory), "advisory");
    }

    #[test]
    fn test_config_new() {
        let c = DecreeConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DecreeConfig::new("test")
            .decree_type(DecreeType::Legislative)
            .binding(DecreeBinding::Recommended);
        assert_eq!(c.decree_type, DecreeType::Legislative);
        assert_eq!(c.binding, DecreeBinding::Recommended);
    }

    #[test]
    fn test_ruling_new() {
        let r = DecreeRuling::new("r1", "Title", "Text");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_ruling_builder() {
        let r = DecreeRuling::new("r1", "Title", "Text")
            .binding(DecreeBinding::Voluntary);
        assert_eq!(r.binding, DecreeBinding::Voluntary);
    }

    #[test]
    fn test_ruling_enact_repeal() {
        let mut r = DecreeRuling::new("r1", "Title", "Text");
        r.enact();
        assert!(r.in_force);
        r.repeal();
        assert!(!r.in_force);
    }

    #[test]
    fn test_clause_new() {
        let c = DecreeClause::new("key", "value", "r1");
        assert_eq!(c.ruling_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DecreeStats::default();
        let mut ruling = DecreeRuling::new("r1", "Title", "Text");
        ruling.enact();
        s.update(&[ruling], DecreeType::Executive);
        assert_eq!(s.total_decrees, 1);
        assert_eq!(s.in_force, 1);
    }

    #[test]
    fn test_decree_new() {
        let d = SettingsDecree::new(DecreeConfig::default());
        assert_eq!(d.ruling_count(), 0);
    }

    #[test]
    fn test_decree_add_ruling() {
        let mut d = SettingsDecree::new(DecreeConfig::default());
        d.add_ruling(DecreeRuling::new("r1", "Title", "Text"));
        assert_eq!(d.ruling_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DecreeRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DecreeRegistry::new();
        r.register("d1", SettingsDecree::new(DecreeConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_decree_query() {
        assert!(is_decree_query("settings decree"));
        assert!(!is_decree_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = decree_fun_fact();
        assert!(fact.contains("decree"));
    }
}
