// v0.0.740: Settings Bloc (Phase 316)
// Regional bloc for settings coordination

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bloc type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlocType {
    /// Trading bloc
    #[default]
    Trading,
    /// Voting bloc
    Voting,
    /// Power bloc
    Power,
    /// Regional bloc
    Regional,
}

impl std::fmt::Display for BlocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trading => write!(f, "trading"),
            Self::Voting => write!(f, "voting"),
            Self::Power => write!(f, "power"),
            Self::Regional => write!(f, "regional"),
        }
    }
}

/// Bloc status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BlocStatus {
    /// Forming status
    #[default]
    Forming,
    /// Active status
    Active,
    /// Dominant status
    Dominant,
    /// Fragmented status
    Fragmented,
}

impl std::fmt::Display for BlocStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forming => write!(f, "forming"),
            Self::Active => write!(f, "active"),
            Self::Dominant => write!(f, "dominant"),
            Self::Fragmented => write!(f, "fragmented"),
        }
    }
}

/// Bloc config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocConfig {
    /// Name
    pub name: String,
    /// Bloc type
    pub bloc_type: BlocType,
    /// Status
    pub status: BlocStatus,
    /// Max policies
    pub max_policies: usize,
}

impl BlocConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bloc_type: BlocType::Trading,
            status: BlocStatus::Forming,
            max_policies: 100,
        }
    }

    /// Set type
    pub fn bloc_type(mut self, bt: BlocType) -> Self {
        self.bloc_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: BlocStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max policies
    pub fn max_policies(mut self, max: usize) -> Self {
        self.max_policies = max;
        self
    }
}

impl Default for BlocConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Bloc policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocPolicy {
    /// Policy ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Priority level
    pub priority: u32,
    /// Coordinated
    pub coordinated: bool,
}

impl BlocPolicy {
    /// Create new policy
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            priority: 0,
            coordinated: true,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: u32) -> Self {
        self.priority = p;
        self
    }

    /// Make coordinated
    pub fn make_coordinated(&mut self) {
        self.coordinated = true;
    }

    /// Make independent
    pub fn make_independent(&mut self) {
        self.coordinated = false;
    }
}

/// Bloc member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Policy ID
    pub policy_id: String,
}

impl BlocMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, policy_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            policy_id: policy_id.into(),
        }
    }
}

/// Bloc stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlocStats {
    /// Total policies
    pub total_policies: usize,
    /// Coordinated policies
    pub coordinated: usize,
    /// Dominant count
    pub dominant_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BlocStats {
    /// Update from policies
    pub fn update(&mut self, policies: &[BlocPolicy], bloc_type: BlocType) {
        self.total_policies = policies.len();
        self.coordinated = policies.iter().filter(|p| p.coordinated).count();
        *self.by_type.entry(bloc_type.to_string()).or_insert(0) += 1;
    }

    /// Coordination rate
    pub fn coordination_rate(&self) -> f64 {
        if self.total_policies == 0 { 0.0 } else { self.coordinated as f64 / self.total_policies as f64 * 100.0 }
    }
}

/// Settings bloc
#[derive(Debug, Clone, Default)]
pub struct SettingsBloc {
    /// Config
    config: BlocConfig,
    /// Policies
    policies: Vec<BlocPolicy>,
    /// Members
    members: Vec<BlocMember>,
    /// Stats
    stats: BlocStats,
}

impl SettingsBloc {
    /// Create new bloc system
    pub fn new(config: BlocConfig) -> Self {
        Self {
            config,
            policies: Vec::new(),
            members: Vec::new(),
            stats: BlocStats::default(),
        }
    }

    /// Add policy
    pub fn add_policy(&mut self, policy: BlocPolicy) -> bool {
        if self.policies.len() >= self.config.max_policies {
            return false;
        }
        self.policies.push(policy);
        self.update_stats();
        true
    }

    /// Get policy
    pub fn get_policy(&self, id: &str) -> Option<&BlocPolicy> {
        self.policies.iter().find(|p| p.id == id)
    }

    /// Get policy mut
    pub fn get_policy_mut(&mut self, id: &str) -> Option<&mut BlocPolicy> {
        self.policies.iter_mut().find(|p| p.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: BlocMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.policies, self.config.bloc_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BlocStats {
        &self.stats
    }

    /// Policy count
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }
}

/// Bloc registry
#[derive(Debug, Clone, Default)]
pub struct BlocRegistry {
    /// Blocs by ID
    blocs: HashMap<String, SettingsBloc>,
}

impl BlocRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register bloc
    pub fn register(&mut self, id: impl Into<String>, bloc: SettingsBloc) {
        self.blocs.insert(id.into(), bloc);
    }

    /// Unregister bloc
    pub fn unregister(&mut self, id: &str) -> bool {
        self.blocs.remove(id).is_some()
    }

    /// Get bloc
    pub fn get(&self, id: &str) -> Option<&SettingsBloc> {
        self.blocs.get(id)
    }

    /// Get bloc mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBloc> {
        self.blocs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.blocs.len()
    }
}

/// Format bloc registry
pub fn format_bloc_registry(registry: &BlocRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Bloc Registry:\n");
    output.push_str(&format!("  Blocs: {}\n", registry.count()));
    output
}

/// Check if query is about bloc
pub fn is_bloc_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings bloc") || lower.contains("bloc settings") || lower.contains("regional bloc")
}

/// Fun fact about bloc
pub fn bloc_fun_fact() -> &'static str {
    "Anna's settings bloc establishes regional coordination!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloc_type_display() {
        assert_eq!(format!("{}", BlocType::Trading), "trading");
        assert_eq!(format!("{}", BlocType::Voting), "voting");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BlocStatus::Forming), "forming");
        assert_eq!(format!("{}", BlocStatus::Dominant), "dominant");
    }

    #[test]
    fn test_config_new() {
        let c = BlocConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BlocConfig::new("test")
            .bloc_type(BlocType::Power)
            .status(BlocStatus::Active);
        assert_eq!(c.bloc_type, BlocType::Power);
        assert_eq!(c.status, BlocStatus::Active);
    }

    #[test]
    fn test_policy_new() {
        let p = BlocPolicy::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_policy_builder() {
        let p = BlocPolicy::new("p1", "Title", "Content")
            .priority(1);
        assert_eq!(p.priority, 1);
    }

    #[test]
    fn test_policy_coordinated() {
        let mut p = BlocPolicy::new("p1", "Title", "Content");
        p.make_independent();
        assert!(!p.coordinated);
        p.make_coordinated();
        assert!(p.coordinated);
    }

    #[test]
    fn test_member_new() {
        let m = BlocMember::new("key", "name", "p1");
        assert_eq!(m.policy_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BlocStats::default();
        let policy = BlocPolicy::new("p1", "Title", "Content");
        s.update(&[policy], BlocType::Trading);
        assert_eq!(s.total_policies, 1);
        assert_eq!(s.coordinated, 1);
    }

    #[test]
    fn test_bloc_new() {
        let b = SettingsBloc::new(BlocConfig::default());
        assert_eq!(b.policy_count(), 0);
    }

    #[test]
    fn test_bloc_add_policy() {
        let mut b = SettingsBloc::new(BlocConfig::default());
        b.add_policy(BlocPolicy::new("p1", "Title", "Content"));
        assert_eq!(b.policy_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BlocRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BlocRegistry::new();
        r.register("b1", SettingsBloc::new(BlocConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_bloc_query() {
        assert!(is_bloc_query("settings bloc"));
        assert!(!is_bloc_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = bloc_fun_fact();
        assert!(fact.contains("bloc"));
    }
}
