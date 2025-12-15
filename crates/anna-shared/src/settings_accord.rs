// v0.0.730: Settings Accord (Phase 306)
// Formal accord for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Accord type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AccordType {
    /// Peace accord
    #[default]
    Peace,
    /// Trade accord
    Trade,
    /// Framework accord
    Framework,
    /// Settlement accord
    Settlement,
}

impl std::fmt::Display for AccordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Peace => write!(f, "peace"),
            Self::Trade => write!(f, "trade"),
            Self::Framework => write!(f, "framework"),
            Self::Settlement => write!(f, "settlement"),
        }
    }
}

/// Accord status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AccordStatus {
    /// Preliminary status
    #[default]
    Preliminary,
    /// Final status
    Final,
    /// Implemented status
    Implemented,
    /// Voided status
    Voided,
}

impl std::fmt::Display for AccordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preliminary => write!(f, "preliminary"),
            Self::Final => write!(f, "final"),
            Self::Implemented => write!(f, "implemented"),
            Self::Voided => write!(f, "voided"),
        }
    }
}

/// Accord config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccordConfig {
    /// Name
    pub name: String,
    /// Accord type
    pub accord_type: AccordType,
    /// Status
    pub status: AccordStatus,
    /// Max provisions
    pub max_provisions: usize,
}

impl AccordConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            accord_type: AccordType::Peace,
            status: AccordStatus::Preliminary,
            max_provisions: 100,
        }
    }

    /// Set type
    pub fn accord_type(mut self, at: AccordType) -> Self {
        self.accord_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AccordStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max provisions
    pub fn max_provisions(mut self, max: usize) -> Self {
        self.max_provisions = max;
        self
    }
}

impl Default for AccordConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Accord provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccordProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Agreed
    pub agreed: bool,
}

impl AccordProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            agreed: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Agree to provision
    pub fn agree(&mut self) {
        self.agreed = true;
    }

    /// Disagree to provision
    pub fn disagree(&mut self) {
        self.agreed = false;
    }
}

/// Accord signatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccordSignatory {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Provision ID
    pub provision_id: String,
}

impl AccordSignatory {
    /// Create new signatory
    pub fn new(key: impl Into<String>, name: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            provision_id: provision_id.into(),
        }
    }
}

/// Accord stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccordStats {
    /// Total provisions
    pub total_provisions: usize,
    /// Agreed provisions
    pub agreed: usize,
    /// Implemented count
    pub implemented_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AccordStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[AccordProvision], accord_type: AccordType) {
        self.total_provisions = provisions.len();
        self.agreed = provisions.iter().filter(|p| p.agreed).count();
        *self.by_type.entry(accord_type.to_string()).or_insert(0) += 1;
    }

    /// Agreement rate
    pub fn agreement_rate(&self) -> f64 {
        if self.total_provisions == 0 { 0.0 } else { self.agreed as f64 / self.total_provisions as f64 * 100.0 }
    }
}

/// Settings accord
#[derive(Debug, Clone, Default)]
pub struct SettingsAccord {
    /// Config
    config: AccordConfig,
    /// Provisions
    provisions: Vec<AccordProvision>,
    /// Signatories
    signatories: Vec<AccordSignatory>,
    /// Stats
    stats: AccordStats,
}

impl SettingsAccord {
    /// Create new accord system
    pub fn new(config: AccordConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            signatories: Vec::new(),
            stats: AccordStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: AccordProvision) -> bool {
        if self.provisions.len() >= self.config.max_provisions {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&AccordProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut AccordProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add signatory
    pub fn add_signatory(&mut self, signatory: AccordSignatory) {
        self.signatories.push(signatory);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.accord_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AccordStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}

/// Accord registry
#[derive(Debug, Clone, Default)]
pub struct AccordRegistry {
    /// Accords by ID
    accords: HashMap<String, SettingsAccord>,
}

impl AccordRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register accord
    pub fn register(&mut self, id: impl Into<String>, accord: SettingsAccord) {
        self.accords.insert(id.into(), accord);
    }

    /// Unregister accord
    pub fn unregister(&mut self, id: &str) -> bool {
        self.accords.remove(id).is_some()
    }

    /// Get accord
    pub fn get(&self, id: &str) -> Option<&SettingsAccord> {
        self.accords.get(id)
    }

    /// Get accord mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAccord> {
        self.accords.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.accords.len()
    }
}

/// Format accord registry
pub fn format_accord_registry(registry: &AccordRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Accord Registry:\n");
    output.push_str(&format!("  Accords: {}\n", registry.count()));
    output
}

/// Check if query is about accord
pub fn is_accord_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings accord") || lower.contains("accord settings") || lower.contains("formal agreement")
}

/// Fun fact about accord
pub fn accord_fun_fact() -> &'static str {
    "Anna's settings accord establishes formal governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accord_type_display() {
        assert_eq!(format!("{}", AccordType::Peace), "peace");
        assert_eq!(format!("{}", AccordType::Trade), "trade");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AccordStatus::Preliminary), "preliminary");
        assert_eq!(format!("{}", AccordStatus::Implemented), "implemented");
    }

    #[test]
    fn test_config_new() {
        let c = AccordConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AccordConfig::new("test")
            .accord_type(AccordType::Trade)
            .status(AccordStatus::Final);
        assert_eq!(c.accord_type, AccordType::Trade);
        assert_eq!(c.status, AccordStatus::Final);
    }

    #[test]
    fn test_provision_new() {
        let p = AccordProvision::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = AccordProvision::new("p1", "Title", "Content")
            .section(1);
        assert_eq!(p.section, 1);
    }

    #[test]
    fn test_provision_agree_disagree() {
        let mut p = AccordProvision::new("p1", "Title", "Content");
        p.agree();
        assert!(p.agreed);
        p.disagree();
        assert!(!p.agreed);
    }

    #[test]
    fn test_signatory_new() {
        let s = AccordSignatory::new("key", "name", "p1");
        assert_eq!(s.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AccordStats::default();
        let mut provision = AccordProvision::new("p1", "Title", "Content");
        provision.agree();
        s.update(&[provision], AccordType::Peace);
        assert_eq!(s.total_provisions, 1);
        assert_eq!(s.agreed, 1);
    }

    #[test]
    fn test_accord_new() {
        let a = SettingsAccord::new(AccordConfig::default());
        assert_eq!(a.provision_count(), 0);
    }

    #[test]
    fn test_accord_add_provision() {
        let mut a = SettingsAccord::new(AccordConfig::default());
        a.add_provision(AccordProvision::new("p1", "Title", "Content"));
        assert_eq!(a.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AccordRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AccordRegistry::new();
        r.register("a1", SettingsAccord::new(AccordConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_accord_query() {
        assert!(is_accord_query("settings accord"));
        assert!(!is_accord_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = accord_fun_fact();
        assert!(fact.contains("accord"));
    }
}
