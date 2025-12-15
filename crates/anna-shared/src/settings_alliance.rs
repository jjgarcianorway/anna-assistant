// v0.0.735: Settings Alliance (Phase 311)
// Formal alliance for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Alliance type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AllianceType {
    /// Military alliance
    #[default]
    Military,
    /// Economic alliance
    Economic,
    /// Political alliance
    Political,
    /// Strategic alliance
    Strategic,
}

impl std::fmt::Display for AllianceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Military => write!(f, "military"),
            Self::Economic => write!(f, "economic"),
            Self::Political => write!(f, "political"),
            Self::Strategic => write!(f, "strategic"),
        }
    }
}

/// Alliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AllianceStatus {
    /// Forming status
    #[default]
    Forming,
    /// Active status
    Active,
    /// Strained status
    Strained,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for AllianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forming => write!(f, "forming"),
            Self::Active => write!(f, "active"),
            Self::Strained => write!(f, "strained"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}

/// Alliance config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceConfig {
    /// Name
    pub name: String,
    /// Alliance type
    pub alliance_type: AllianceType,
    /// Status
    pub status: AllianceStatus,
    /// Max commitments
    pub max_commitments: usize,
}

impl AllianceConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alliance_type: AllianceType::Military,
            status: AllianceStatus::Forming,
            max_commitments: 100,
        }
    }

    /// Set type
    pub fn alliance_type(mut self, at: AllianceType) -> Self {
        self.alliance_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AllianceStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max commitments
    pub fn max_commitments(mut self, max: usize) -> Self {
        self.max_commitments = max;
        self
    }
}

impl Default for AllianceConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Alliance commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceCommitment {
    /// Commitment ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub article: u32,
    /// Binding
    pub binding: bool,
}

impl AllianceCommitment {
    /// Create new commitment
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            article: 0,
            binding: true,
        }
    }

    /// Set article
    pub fn article(mut self, a: u32) -> Self {
        self.article = a;
        self
    }

    /// Make binding
    pub fn make_binding(&mut self) {
        self.binding = true;
    }

    /// Make optional
    pub fn make_optional(&mut self) {
        self.binding = false;
    }
}

/// Alliance member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Commitment ID
    pub commitment_id: String,
}

impl AllianceMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, commitment_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            commitment_id: commitment_id.into(),
        }
    }
}

/// Alliance stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllianceStats {
    /// Total commitments
    pub total_commitments: usize,
    /// Binding commitments
    pub binding: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AllianceStats {
    /// Update from commitments
    pub fn update(&mut self, commitments: &[AllianceCommitment], alliance_type: AllianceType) {
        self.total_commitments = commitments.len();
        self.binding = commitments.iter().filter(|c| c.binding).count();
        *self.by_type.entry(alliance_type.to_string()).or_insert(0) += 1;
    }

    /// Binding rate
    pub fn binding_rate(&self) -> f64 {
        if self.total_commitments == 0 { 0.0 } else { self.binding as f64 / self.total_commitments as f64 * 100.0 }
    }
}

/// Settings alliance
#[derive(Debug, Clone, Default)]
pub struct SettingsAlliance {
    /// Config
    config: AllianceConfig,
    /// Commitments
    commitments: Vec<AllianceCommitment>,
    /// Members
    members: Vec<AllianceMember>,
    /// Stats
    stats: AllianceStats,
}

impl SettingsAlliance {
    /// Create new alliance system
    pub fn new(config: AllianceConfig) -> Self {
        Self {
            config,
            commitments: Vec::new(),
            members: Vec::new(),
            stats: AllianceStats::default(),
        }
    }

    /// Add commitment
    pub fn add_commitment(&mut self, commitment: AllianceCommitment) -> bool {
        if self.commitments.len() >= self.config.max_commitments {
            return false;
        }
        self.commitments.push(commitment);
        self.update_stats();
        true
    }

    /// Get commitment
    pub fn get_commitment(&self, id: &str) -> Option<&AllianceCommitment> {
        self.commitments.iter().find(|c| c.id == id)
    }

    /// Get commitment mut
    pub fn get_commitment_mut(&mut self, id: &str) -> Option<&mut AllianceCommitment> {
        self.commitments.iter_mut().find(|c| c.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: AllianceMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.commitments, self.config.alliance_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AllianceStats {
        &self.stats
    }

    /// Commitment count
    pub fn commitment_count(&self) -> usize {
        self.commitments.len()
    }
}

/// Alliance registry
#[derive(Debug, Clone, Default)]
pub struct AllianceRegistry {
    /// Alliances by ID
    alliances: HashMap<String, SettingsAlliance>,
}

impl AllianceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register alliance
    pub fn register(&mut self, id: impl Into<String>, alliance: SettingsAlliance) {
        self.alliances.insert(id.into(), alliance);
    }

    /// Unregister alliance
    pub fn unregister(&mut self, id: &str) -> bool {
        self.alliances.remove(id).is_some()
    }

    /// Get alliance
    pub fn get(&self, id: &str) -> Option<&SettingsAlliance> {
        self.alliances.get(id)
    }

    /// Get alliance mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAlliance> {
        self.alliances.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.alliances.len()
    }
}

/// Format alliance registry
pub fn format_alliance_registry(registry: &AllianceRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Alliance Registry:\n");
    output.push_str(&format!("  Alliances: {}\n", registry.count()));
    output
}

/// Check if query is about alliance
pub fn is_alliance_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings alliance") || lower.contains("alliance settings") || lower.contains("mutual defense")
}

/// Fun fact about alliance
pub fn alliance_fun_fact() -> &'static str {
    "Anna's settings alliance establishes mutual governance commitments!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alliance_type_display() {
        assert_eq!(format!("{}", AllianceType::Military), "military");
        assert_eq!(format!("{}", AllianceType::Economic), "economic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AllianceStatus::Forming), "forming");
        assert_eq!(format!("{}", AllianceStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = AllianceConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AllianceConfig::new("test")
            .alliance_type(AllianceType::Economic)
            .status(AllianceStatus::Active);
        assert_eq!(c.alliance_type, AllianceType::Economic);
        assert_eq!(c.status, AllianceStatus::Active);
    }

    #[test]
    fn test_commitment_new() {
        let c = AllianceCommitment::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_commitment_builder() {
        let c = AllianceCommitment::new("c1", "Title", "Content")
            .article(1);
        assert_eq!(c.article, 1);
    }

    #[test]
    fn test_commitment_binding() {
        let mut c = AllianceCommitment::new("c1", "Title", "Content");
        c.make_optional();
        assert!(!c.binding);
        c.make_binding();
        assert!(c.binding);
    }

    #[test]
    fn test_member_new() {
        let m = AllianceMember::new("key", "name", "c1");
        assert_eq!(m.commitment_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AllianceStats::default();
        let commitment = AllianceCommitment::new("c1", "Title", "Content");
        s.update(&[commitment], AllianceType::Military);
        assert_eq!(s.total_commitments, 1);
        assert_eq!(s.binding, 1);
    }

    #[test]
    fn test_alliance_new() {
        let a = SettingsAlliance::new(AllianceConfig::default());
        assert_eq!(a.commitment_count(), 0);
    }

    #[test]
    fn test_alliance_add_commitment() {
        let mut a = SettingsAlliance::new(AllianceConfig::default());
        a.add_commitment(AllianceCommitment::new("c1", "Title", "Content"));
        assert_eq!(a.commitment_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AllianceRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AllianceRegistry::new();
        r.register("a1", SettingsAlliance::new(AllianceConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_alliance_query() {
        assert!(is_alliance_query("settings alliance"));
        assert!(!is_alliance_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = alliance_fun_fact();
        assert!(fact.contains("alliance"));
    }
}
