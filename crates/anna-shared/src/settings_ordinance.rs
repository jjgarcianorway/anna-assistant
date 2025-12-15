// v0.0.722: Settings Ordinance (Phase 298)
// Local ordinances for settings regulation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ordinance type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrdinanceType {
    /// Municipal ordinance
    #[default]
    Municipal,
    /// Regional ordinance
    Regional,
    /// Local ordinance
    Local,
    /// Zoning ordinance
    Zoning,
}

impl std::fmt::Display for OrdinanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Municipal => write!(f, "municipal"),
            Self::Regional => write!(f, "regional"),
            Self::Local => write!(f, "local"),
            Self::Zoning => write!(f, "zoning"),
        }
    }
}

/// Ordinance jurisdiction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrdinanceJurisdiction {
    /// City jurisdiction
    #[default]
    City,
    /// County jurisdiction
    County,
    /// District jurisdiction
    District,
    /// Zone jurisdiction
    Zone,
}

impl std::fmt::Display for OrdinanceJurisdiction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::City => write!(f, "city"),
            Self::County => write!(f, "county"),
            Self::District => write!(f, "district"),
            Self::Zone => write!(f, "zone"),
        }
    }
}

/// Ordinance config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdinanceConfig {
    /// Name
    pub name: String,
    /// Ordinance type
    pub ordinance_type: OrdinanceType,
    /// Jurisdiction
    pub jurisdiction: OrdinanceJurisdiction,
    /// Max ordinances
    pub max_ordinances: usize,
}

impl OrdinanceConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ordinance_type: OrdinanceType::Municipal,
            jurisdiction: OrdinanceJurisdiction::City,
            max_ordinances: 150,
        }
    }

    /// Set type
    pub fn ordinance_type(mut self, ot: OrdinanceType) -> Self {
        self.ordinance_type = ot;
        self
    }

    /// Set jurisdiction
    pub fn jurisdiction(mut self, j: OrdinanceJurisdiction) -> Self {
        self.jurisdiction = j;
        self
    }

    /// Set max ordinances
    pub fn max_ordinances(mut self, max: usize) -> Self {
        self.max_ordinances = max;
        self
    }
}

impl Default for OrdinanceConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Ordinance provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdinanceProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Text
    pub text: String,
    /// Section number
    pub section: String,
    /// Effective
    pub effective: bool,
}

impl OrdinanceProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            text: text.into(),
            section: String::new(),
            effective: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: impl Into<String>) -> Self {
        self.section = s.into();
        self
    }

    /// Make effective
    pub fn make_effective(&mut self) {
        self.effective = true;
    }

    /// Make ineffective
    pub fn make_ineffective(&mut self) {
        self.effective = false;
    }
}

/// Ordinance amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdinanceAmendment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Provision ID
    pub provision_id: String,
}

impl OrdinanceAmendment {
    /// Create new amendment
    pub fn new(key: impl Into<String>, value: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            provision_id: provision_id.into(),
        }
    }
}

/// Ordinance stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrdinanceStats {
    /// Total ordinances
    pub total_ordinances: usize,
    /// Effective ordinances
    pub effective: usize,
    /// Municipal count
    pub municipal_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl OrdinanceStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[OrdinanceProvision], ordinance_type: OrdinanceType) {
        self.total_ordinances = provisions.len();
        self.effective = provisions.iter().filter(|p| p.effective).count();
        if ordinance_type == OrdinanceType::Municipal {
            self.municipal_count = provisions.len();
        }
        *self.by_type.entry(ordinance_type.to_string()).or_insert(0) += 1;
    }

    /// Effective rate
    pub fn effective_rate(&self) -> f64 {
        if self.total_ordinances == 0 { 0.0 } else { self.effective as f64 / self.total_ordinances as f64 * 100.0 }
    }
}

/// Settings ordinance
#[derive(Debug, Clone, Default)]
pub struct SettingsOrdinance {
    /// Config
    config: OrdinanceConfig,
    /// Provisions
    provisions: Vec<OrdinanceProvision>,
    /// Amendments
    amendments: Vec<OrdinanceAmendment>,
    /// Stats
    stats: OrdinanceStats,
}

impl SettingsOrdinance {
    /// Create new ordinance system
    pub fn new(config: OrdinanceConfig) -> Self {
        Self {
            config,
            provisions: Vec::new(),
            amendments: Vec::new(),
            stats: OrdinanceStats::default(),
        }
    }

    /// Add provision
    pub fn add_provision(&mut self, provision: OrdinanceProvision) -> bool {
        if self.provisions.len() >= self.config.max_ordinances {
            return false;
        }
        self.provisions.push(provision);
        self.update_stats();
        true
    }

    /// Get provision
    pub fn get_provision(&self, id: &str) -> Option<&OrdinanceProvision> {
        self.provisions.iter().find(|p| p.id == id)
    }

    /// Get provision mut
    pub fn get_provision_mut(&mut self, id: &str) -> Option<&mut OrdinanceProvision> {
        self.provisions.iter_mut().find(|p| p.id == id)
    }

    /// Add amendment
    pub fn add_amendment(&mut self, amendment: OrdinanceAmendment) {
        self.amendments.push(amendment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.provisions, self.config.ordinance_type);
    }

    /// Get stats
    pub fn stats(&self) -> &OrdinanceStats {
        &self.stats
    }

    /// Provision count
    pub fn provision_count(&self) -> usize {
        self.provisions.len()
    }
}

/// Ordinance registry
#[derive(Debug, Clone, Default)]
pub struct OrdinanceRegistry {
    /// Ordinances by ID
    ordinances: HashMap<String, SettingsOrdinance>,
}

impl OrdinanceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register ordinance
    pub fn register(&mut self, id: impl Into<String>, ordinance: SettingsOrdinance) {
        self.ordinances.insert(id.into(), ordinance);
    }

    /// Unregister ordinance
    pub fn unregister(&mut self, id: &str) -> bool {
        self.ordinances.remove(id).is_some()
    }

    /// Get ordinance
    pub fn get(&self, id: &str) -> Option<&SettingsOrdinance> {
        self.ordinances.get(id)
    }

    /// Get ordinance mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsOrdinance> {
        self.ordinances.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.ordinances.len()
    }
}

/// Format ordinance registry
pub fn format_ordinance_registry(registry: &OrdinanceRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Ordinance Registry:\n");
    output.push_str(&format!("  Ordinances: {}\n", registry.count()));
    output
}

/// Check if query is about ordinance
pub fn is_ordinance_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings ordinance") || lower.contains("ordinance settings") || lower.contains("local ordinance")
}

/// Fun fact about ordinance
pub fn ordinance_fun_fact() -> &'static str {
    "Anna's settings ordinance implements local regulations for configuration management!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordinance_type_display() {
        assert_eq!(format!("{}", OrdinanceType::Municipal), "municipal");
        assert_eq!(format!("{}", OrdinanceType::Zoning), "zoning");
    }

    #[test]
    fn test_jurisdiction_display() {
        assert_eq!(format!("{}", OrdinanceJurisdiction::City), "city");
        assert_eq!(format!("{}", OrdinanceJurisdiction::District), "district");
    }

    #[test]
    fn test_config_new() {
        let c = OrdinanceConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = OrdinanceConfig::new("test")
            .ordinance_type(OrdinanceType::Regional)
            .jurisdiction(OrdinanceJurisdiction::County);
        assert_eq!(c.ordinance_type, OrdinanceType::Regional);
        assert_eq!(c.jurisdiction, OrdinanceJurisdiction::County);
    }

    #[test]
    fn test_provision_new() {
        let p = OrdinanceProvision::new("p1", "Title", "Text");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = OrdinanceProvision::new("p1", "Title", "Text")
            .section("1.1");
        assert_eq!(p.section, "1.1");
    }

    #[test]
    fn test_provision_effective() {
        let mut p = OrdinanceProvision::new("p1", "Title", "Text");
        p.make_effective();
        assert!(p.effective);
        p.make_ineffective();
        assert!(!p.effective);
    }

    #[test]
    fn test_amendment_new() {
        let a = OrdinanceAmendment::new("key", "value", "p1");
        assert_eq!(a.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = OrdinanceStats::default();
        let mut prov = OrdinanceProvision::new("p1", "Title", "Text");
        prov.make_effective();
        s.update(&[prov], OrdinanceType::Municipal);
        assert_eq!(s.total_ordinances, 1);
        assert_eq!(s.effective, 1);
        assert_eq!(s.municipal_count, 1);
    }

    #[test]
    fn test_ordinance_new() {
        let o = SettingsOrdinance::new(OrdinanceConfig::default());
        assert_eq!(o.provision_count(), 0);
    }

    #[test]
    fn test_ordinance_add_provision() {
        let mut o = SettingsOrdinance::new(OrdinanceConfig::default());
        o.add_provision(OrdinanceProvision::new("p1", "Title", "Text"));
        assert_eq!(o.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = OrdinanceRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = OrdinanceRegistry::new();
        r.register("o1", SettingsOrdinance::new(OrdinanceConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_ordinance_query() {
        assert!(is_ordinance_query("settings ordinance"));
        assert!(!is_ordinance_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = ordinance_fun_fact();
        assert!(fact.contains("ordinance"));
    }
}
