// v0.0.736: Settings Coalition (Phase 312)
// Temporary coalition for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Coalition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CoalitionType {
    /// Governing coalition
    #[default]
    Governing,
    /// Opposition coalition
    Opposition,
    /// Emergency coalition
    Emergency,
    /// Issue coalition
    Issue,
}

impl std::fmt::Display for CoalitionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Governing => write!(f, "governing"),
            Self::Opposition => write!(f, "opposition"),
            Self::Emergency => write!(f, "emergency"),
            Self::Issue => write!(f, "issue"),
        }
    }
}

/// Coalition status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CoalitionStatus {
    /// Forming status
    #[default]
    Forming,
    /// Stable status
    Stable,
    /// Unstable status
    Unstable,
    /// Collapsed status
    Collapsed,
}

impl std::fmt::Display for CoalitionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forming => write!(f, "forming"),
            Self::Stable => write!(f, "stable"),
            Self::Unstable => write!(f, "unstable"),
            Self::Collapsed => write!(f, "collapsed"),
        }
    }
}

/// Coalition config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalitionConfig {
    /// Name
    pub name: String,
    /// Coalition type
    pub coalition_type: CoalitionType,
    /// Status
    pub status: CoalitionStatus,
    /// Max agreements
    pub max_agreements: usize,
}

impl CoalitionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            coalition_type: CoalitionType::Governing,
            status: CoalitionStatus::Forming,
            max_agreements: 100,
        }
    }

    /// Set type
    pub fn coalition_type(mut self, ct: CoalitionType) -> Self {
        self.coalition_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CoalitionStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max agreements
    pub fn max_agreements(mut self, max: usize) -> Self {
        self.max_agreements = max;
        self
    }
}

impl Default for CoalitionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Coalition agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalitionAgreement {
    /// Agreement ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Priority
    pub priority: u32,
    /// Consensus
    pub consensus: bool,
}

impl CoalitionAgreement {
    /// Create new agreement
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            priority: 0,
            consensus: false,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: u32) -> Self {
        self.priority = p;
        self
    }

    /// Reach consensus
    pub fn reach_consensus(&mut self) {
        self.consensus = true;
    }

    /// Break consensus
    pub fn break_consensus(&mut self) {
        self.consensus = false;
    }
}

/// Coalition partner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalitionPartner {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Agreement ID
    pub agreement_id: String,
}

impl CoalitionPartner {
    /// Create new partner
    pub fn new(key: impl Into<String>, name: impl Into<String>, agreement_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            agreement_id: agreement_id.into(),
        }
    }
}

/// Coalition stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoalitionStats {
    /// Total agreements
    pub total_agreements: usize,
    /// Consensus agreements
    pub consensus: usize,
    /// Stable count
    pub stable_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CoalitionStats {
    /// Update from agreements
    pub fn update(&mut self, agreements: &[CoalitionAgreement], coalition_type: CoalitionType) {
        self.total_agreements = agreements.len();
        self.consensus = agreements.iter().filter(|a| a.consensus).count();
        *self.by_type.entry(coalition_type.to_string()).or_insert(0) += 1;
    }

    /// Consensus rate
    pub fn consensus_rate(&self) -> f64 {
        if self.total_agreements == 0 { 0.0 } else { self.consensus as f64 / self.total_agreements as f64 * 100.0 }
    }
}

/// Settings coalition
#[derive(Debug, Clone, Default)]
pub struct SettingsCoalition {
    /// Config
    config: CoalitionConfig,
    /// Agreements
    agreements: Vec<CoalitionAgreement>,
    /// Partners
    partners: Vec<CoalitionPartner>,
    /// Stats
    stats: CoalitionStats,
}

impl SettingsCoalition {
    /// Create new coalition system
    pub fn new(config: CoalitionConfig) -> Self {
        Self {
            config,
            agreements: Vec::new(),
            partners: Vec::new(),
            stats: CoalitionStats::default(),
        }
    }

    /// Add agreement
    pub fn add_agreement(&mut self, agreement: CoalitionAgreement) -> bool {
        if self.agreements.len() >= self.config.max_agreements {
            return false;
        }
        self.agreements.push(agreement);
        self.update_stats();
        true
    }

    /// Get agreement
    pub fn get_agreement(&self, id: &str) -> Option<&CoalitionAgreement> {
        self.agreements.iter().find(|a| a.id == id)
    }

    /// Get agreement mut
    pub fn get_agreement_mut(&mut self, id: &str) -> Option<&mut CoalitionAgreement> {
        self.agreements.iter_mut().find(|a| a.id == id)
    }

    /// Add partner
    pub fn add_partner(&mut self, partner: CoalitionPartner) {
        self.partners.push(partner);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.agreements, self.config.coalition_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CoalitionStats {
        &self.stats
    }

    /// Agreement count
    pub fn agreement_count(&self) -> usize {
        self.agreements.len()
    }
}

/// Coalition registry
#[derive(Debug, Clone, Default)]
pub struct CoalitionRegistry {
    /// Coalitions by ID
    coalitions: HashMap<String, SettingsCoalition>,
}

impl CoalitionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register coalition
    pub fn register(&mut self, id: impl Into<String>, coalition: SettingsCoalition) {
        self.coalitions.insert(id.into(), coalition);
    }

    /// Unregister coalition
    pub fn unregister(&mut self, id: &str) -> bool {
        self.coalitions.remove(id).is_some()
    }

    /// Get coalition
    pub fn get(&self, id: &str) -> Option<&SettingsCoalition> {
        self.coalitions.get(id)
    }

    /// Get coalition mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCoalition> {
        self.coalitions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.coalitions.len()
    }
}

/// Format coalition registry
pub fn format_coalition_registry(registry: &CoalitionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Coalition Registry:\n");
    output.push_str(&format!("  Coalitions: {}\n", registry.count()));
    output
}

/// Check if query is about coalition
pub fn is_coalition_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings coalition") || lower.contains("coalition settings") || lower.contains("temporary alliance")
}

/// Fun fact about coalition
pub fn coalition_fun_fact() -> &'static str {
    "Anna's settings coalition establishes temporary governance partnerships!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coalition_type_display() {
        assert_eq!(format!("{}", CoalitionType::Governing), "governing");
        assert_eq!(format!("{}", CoalitionType::Opposition), "opposition");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CoalitionStatus::Forming), "forming");
        assert_eq!(format!("{}", CoalitionStatus::Stable), "stable");
    }

    #[test]
    fn test_config_new() {
        let c = CoalitionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CoalitionConfig::new("test")
            .coalition_type(CoalitionType::Opposition)
            .status(CoalitionStatus::Stable);
        assert_eq!(c.coalition_type, CoalitionType::Opposition);
        assert_eq!(c.status, CoalitionStatus::Stable);
    }

    #[test]
    fn test_agreement_new() {
        let a = CoalitionAgreement::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_agreement_builder() {
        let a = CoalitionAgreement::new("a1", "Title", "Content")
            .priority(1);
        assert_eq!(a.priority, 1);
    }

    #[test]
    fn test_agreement_consensus() {
        let mut a = CoalitionAgreement::new("a1", "Title", "Content");
        a.reach_consensus();
        assert!(a.consensus);
        a.break_consensus();
        assert!(!a.consensus);
    }

    #[test]
    fn test_partner_new() {
        let p = CoalitionPartner::new("key", "name", "a1");
        assert_eq!(p.agreement_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CoalitionStats::default();
        let mut agreement = CoalitionAgreement::new("a1", "Title", "Content");
        agreement.reach_consensus();
        s.update(&[agreement], CoalitionType::Governing);
        assert_eq!(s.total_agreements, 1);
        assert_eq!(s.consensus, 1);
    }

    #[test]
    fn test_coalition_new() {
        let c = SettingsCoalition::new(CoalitionConfig::default());
        assert_eq!(c.agreement_count(), 0);
    }

    #[test]
    fn test_coalition_add_agreement() {
        let mut c = SettingsCoalition::new(CoalitionConfig::default());
        c.add_agreement(CoalitionAgreement::new("a1", "Title", "Content"));
        assert_eq!(c.agreement_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CoalitionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CoalitionRegistry::new();
        r.register("c1", SettingsCoalition::new(CoalitionConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_coalition_query() {
        assert!(is_coalition_query("settings coalition"));
        assert!(!is_coalition_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = coalition_fun_fact();
        assert!(fact.contains("coalition"));
    }
}
