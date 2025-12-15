// v0.0.747: Settings Region (Phase 323)
// Geographic region for settings organization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Region type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RegionType {
    /// Administrative region
    #[default]
    Administrative,
    /// Economic region
    Economic,
    /// Cultural region
    Cultural,
    /// Geographic region
    Geographic,
}

impl std::fmt::Display for RegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Administrative => write!(f, "administrative"),
            Self::Economic => write!(f, "economic"),
            Self::Cultural => write!(f, "cultural"),
            Self::Geographic => write!(f, "geographic"),
        }
    }
}

/// Region status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RegionStatus {
    /// Defined status
    #[default]
    Defined,
    /// Active status
    Active,
    /// Expanding status
    Expanding,
    /// Contracting status
    Contracting,
}

impl std::fmt::Display for RegionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defined => write!(f, "defined"),
            Self::Active => write!(f, "active"),
            Self::Expanding => write!(f, "expanding"),
            Self::Contracting => write!(f, "contracting"),
        }
    }
}

/// Region config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    /// Name
    pub name: String,
    /// Region type
    pub region_type: RegionType,
    /// Status
    pub status: RegionStatus,
    /// Max policies
    pub max_policies: usize,
}

impl RegionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            region_type: RegionType::Administrative,
            status: RegionStatus::Defined,
            max_policies: 100,
        }
    }

    /// Set type
    pub fn region_type(mut self, rt: RegionType) -> Self {
        self.region_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: RegionStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max policies
    pub fn max_policies(mut self, max: usize) -> Self {
        self.max_policies = max;
        self
    }
}

impl Default for RegionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Region policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionPolicy {
    /// Policy ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Sector number
    pub sector: u32,
    /// Regional
    pub regional: bool,
}

impl RegionPolicy {
    /// Create new policy
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            sector: 0,
            regional: true,
        }
    }

    /// Set sector
    pub fn sector(mut self, s: u32) -> Self {
        self.sector = s;
        self
    }

    /// Make regional
    pub fn make_regional(&mut self) {
        self.regional = true;
    }

    /// Make local
    pub fn make_local(&mut self) {
        self.regional = false;
    }
}

/// Region coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCoordinator {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Policy ID
    pub policy_id: String,
}

impl RegionCoordinator {
    /// Create new coordinator
    pub fn new(key: impl Into<String>, name: impl Into<String>, policy_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            policy_id: policy_id.into(),
        }
    }
}

/// Region stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegionStats {
    /// Total policies
    pub total_policies: usize,
    /// Regional policies
    pub regional: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl RegionStats {
    /// Update from policies
    pub fn update(&mut self, policies: &[RegionPolicy], region_type: RegionType) {
        self.total_policies = policies.len();
        self.regional = policies.iter().filter(|p| p.regional).count();
        *self.by_type.entry(region_type.to_string()).or_insert(0) += 1;
    }

    /// Regional rate
    pub fn regional_rate(&self) -> f64 {
        if self.total_policies == 0 { 0.0 } else { self.regional as f64 / self.total_policies as f64 * 100.0 }
    }
}

/// Settings region
#[derive(Debug, Clone, Default)]
pub struct SettingsRegion {
    /// Config
    config: RegionConfig,
    /// Policies
    policies: Vec<RegionPolicy>,
    /// Coordinators
    coordinators: Vec<RegionCoordinator>,
    /// Stats
    stats: RegionStats,
}

impl SettingsRegion {
    /// Create new region system
    pub fn new(config: RegionConfig) -> Self {
        Self {
            config,
            policies: Vec::new(),
            coordinators: Vec::new(),
            stats: RegionStats::default(),
        }
    }

    /// Add policy
    pub fn add_policy(&mut self, policy: RegionPolicy) -> bool {
        if self.policies.len() >= self.config.max_policies {
            return false;
        }
        self.policies.push(policy);
        self.update_stats();
        true
    }

    /// Get policy
    pub fn get_policy(&self, id: &str) -> Option<&RegionPolicy> {
        self.policies.iter().find(|p| p.id == id)
    }

    /// Get policy mut
    pub fn get_policy_mut(&mut self, id: &str) -> Option<&mut RegionPolicy> {
        self.policies.iter_mut().find(|p| p.id == id)
    }

    /// Add coordinator
    pub fn add_coordinator(&mut self, coordinator: RegionCoordinator) {
        self.coordinators.push(coordinator);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.policies, self.config.region_type);
    }

    /// Get stats
    pub fn stats(&self) -> &RegionStats {
        &self.stats
    }

    /// Policy count
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }
}

/// Region registry
#[derive(Debug, Clone, Default)]
pub struct RegionRegistry {
    /// Regions by ID
    regions: HashMap<String, SettingsRegion>,
}

impl RegionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register region
    pub fn register(&mut self, id: impl Into<String>, region: SettingsRegion) {
        self.regions.insert(id.into(), region);
    }

    /// Unregister region
    pub fn unregister(&mut self, id: &str) -> bool {
        self.regions.remove(id).is_some()
    }

    /// Get region
    pub fn get(&self, id: &str) -> Option<&SettingsRegion> {
        self.regions.get(id)
    }

    /// Get region mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRegion> {
        self.regions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.regions.len()
    }
}

/// Format region registry
pub fn format_region_registry(registry: &RegionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Region Registry:\n");
    output.push_str(&format!("  Regions: {}\n", registry.count()));
    output
}

/// Check if query is about region
pub fn is_region_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings region") || lower.contains("region settings") || lower.contains("geographic region")
}

/// Fun fact about region
pub fn region_fun_fact() -> &'static str {
    "Anna's settings region establishes geographic organization!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_type_display() {
        assert_eq!(format!("{}", RegionType::Administrative), "administrative");
        assert_eq!(format!("{}", RegionType::Economic), "economic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RegionStatus::Defined), "defined");
        assert_eq!(format!("{}", RegionStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = RegionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RegionConfig::new("test")
            .region_type(RegionType::Cultural)
            .status(RegionStatus::Expanding);
        assert_eq!(c.region_type, RegionType::Cultural);
        assert_eq!(c.status, RegionStatus::Expanding);
    }

    #[test]
    fn test_policy_new() {
        let p = RegionPolicy::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_policy_builder() {
        let p = RegionPolicy::new("p1", "Title", "Content")
            .sector(1);
        assert_eq!(p.sector, 1);
    }

    #[test]
    fn test_policy_regional() {
        let mut p = RegionPolicy::new("p1", "Title", "Content");
        p.make_local();
        assert!(!p.regional);
        p.make_regional();
        assert!(p.regional);
    }

    #[test]
    fn test_coordinator_new() {
        let c = RegionCoordinator::new("key", "name", "p1");
        assert_eq!(c.policy_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = RegionStats::default();
        let policy = RegionPolicy::new("p1", "Title", "Content");
        s.update(&[policy], RegionType::Administrative);
        assert_eq!(s.total_policies, 1);
        assert_eq!(s.regional, 1);
    }

    #[test]
    fn test_region_new() {
        let r = SettingsRegion::new(RegionConfig::default());
        assert_eq!(r.policy_count(), 0);
    }

    #[test]
    fn test_region_add_policy() {
        let mut r = SettingsRegion::new(RegionConfig::default());
        r.add_policy(RegionPolicy::new("p1", "Title", "Content"));
        assert_eq!(r.policy_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = RegionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RegionRegistry::new();
        r.register("r1", SettingsRegion::new(RegionConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_region_query() {
        assert!(is_region_query("settings region"));
        assert!(!is_region_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = region_fun_fact();
        assert!(fact.contains("region"));
    }
}
