// v0.0.728: Settings Protocol (Phase 304)
// Formal protocol for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProtocolType {
    /// Amendment protocol
    #[default]
    Amendment,
    /// Optional protocol
    Optional,
    /// Supplementary protocol
    Supplementary,
    /// Implementation protocol
    Implementation,
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Amendment => write!(f, "amendment"),
            Self::Optional => write!(f, "optional"),
            Self::Supplementary => write!(f, "supplementary"),
            Self::Implementation => write!(f, "implementation"),
        }
    }
}

/// Protocol status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProtocolStatus {
    /// Draft status
    #[default]
    Draft,
    /// Open status
    Open,
    /// Active status
    Active,
    /// Closed status
    Closed,
}

impl std::fmt::Display for ProtocolStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Open => write!(f, "open"),
            Self::Active => write!(f, "active"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

/// Protocol config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Name
    pub name: String,
    /// Protocol type
    pub protocol_type: ProtocolType,
    /// Status
    pub status: ProtocolStatus,
    /// Max clauses
    pub max_clauses: usize,
}

impl ProtocolConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            protocol_type: ProtocolType::Amendment,
            status: ProtocolStatus::Draft,
            max_clauses: 100,
        }
    }

    /// Set type
    pub fn protocol_type(mut self, pt: ProtocolType) -> Self {
        self.protocol_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ProtocolStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max clauses
    pub fn max_clauses(mut self, max: usize) -> Self {
        self.max_clauses = max;
        self
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Protocol clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolClause {
    /// Clause ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Adopted
    pub adopted: bool,
}

impl ProtocolClause {
    /// Create new clause
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            adopted: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Adopt clause
    pub fn adopt(&mut self) {
        self.adopted = true;
    }

    /// Reject clause
    pub fn reject(&mut self) {
        self.adopted = false;
    }
}

/// Protocol party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolParty {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Clause ID
    pub clause_id: String,
}

impl ProtocolParty {
    /// Create new party
    pub fn new(key: impl Into<String>, name: impl Into<String>, clause_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            clause_id: clause_id.into(),
        }
    }
}

/// Protocol stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolStats {
    /// Total clauses
    pub total_clauses: usize,
    /// Adopted clauses
    pub adopted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ProtocolStats {
    /// Update from clauses
    pub fn update(&mut self, clauses: &[ProtocolClause], protocol_type: ProtocolType) {
        self.total_clauses = clauses.len();
        self.adopted = clauses.iter().filter(|c| c.adopted).count();
        *self.by_type.entry(protocol_type.to_string()).or_insert(0) += 1;
    }

    /// Adoption rate
    pub fn adoption_rate(&self) -> f64 {
        if self.total_clauses == 0 { 0.0 } else { self.adopted as f64 / self.total_clauses as f64 * 100.0 }
    }
}

/// Settings protocol
#[derive(Debug, Clone, Default)]
pub struct SettingsProtocol {
    /// Config
    config: ProtocolConfig,
    /// Clauses
    clauses: Vec<ProtocolClause>,
    /// Parties
    parties: Vec<ProtocolParty>,
    /// Stats
    stats: ProtocolStats,
}

impl SettingsProtocol {
    /// Create new protocol system
    pub fn new(config: ProtocolConfig) -> Self {
        Self {
            config,
            clauses: Vec::new(),
            parties: Vec::new(),
            stats: ProtocolStats::default(),
        }
    }

    /// Add clause
    pub fn add_clause(&mut self, clause: ProtocolClause) -> bool {
        if self.clauses.len() >= self.config.max_clauses {
            return false;
        }
        self.clauses.push(clause);
        self.update_stats();
        true
    }

    /// Get clause
    pub fn get_clause(&self, id: &str) -> Option<&ProtocolClause> {
        self.clauses.iter().find(|c| c.id == id)
    }

    /// Get clause mut
    pub fn get_clause_mut(&mut self, id: &str) -> Option<&mut ProtocolClause> {
        self.clauses.iter_mut().find(|c| c.id == id)
    }

    /// Add party
    pub fn add_party(&mut self, party: ProtocolParty) {
        self.parties.push(party);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.clauses, self.config.protocol_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ProtocolStats {
        &self.stats
    }

    /// Clause count
    pub fn clause_count(&self) -> usize {
        self.clauses.len()
    }
}

/// Protocol registry
#[derive(Debug, Clone, Default)]
pub struct ProtocolRegistry {
    /// Protocols by ID
    protocols: HashMap<String, SettingsProtocol>,
}

impl ProtocolRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register protocol
    pub fn register(&mut self, id: impl Into<String>, protocol: SettingsProtocol) {
        self.protocols.insert(id.into(), protocol);
    }

    /// Unregister protocol
    pub fn unregister(&mut self, id: &str) -> bool {
        self.protocols.remove(id).is_some()
    }

    /// Get protocol
    pub fn get(&self, id: &str) -> Option<&SettingsProtocol> {
        self.protocols.get(id)
    }

    /// Get protocol mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsProtocol> {
        self.protocols.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.protocols.len()
    }
}

/// Format protocol registry
pub fn format_protocol_registry(registry: &ProtocolRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Protocol Registry:\n");
    output.push_str(&format!("  Protocols: {}\n", registry.count()));
    output
}

/// Check if query is about protocol
pub fn is_protocol_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings protocol") || lower.contains("protocol settings") || lower.contains("formal procedure")
}

/// Fun fact about protocol
pub fn protocol_fun_fact() -> &'static str {
    "Anna's settings protocol establishes formal governance procedures!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_type_display() {
        assert_eq!(format!("{}", ProtocolType::Amendment), "amendment");
        assert_eq!(format!("{}", ProtocolType::Optional), "optional");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ProtocolStatus::Draft), "draft");
        assert_eq!(format!("{}", ProtocolStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = ProtocolConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ProtocolConfig::new("test")
            .protocol_type(ProtocolType::Optional)
            .status(ProtocolStatus::Open);
        assert_eq!(c.protocol_type, ProtocolType::Optional);
        assert_eq!(c.status, ProtocolStatus::Open);
    }

    #[test]
    fn test_clause_new() {
        let c = ProtocolClause::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_clause_builder() {
        let c = ProtocolClause::new("c1", "Title", "Content")
            .section(1);
        assert_eq!(c.section, 1);
    }

    #[test]
    fn test_clause_adopt_reject() {
        let mut c = ProtocolClause::new("c1", "Title", "Content");
        c.adopt();
        assert!(c.adopted);
        c.reject();
        assert!(!c.adopted);
    }

    #[test]
    fn test_party_new() {
        let p = ProtocolParty::new("key", "name", "c1");
        assert_eq!(p.clause_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ProtocolStats::default();
        let mut clause = ProtocolClause::new("c1", "Title", "Content");
        clause.adopt();
        s.update(&[clause], ProtocolType::Amendment);
        assert_eq!(s.total_clauses, 1);
        assert_eq!(s.adopted, 1);
    }

    #[test]
    fn test_protocol_new() {
        let p = SettingsProtocol::new(ProtocolConfig::default());
        assert_eq!(p.clause_count(), 0);
    }

    #[test]
    fn test_protocol_add_clause() {
        let mut p = SettingsProtocol::new(ProtocolConfig::default());
        p.add_clause(ProtocolClause::new("c1", "Title", "Content"));
        assert_eq!(p.clause_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ProtocolRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ProtocolRegistry::new();
        r.register("p1", SettingsProtocol::new(ProtocolConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_protocol_query() {
        assert!(is_protocol_query("settings protocol"));
        assert!(!is_protocol_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = protocol_fun_fact();
        assert!(fact.contains("protocol"));
    }
}
