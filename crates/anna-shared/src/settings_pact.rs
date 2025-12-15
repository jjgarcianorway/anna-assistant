// v0.0.731: Settings Pact (Phase 307)
// Formal pact for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pact type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PactType {
    /// Defense pact
    #[default]
    Defense,
    /// Non-aggression pact
    NonAggression,
    /// Alliance pact
    Alliance,
    /// Cooperation pact
    Cooperation,
}

impl std::fmt::Display for PactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defense => write!(f, "defense"),
            Self::NonAggression => write!(f, "non-aggression"),
            Self::Alliance => write!(f, "alliance"),
            Self::Cooperation => write!(f, "cooperation"),
        }
    }
}

/// Pact status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PactStatus {
    /// Proposed status
    #[default]
    Proposed,
    /// Sealed status
    Sealed,
    /// Honored status
    Honored,
    /// Broken status
    Broken,
}

impl std::fmt::Display for PactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Sealed => write!(f, "sealed"),
            Self::Honored => write!(f, "honored"),
            Self::Broken => write!(f, "broken"),
        }
    }
}

/// Pact config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactConfig {
    /// Name
    pub name: String,
    /// Pact type
    pub pact_type: PactType,
    /// Status
    pub status: PactStatus,
    /// Max clauses
    pub max_clauses: usize,
}

impl PactConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pact_type: PactType::Defense,
            status: PactStatus::Proposed,
            max_clauses: 100,
        }
    }

    /// Set type
    pub fn pact_type(mut self, pt: PactType) -> Self {
        self.pact_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: PactStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max clauses
    pub fn max_clauses(mut self, max: usize) -> Self {
        self.max_clauses = max;
        self
    }
}

impl Default for PactConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Pact clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactClause {
    /// Clause ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub article: u32,
    /// Sacred
    pub sacred: bool,
}

impl PactClause {
    /// Create new clause
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            article: 0,
            sacred: true,
        }
    }

    /// Set article
    pub fn article(mut self, a: u32) -> Self {
        self.article = a;
        self
    }

    /// Make sacred
    pub fn make_sacred(&mut self) {
        self.sacred = true;
    }

    /// Make profane
    pub fn make_profane(&mut self) {
        self.sacred = false;
    }
}

/// Pact party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactParty {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Clause ID
    pub clause_id: String,
}

impl PactParty {
    /// Create new party
    pub fn new(key: impl Into<String>, name: impl Into<String>, clause_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            clause_id: clause_id.into(),
        }
    }
}

/// Pact stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PactStats {
    /// Total clauses
    pub total_clauses: usize,
    /// Sacred clauses
    pub sacred: usize,
    /// Sealed count
    pub sealed_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl PactStats {
    /// Update from clauses
    pub fn update(&mut self, clauses: &[PactClause], pact_type: PactType) {
        self.total_clauses = clauses.len();
        self.sacred = clauses.iter().filter(|c| c.sacred).count();
        *self.by_type.entry(pact_type.to_string()).or_insert(0) += 1;
    }

    /// Sacred rate
    pub fn sacred_rate(&self) -> f64 {
        if self.total_clauses == 0 { 0.0 } else { self.sacred as f64 / self.total_clauses as f64 * 100.0 }
    }
}

/// Settings pact
#[derive(Debug, Clone, Default)]
pub struct SettingsPact {
    /// Config
    config: PactConfig,
    /// Clauses
    clauses: Vec<PactClause>,
    /// Parties
    parties: Vec<PactParty>,
    /// Stats
    stats: PactStats,
}

impl SettingsPact {
    /// Create new pact system
    pub fn new(config: PactConfig) -> Self {
        Self {
            config,
            clauses: Vec::new(),
            parties: Vec::new(),
            stats: PactStats::default(),
        }
    }

    /// Add clause
    pub fn add_clause(&mut self, clause: PactClause) -> bool {
        if self.clauses.len() >= self.config.max_clauses {
            return false;
        }
        self.clauses.push(clause);
        self.update_stats();
        true
    }

    /// Get clause
    pub fn get_clause(&self, id: &str) -> Option<&PactClause> {
        self.clauses.iter().find(|c| c.id == id)
    }

    /// Get clause mut
    pub fn get_clause_mut(&mut self, id: &str) -> Option<&mut PactClause> {
        self.clauses.iter_mut().find(|c| c.id == id)
    }

    /// Add party
    pub fn add_party(&mut self, party: PactParty) {
        self.parties.push(party);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.clauses, self.config.pact_type);
    }

    /// Get stats
    pub fn stats(&self) -> &PactStats {
        &self.stats
    }

    /// Clause count
    pub fn clause_count(&self) -> usize {
        self.clauses.len()
    }
}

/// Pact registry
#[derive(Debug, Clone, Default)]
pub struct PactRegistry {
    /// Pacts by ID
    pacts: HashMap<String, SettingsPact>,
}

impl PactRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register pact
    pub fn register(&mut self, id: impl Into<String>, pact: SettingsPact) {
        self.pacts.insert(id.into(), pact);
    }

    /// Unregister pact
    pub fn unregister(&mut self, id: &str) -> bool {
        self.pacts.remove(id).is_some()
    }

    /// Get pact
    pub fn get(&self, id: &str) -> Option<&SettingsPact> {
        self.pacts.get(id)
    }

    /// Get pact mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPact> {
        self.pacts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.pacts.len()
    }
}

/// Format pact registry
pub fn format_pact_registry(registry: &PactRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Pact Registry:\n");
    output.push_str(&format!("  Pacts: {}\n", registry.count()));
    output
}

/// Check if query is about pact
pub fn is_pact_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings pact") || lower.contains("pact settings") || lower.contains("sacred agreement")
}

/// Fun fact about pact
pub fn pact_fun_fact() -> &'static str {
    "Anna's settings pact establishes sacred governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pact_type_display() {
        assert_eq!(format!("{}", PactType::Defense), "defense");
        assert_eq!(format!("{}", PactType::Alliance), "alliance");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PactStatus::Proposed), "proposed");
        assert_eq!(format!("{}", PactStatus::Honored), "honored");
    }

    #[test]
    fn test_config_new() {
        let c = PactConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PactConfig::new("test")
            .pact_type(PactType::Alliance)
            .status(PactStatus::Sealed);
        assert_eq!(c.pact_type, PactType::Alliance);
        assert_eq!(c.status, PactStatus::Sealed);
    }

    #[test]
    fn test_clause_new() {
        let c = PactClause::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_clause_builder() {
        let c = PactClause::new("c1", "Title", "Content")
            .article(1);
        assert_eq!(c.article, 1);
    }

    #[test]
    fn test_clause_sacred() {
        let mut c = PactClause::new("c1", "Title", "Content");
        c.make_profane();
        assert!(!c.sacred);
        c.make_sacred();
        assert!(c.sacred);
    }

    #[test]
    fn test_party_new() {
        let p = PactParty::new("key", "name", "c1");
        assert_eq!(p.clause_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = PactStats::default();
        let clause = PactClause::new("c1", "Title", "Content");
        s.update(&[clause], PactType::Defense);
        assert_eq!(s.total_clauses, 1);
        assert_eq!(s.sacred, 1);
    }

    #[test]
    fn test_pact_new() {
        let p = SettingsPact::new(PactConfig::default());
        assert_eq!(p.clause_count(), 0);
    }

    #[test]
    fn test_pact_add_clause() {
        let mut p = SettingsPact::new(PactConfig::default());
        p.add_clause(PactClause::new("c1", "Title", "Content"));
        assert_eq!(p.clause_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = PactRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PactRegistry::new();
        r.register("p1", SettingsPact::new(PactConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_pact_query() {
        assert!(is_pact_query("settings pact"));
        assert!(!is_pact_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = pact_fun_fact();
        assert!(fact.contains("pact"));
    }
}
