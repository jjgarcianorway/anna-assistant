// v0.0.729: Settings Compact (Phase 305)
// Formal compact for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compact type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompactType {
    /// Interstate compact
    #[default]
    Interstate,
    /// Federal compact
    Federal,
    /// Regional compact
    Regional,
    /// Municipal compact
    Municipal,
}

impl std::fmt::Display for CompactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interstate => write!(f, "interstate"),
            Self::Federal => write!(f, "federal"),
            Self::Regional => write!(f, "regional"),
            Self::Municipal => write!(f, "municipal"),
        }
    }
}

/// Compact status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompactStatus {
    /// Proposed status
    #[default]
    Proposed,
    /// Negotiating status
    Negotiating,
    /// Enacted status
    Enacted,
    /// Suspended status
    Suspended,
}

impl std::fmt::Display for CompactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Negotiating => write!(f, "negotiating"),
            Self::Enacted => write!(f, "enacted"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}

/// Compact config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactConfig {
    /// Name
    pub name: String,
    /// Compact type
    pub compact_type: CompactType,
    /// Status
    pub status: CompactStatus,
    /// Max terms
    pub max_terms: usize,
}

impl CompactConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compact_type: CompactType::Interstate,
            status: CompactStatus::Proposed,
            max_terms: 100,
        }
    }

    /// Set type
    pub fn compact_type(mut self, ct: CompactType) -> Self {
        self.compact_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CompactStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max terms
    pub fn max_terms(mut self, max: usize) -> Self {
        self.max_terms = max;
        self
    }
}

impl Default for CompactConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Compact term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactTerm {
    /// Term ID
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

impl CompactTerm {
    /// Create new term
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

    /// Make non-binding
    pub fn make_non_binding(&mut self) {
        self.binding = false;
    }
}

/// Compact member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Term ID
    pub term_id: String,
}

impl CompactMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, term_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            term_id: term_id.into(),
        }
    }
}

/// Compact stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompactStats {
    /// Total terms
    pub total_terms: usize,
    /// Binding terms
    pub binding: usize,
    /// Enacted count
    pub enacted_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CompactStats {
    /// Update from terms
    pub fn update(&mut self, terms: &[CompactTerm], compact_type: CompactType) {
        self.total_terms = terms.len();
        self.binding = terms.iter().filter(|t| t.binding).count();
        *self.by_type.entry(compact_type.to_string()).or_insert(0) += 1;
    }

    /// Binding rate
    pub fn binding_rate(&self) -> f64 {
        if self.total_terms == 0 { 0.0 } else { self.binding as f64 / self.total_terms as f64 * 100.0 }
    }
}

/// Settings compact
#[derive(Debug, Clone, Default)]
pub struct SettingsCompact {
    /// Config
    config: CompactConfig,
    /// Terms
    terms: Vec<CompactTerm>,
    /// Members
    members: Vec<CompactMember>,
    /// Stats
    stats: CompactStats,
}

impl SettingsCompact {
    /// Create new compact system
    pub fn new(config: CompactConfig) -> Self {
        Self {
            config,
            terms: Vec::new(),
            members: Vec::new(),
            stats: CompactStats::default(),
        }
    }

    /// Add term
    pub fn add_term(&mut self, term: CompactTerm) -> bool {
        if self.terms.len() >= self.config.max_terms {
            return false;
        }
        self.terms.push(term);
        self.update_stats();
        true
    }

    /// Get term
    pub fn get_term(&self, id: &str) -> Option<&CompactTerm> {
        self.terms.iter().find(|t| t.id == id)
    }

    /// Get term mut
    pub fn get_term_mut(&mut self, id: &str) -> Option<&mut CompactTerm> {
        self.terms.iter_mut().find(|t| t.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: CompactMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.terms, self.config.compact_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CompactStats {
        &self.stats
    }

    /// Term count
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }
}

/// Compact registry
#[derive(Debug, Clone, Default)]
pub struct CompactRegistry {
    /// Compacts by ID
    compacts: HashMap<String, SettingsCompact>,
}

impl CompactRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register compact
    pub fn register(&mut self, id: impl Into<String>, compact: SettingsCompact) {
        self.compacts.insert(id.into(), compact);
    }

    /// Unregister compact
    pub fn unregister(&mut self, id: &str) -> bool {
        self.compacts.remove(id).is_some()
    }

    /// Get compact
    pub fn get(&self, id: &str) -> Option<&SettingsCompact> {
        self.compacts.get(id)
    }

    /// Get compact mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCompact> {
        self.compacts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.compacts.len()
    }
}

/// Format compact registry
pub fn format_compact_registry(registry: &CompactRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Compact Registry:\n");
    output.push_str(&format!("  Compacts: {}\n", registry.count()));
    output
}

/// Check if query is about compact
pub fn is_compact_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings compact") || lower.contains("compact settings") || lower.contains("interstate agreement")
}

/// Fun fact about compact
pub fn compact_fun_fact() -> &'static str {
    "Anna's settings compact establishes interstate governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_type_display() {
        assert_eq!(format!("{}", CompactType::Interstate), "interstate");
        assert_eq!(format!("{}", CompactType::Regional), "regional");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CompactStatus::Proposed), "proposed");
        assert_eq!(format!("{}", CompactStatus::Enacted), "enacted");
    }

    #[test]
    fn test_config_new() {
        let c = CompactConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CompactConfig::new("test")
            .compact_type(CompactType::Regional)
            .status(CompactStatus::Negotiating);
        assert_eq!(c.compact_type, CompactType::Regional);
        assert_eq!(c.status, CompactStatus::Negotiating);
    }

    #[test]
    fn test_term_new() {
        let t = CompactTerm::new("t1", "Title", "Content");
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_term_builder() {
        let t = CompactTerm::new("t1", "Title", "Content")
            .article(1);
        assert_eq!(t.article, 1);
    }

    #[test]
    fn test_term_binding() {
        let mut t = CompactTerm::new("t1", "Title", "Content");
        t.make_non_binding();
        assert!(!t.binding);
        t.make_binding();
        assert!(t.binding);
    }

    #[test]
    fn test_member_new() {
        let m = CompactMember::new("key", "name", "t1");
        assert_eq!(m.term_id, "t1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CompactStats::default();
        let term = CompactTerm::new("t1", "Title", "Content");
        s.update(&[term], CompactType::Interstate);
        assert_eq!(s.total_terms, 1);
        assert_eq!(s.binding, 1);
    }

    #[test]
    fn test_compact_new() {
        let c = SettingsCompact::new(CompactConfig::default());
        assert_eq!(c.term_count(), 0);
    }

    #[test]
    fn test_compact_add_term() {
        let mut c = SettingsCompact::new(CompactConfig::default());
        c.add_term(CompactTerm::new("t1", "Title", "Content"));
        assert_eq!(c.term_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CompactRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CompactRegistry::new();
        r.register("c1", SettingsCompact::new(CompactConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_compact_query() {
        assert!(is_compact_query("settings compact"));
        assert!(!is_compact_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = compact_fun_fact();
        assert!(fact.contains("compact"));
    }
}
