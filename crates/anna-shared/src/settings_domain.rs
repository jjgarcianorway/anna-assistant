// v0.0.743: Settings Domain (Phase 319)
// Sovereign domain for settings jurisdiction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Domain type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DomainType {
    /// Public domain
    #[default]
    Public,
    /// Private domain
    Private,
    /// Royal domain
    Royal,
    /// Eminent domain
    Eminent,
}

impl std::fmt::Display for DomainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Private => write!(f, "private"),
            Self::Royal => write!(f, "royal"),
            Self::Eminent => write!(f, "eminent"),
        }
    }
}

/// Domain status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DomainStatus {
    /// Claimed status
    #[default]
    Claimed,
    /// Recognized status
    Recognized,
    /// Consolidated status
    Consolidated,
    /// Disputed status
    Disputed,
}

impl std::fmt::Display for DomainStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claimed => write!(f, "claimed"),
            Self::Recognized => write!(f, "recognized"),
            Self::Consolidated => write!(f, "consolidated"),
            Self::Disputed => write!(f, "disputed"),
        }
    }
}

/// Domain config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Name
    pub name: String,
    /// Domain type
    pub domain_type: DomainType,
    /// Status
    pub status: DomainStatus,
    /// Max rights
    pub max_rights: usize,
}

impl DomainConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            domain_type: DomainType::Public,
            status: DomainStatus::Claimed,
            max_rights: 100,
        }
    }

    /// Set type
    pub fn domain_type(mut self, dt: DomainType) -> Self {
        self.domain_type = dt;
        self
    }

    /// Set status
    pub fn status(mut self, s: DomainStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max rights
    pub fn max_rights(mut self, max: usize) -> Self {
        self.max_rights = max;
        self
    }
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Domain right
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRight {
    /// Right ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Priority level
    pub priority: u32,
    /// Exclusive
    pub exclusive: bool,
}

impl DomainRight {
    /// Create new right
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            priority: 0,
            exclusive: true,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: u32) -> Self {
        self.priority = p;
        self
    }

    /// Make exclusive
    pub fn make_exclusive(&mut self) {
        self.exclusive = true;
    }

    /// Make shared
    pub fn make_shared(&mut self) {
        self.exclusive = false;
    }
}

/// Domain holder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainHolder {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Right ID
    pub right_id: String,
}

impl DomainHolder {
    /// Create new holder
    pub fn new(key: impl Into<String>, name: impl Into<String>, right_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            right_id: right_id.into(),
        }
    }
}

/// Domain stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainStats {
    /// Total rights
    pub total_rights: usize,
    /// Exclusive rights
    pub exclusive: usize,
    /// Consolidated count
    pub consolidated_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DomainStats {
    /// Update from rights
    pub fn update(&mut self, rights: &[DomainRight], domain_type: DomainType) {
        self.total_rights = rights.len();
        self.exclusive = rights.iter().filter(|r| r.exclusive).count();
        *self.by_type.entry(domain_type.to_string()).or_insert(0) += 1;
    }

    /// Exclusive rate
    pub fn exclusive_rate(&self) -> f64 {
        if self.total_rights == 0 { 0.0 } else { self.exclusive as f64 / self.total_rights as f64 * 100.0 }
    }
}

/// Settings domain
#[derive(Debug, Clone, Default)]
pub struct SettingsDomain {
    /// Config
    config: DomainConfig,
    /// Rights
    rights: Vec<DomainRight>,
    /// Holders
    holders: Vec<DomainHolder>,
    /// Stats
    stats: DomainStats,
}

impl SettingsDomain {
    /// Create new domain system
    pub fn new(config: DomainConfig) -> Self {
        Self {
            config,
            rights: Vec::new(),
            holders: Vec::new(),
            stats: DomainStats::default(),
        }
    }

    /// Add right
    pub fn add_right(&mut self, right: DomainRight) -> bool {
        if self.rights.len() >= self.config.max_rights {
            return false;
        }
        self.rights.push(right);
        self.update_stats();
        true
    }

    /// Get right
    pub fn get_right(&self, id: &str) -> Option<&DomainRight> {
        self.rights.iter().find(|r| r.id == id)
    }

    /// Get right mut
    pub fn get_right_mut(&mut self, id: &str) -> Option<&mut DomainRight> {
        self.rights.iter_mut().find(|r| r.id == id)
    }

    /// Add holder
    pub fn add_holder(&mut self, holder: DomainHolder) {
        self.holders.push(holder);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.rights, self.config.domain_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DomainStats {
        &self.stats
    }

    /// Right count
    pub fn right_count(&self) -> usize {
        self.rights.len()
    }
}

/// Domain registry
#[derive(Debug, Clone, Default)]
pub struct DomainRegistry {
    /// Domains by ID
    domains: HashMap<String, SettingsDomain>,
}

impl DomainRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register domain
    pub fn register(&mut self, id: impl Into<String>, domain: SettingsDomain) {
        self.domains.insert(id.into(), domain);
    }

    /// Unregister domain
    pub fn unregister(&mut self, id: &str) -> bool {
        self.domains.remove(id).is_some()
    }

    /// Get domain
    pub fn get(&self, id: &str) -> Option<&SettingsDomain> {
        self.domains.get(id)
    }

    /// Get domain mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDomain> {
        self.domains.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.domains.len()
    }
}

/// Format domain registry
pub fn format_domain_registry(registry: &DomainRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Domain Registry:\n");
    output.push_str(&format!("  Domains: {}\n", registry.count()));
    output
}

/// Check if query is about domain
pub fn is_domain_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings domain") || lower.contains("domain settings") || lower.contains("sovereign domain")
}

/// Fun fact about domain
pub fn domain_fun_fact() -> &'static str {
    "Anna's settings domain establishes sovereign jurisdiction!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_type_display() {
        assert_eq!(format!("{}", DomainType::Public), "public");
        assert_eq!(format!("{}", DomainType::Private), "private");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", DomainStatus::Claimed), "claimed");
        assert_eq!(format!("{}", DomainStatus::Consolidated), "consolidated");
    }

    #[test]
    fn test_config_new() {
        let c = DomainConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DomainConfig::new("test")
            .domain_type(DomainType::Royal)
            .status(DomainStatus::Recognized);
        assert_eq!(c.domain_type, DomainType::Royal);
        assert_eq!(c.status, DomainStatus::Recognized);
    }

    #[test]
    fn test_right_new() {
        let r = DomainRight::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_right_builder() {
        let r = DomainRight::new("r1", "Title", "Content")
            .priority(1);
        assert_eq!(r.priority, 1);
    }

    #[test]
    fn test_right_exclusive() {
        let mut r = DomainRight::new("r1", "Title", "Content");
        r.make_shared();
        assert!(!r.exclusive);
        r.make_exclusive();
        assert!(r.exclusive);
    }

    #[test]
    fn test_holder_new() {
        let h = DomainHolder::new("key", "name", "r1");
        assert_eq!(h.right_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DomainStats::default();
        let right = DomainRight::new("r1", "Title", "Content");
        s.update(&[right], DomainType::Public);
        assert_eq!(s.total_rights, 1);
        assert_eq!(s.exclusive, 1);
    }

    #[test]
    fn test_domain_new() {
        let d = SettingsDomain::new(DomainConfig::default());
        assert_eq!(d.right_count(), 0);
    }

    #[test]
    fn test_domain_add_right() {
        let mut d = SettingsDomain::new(DomainConfig::default());
        d.add_right(DomainRight::new("r1", "Title", "Content"));
        assert_eq!(d.right_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DomainRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DomainRegistry::new();
        r.register("d1", SettingsDomain::new(DomainConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_domain_query() {
        assert!(is_domain_query("settings domain"));
        assert!(!is_domain_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = domain_fun_fact();
        assert!(fact.contains("domain"));
    }
}
