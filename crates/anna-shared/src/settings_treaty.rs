// v0.0.727: Settings Treaty (Phase 303)
// International agreement for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Treaty type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TreatyType {
    /// Bilateral treaty
    #[default]
    Bilateral,
    /// Multilateral treaty
    Multilateral,
    /// Framework treaty
    Framework,
    /// Protocol treaty
    Protocol,
}

impl std::fmt::Display for TreatyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bilateral => write!(f, "bilateral"),
            Self::Multilateral => write!(f, "multilateral"),
            Self::Framework => write!(f, "framework"),
            Self::Protocol => write!(f, "protocol"),
        }
    }
}

/// Treaty status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TreatyStatus {
    /// Negotiating status
    #[default]
    Negotiating,
    /// Signed status
    Signed,
    /// Ratified status
    Ratified,
    /// Terminated status
    Terminated,
}

impl std::fmt::Display for TreatyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negotiating => write!(f, "negotiating"),
            Self::Signed => write!(f, "signed"),
            Self::Ratified => write!(f, "ratified"),
            Self::Terminated => write!(f, "terminated"),
        }
    }
}

/// Treaty config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyConfig {
    /// Name
    pub name: String,
    /// Treaty type
    pub treaty_type: TreatyType,
    /// Status
    pub status: TreatyStatus,
    /// Max provisions
    pub max_provisions: usize,
}

impl TreatyConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            treaty_type: TreatyType::Bilateral,
            status: TreatyStatus::Negotiating,
            max_provisions: 100,
        }
    }

    /// Set type
    pub fn treaty_type(mut self, tt: TreatyType) -> Self {
        self.treaty_type = tt;
        self
    }

    /// Set status
    pub fn status(mut self, s: TreatyStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max provisions
    pub fn max_provisions(mut self, max: usize) -> Self {
        self.max_provisions = max;
        self
    }
}

impl Default for TreatyConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Treaty provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub article: u32,
    /// In force
    pub in_force: bool,
}

impl TreatyProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            article: 0,
            in_force: false,
        }
    }

    /// Set article
    pub fn article(mut self, a: u32) -> Self {
        self.article = a;
        self
    }

    /// Enter into force
    pub fn enter_force(&mut self) {
        self.in_force = true;
    }

    /// Terminate
    pub fn terminate(&mut self) {
        self.in_force = false;
    }
}

/// Treaty signatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatySignatory {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Provision ID
    pub provision_id: String,
}

impl TreatySignatory {
    /// Create new signatory
    pub fn new(key: impl Into<String>, name: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            provision_id: provision_id.into(),
        }
    }
}

/// Treaty stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TreatyStats {
    /// Total provisions
    pub total_provisions: usize,
    /// In force provisions
    pub in_force: usize,
    /// Ratified count
    pub ratified_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TreatyStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[TreatyProvision], treaty_type: TreatyType) {
        self.total_provisions = provisions.len();
        self.in_force = provisions.iter().filter(|p| p.in_force).count();
        *self.by_type.entry(treaty_type.to_string()).or_insert(0) += 1;
    }

    /// In force rate
    pub fn in_force_rate(&self) -> f64 {
        if self.total_provisions == 0 { 0.0 } else { self.in_force as f64 / self.total_provisions as f64 * 100.0 }
    }
}

/// Settings treaty
#[derive(Debug, Clone, Default)]
pub struct SettingsTreaty {
    /// Config
    config: TreatyConfig,
    /// Provisions
    provisions: Vec<TreatyProvision>,
    /// Signatories
    signatories: Vec<TreatySignatory>,
    /// Stats
    stats: TreatyStats,
}

impl SettingsTreaty {
    /// Create new treaty system
    pub fn new(config: TreatyConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            signatories: Vec::new(),
            stats: TreatyStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: TreatyProvision) -> bool {
        if self.provisions.len() >= self.config.max_provisions {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&TreatyProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut TreatyProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add signatory
    pub fn add_signatory(&mut self, signatory: TreatySignatory) {
        self.signatories.push(signatory);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.treaty_type);
    }

    /// Get stats
    pub fn stats(&self) -> &TreatyStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}

/// Treaty registry
#[derive(Debug, Clone, Default)]
pub struct TreatyRegistry {
    /// Treaties by ID
    treaties: HashMap<String, SettingsTreaty>,
}

impl TreatyRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register treaty
    pub fn register(&mut self, id: impl Into<String>, treaty: SettingsTreaty) {
        self.treaties.insert(id.into(), treaty);
    }

    /// Unregister treaty
    pub fn unregister(&mut self, id: &str) -> bool {
        self.treaties.remove(id).is_some()
    }

    /// Get treaty
    pub fn get(&self, id: &str) -> Option<&SettingsTreaty> {
        self.treaties.get(id)
    }

    /// Get treaty mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTreaty> {
        self.treaties.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.treaties.len()
    }
}

/// Format treaty registry
pub fn format_treaty_registry(registry: &TreatyRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Treaty Registry:\n");
    output.push_str(&format!("  Treaties: {}\n", registry.count()));
    output
}

/// Check if query is about treaty
pub fn is_treaty_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings treaty") || lower.contains("treaty settings") || lower.contains("international agreement")
}

/// Fun fact about treaty
pub fn treaty_fun_fact() -> &'static str {
    "Anna's settings treaty establishes international governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treaty_type_display() {
        assert_eq!(format!("{}", TreatyType::Bilateral), "bilateral");
        assert_eq!(format!("{}", TreatyType::Multilateral), "multilateral");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TreatyStatus::Negotiating), "negotiating");
        assert_eq!(format!("{}", TreatyStatus::Ratified), "ratified");
    }

    #[test]
    fn test_config_new() {
        let c = TreatyConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = TreatyConfig::new("test")
            .treaty_type(TreatyType::Multilateral)
            .status(TreatyStatus::Signed);
        assert_eq!(c.treaty_type, TreatyType::Multilateral);
        assert_eq!(c.status, TreatyStatus::Signed);
    }

    #[test]
    fn test_provision_new() {
        let p = TreatyProvision::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = TreatyProvision::new("p1", "Title", "Content")
            .article(1);
        assert_eq!(p.article, 1);
    }

    #[test]
    fn test_provision_force_terminate() {
        let mut p = TreatyProvision::new("p1", "Title", "Content");
        p.enter_force();
        assert!(p.in_force);
        p.terminate();
        assert!(!p.in_force);
    }

    #[test]
    fn test_signatory_new() {
        let s = TreatySignatory::new("key", "name", "p1");
        assert_eq!(s.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = TreatyStats::default();
        let mut provision = TreatyProvision::new("p1", "Title", "Content");
        provision.enter_force();
        s.update(&[provision], TreatyType::Bilateral);
        assert_eq!(s.total_provisions, 1);
        assert_eq!(s.in_force, 1);
    }

    #[test]
    fn test_treaty_new() {
        let t = SettingsTreaty::new(TreatyConfig::default());
        assert_eq!(t.provision_count(), 0);
    }

    #[test]
    fn test_treaty_add_provision() {
        let mut t = SettingsTreaty::new(TreatyConfig::default());
        t.add_provision(TreatyProvision::new("p1", "Title", "Content"));
        assert_eq!(t.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = TreatyRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TreatyRegistry::new();
        r.register("t1", SettingsTreaty::new(TreatyConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_treaty_query() {
        assert!(is_treaty_query("settings treaty"));
        assert!(!is_treaty_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = treaty_fun_fact();
        assert!(fact.contains("treaty"));
    }
}
