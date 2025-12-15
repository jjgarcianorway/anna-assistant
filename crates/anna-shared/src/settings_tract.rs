// v0.0.759: Settings Tract (Phase 335)
// Land tract for settings territory

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tract type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TractType {
    /// Residential tract
    #[default]
    Residential,
    /// Commercial tract
    Commercial,
    /// Agricultural tract
    Agricultural,
    /// Wilderness tract
    Wilderness,
}

impl std::fmt::Display for TractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Residential => write!(f, "residential"),
            Self::Commercial => write!(f, "commercial"),
            Self::Agricultural => write!(f, "agricultural"),
            Self::Wilderness => write!(f, "wilderness"),
        }
    }
}

/// Tract status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TractStatus {
    /// Surveyed status
    #[default]
    Surveyed,
    /// Developed status
    Developed,
    /// Preserved status
    Preserved,
    /// Disputed status
    Disputed,
}

impl std::fmt::Display for TractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surveyed => write!(f, "surveyed"),
            Self::Developed => write!(f, "developed"),
            Self::Preserved => write!(f, "preserved"),
            Self::Disputed => write!(f, "disputed"),
        }
    }
}

/// Tract config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TractConfig {
    /// Name
    pub name: String,
    /// Tract type
    pub tract_type: TractType,
    /// Status
    pub status: TractStatus,
    /// Max grants
    pub max_grants: usize,
}

impl TractConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tract_type: TractType::Residential,
            status: TractStatus::Surveyed,
            max_grants: 100,
        }
    }

    /// Set type
    pub fn tract_type(mut self, tt: TractType) -> Self {
        self.tract_type = tt;
        self
    }

    /// Set status
    pub fn status(mut self, s: TractStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max grants
    pub fn max_grants(mut self, max: usize) -> Self {
        self.max_grants = max;
        self
    }
}

impl Default for TractConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Tract grant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TractGrant {
    /// Grant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Range number
    pub range: u32,
    /// Patented
    pub patented: bool,
}

impl TractGrant {
    /// Create new grant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            range: 0,
            patented: true,
        }
    }

    /// Set range
    pub fn range(mut self, r: u32) -> Self {
        self.range = r;
        self
    }

    /// Make patented
    pub fn make_patented(&mut self) {
        self.patented = true;
    }

    /// Make pending
    pub fn make_pending(&mut self) {
        self.patented = false;
    }
}

/// Tract ranger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TractRanger {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Grant ID
    pub grant_id: String,
}

impl TractRanger {
    /// Create new ranger
    pub fn new(key: impl Into<String>, name: impl Into<String>, grant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            grant_id: grant_id.into(),
        }
    }
}

/// Tract stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TractStats {
    /// Total grants
    pub total_grants: usize,
    /// Patented grants
    pub patented: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TractStats {
    /// Update from grants
    pub fn update(&mut self, grants: &[TractGrant], tract_type: TractType) {
        self.total_grants = grants.len();
        self.patented = grants.iter().filter(|g| g.patented).count();
        *self.by_type.entry(tract_type.to_string()).or_insert(0) += 1;
    }

    /// Patented rate
    pub fn patented_rate(&self) -> f64 {
        if self.total_grants == 0 { 0.0 } else { self.patented as f64 / self.total_grants as f64 * 100.0 }
    }
}

/// Settings tract
#[derive(Debug, Clone, Default)]
pub struct SettingsTract {
    /// Config
    config: TractConfig,
    /// Grants
    grants: Vec<TractGrant>,
    /// Rangers
    rangers: Vec<TractRanger>,
    /// Stats
    stats: TractStats,
}

impl SettingsTract {
    /// Create new tract system
    pub fn new(config: TractConfig) -> Self {
        Self {
            config,
            grants: Vec::new(),
            rangers: Vec::new(),
            stats: TractStats::default(),
        }
    }

    /// Add grant
    pub fn add_grant(&mut self, grant: TractGrant) -> bool {
        if self.grants.len() >= self.config.max_grants {
            return false;
        }
        self.grants.push(grant);
        self.update_stats();
        true
    }

    /// Get grant
    pub fn get_grant(&self, id: &str) -> Option<&TractGrant> {
        self.grants.iter().find(|g| g.id == id)
    }

    /// Get grant mut
    pub fn get_grant_mut(&mut self, id: &str) -> Option<&mut TractGrant> {
        self.grants.iter_mut().find(|g| g.id == id)
    }

    /// Add ranger
    pub fn add_ranger(&mut self, ranger: TractRanger) {
        self.rangers.push(ranger);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.grants, self.config.tract_type);
    }

    /// Get stats
    pub fn stats(&self) -> &TractStats {
        &self.stats
    }

    /// Grant count
    pub fn grant_count(&self) -> usize {
        self.grants.len()
    }
}

/// Tract registry
#[derive(Debug, Clone, Default)]
pub struct TractRegistry {
    /// Tracts by ID
    tracts: HashMap<String, SettingsTract>,
}

impl TractRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register tract
    pub fn register(&mut self, id: impl Into<String>, tract: SettingsTract) {
        self.tracts.insert(id.into(), tract);
    }

    /// Unregister tract
    pub fn unregister(&mut self, id: &str) -> bool {
        self.tracts.remove(id).is_some()
    }

    /// Get tract
    pub fn get(&self, id: &str) -> Option<&SettingsTract> {
        self.tracts.get(id)
    }

    /// Get tract mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTract> {
        self.tracts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.tracts.len()
    }
}

/// Format tract registry
pub fn format_tract_registry(registry: &TractRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Tract Registry:\n");
    output.push_str(&format!("  Tracts: {}\n", registry.count()));
    output
}

/// Check if query is about tract
pub fn is_tract_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings tract") || lower.contains("tract settings") || lower.contains("land tract")
}

/// Fun fact about tract
pub fn tract_fun_fact() -> &'static str {
    "Anna's settings tract establishes territory boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tract_type_display() {
        assert_eq!(format!("{}", TractType::Residential), "residential");
        assert_eq!(format!("{}", TractType::Agricultural), "agricultural");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TractStatus::Surveyed), "surveyed");
        assert_eq!(format!("{}", TractStatus::Preserved), "preserved");
    }

    #[test]
    fn test_config_new() {
        let c = TractConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = TractConfig::new("test")
            .tract_type(TractType::Wilderness)
            .status(TractStatus::Disputed);
        assert_eq!(c.tract_type, TractType::Wilderness);
        assert_eq!(c.status, TractStatus::Disputed);
    }

    #[test]
    fn test_grant_new() {
        let g = TractGrant::new("g1", "Title", "Content");
        assert_eq!(g.id, "g1");
    }

    #[test]
    fn test_grant_builder() {
        let g = TractGrant::new("g1", "Title", "Content")
            .range(1);
        assert_eq!(g.range, 1);
    }

    #[test]
    fn test_grant_patented() {
        let mut g = TractGrant::new("g1", "Title", "Content");
        g.make_pending();
        assert!(!g.patented);
        g.make_patented();
        assert!(g.patented);
    }

    #[test]
    fn test_ranger_new() {
        let r = TractRanger::new("key", "name", "g1");
        assert_eq!(r.grant_id, "g1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = TractStats::default();
        let grant = TractGrant::new("g1", "Title", "Content");
        s.update(&[grant], TractType::Residential);
        assert_eq!(s.total_grants, 1);
        assert_eq!(s.patented, 1);
    }

    #[test]
    fn test_tract_new() {
        let t = SettingsTract::new(TractConfig::default());
        assert_eq!(t.grant_count(), 0);
    }

    #[test]
    fn test_tract_add_grant() {
        let mut t = SettingsTract::new(TractConfig::default());
        t.add_grant(TractGrant::new("g1", "Title", "Content"));
        assert_eq!(t.grant_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = TractRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TractRegistry::new();
        r.register("t1", SettingsTract::new(TractConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_tract_query() {
        assert!(is_tract_query("settings tract"));
        assert!(!is_tract_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = tract_fun_fact();
        assert!(fact.contains("tract"));
    }
}
