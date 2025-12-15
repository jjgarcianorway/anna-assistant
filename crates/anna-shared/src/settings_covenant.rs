// v0.0.726: Settings Covenant (Phase 302)
// Binding agreement for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Covenant type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CovenantType {
    /// Mutual covenant
    #[default]
    Mutual,
    /// Unilateral covenant
    Unilateral,
    /// Conditional covenant
    Conditional,
    /// Unconditional covenant
    Unconditional,
}

impl std::fmt::Display for CovenantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mutual => write!(f, "mutual"),
            Self::Unilateral => write!(f, "unilateral"),
            Self::Conditional => write!(f, "conditional"),
            Self::Unconditional => write!(f, "unconditional"),
        }
    }
}

/// Covenant status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CovenantStatus {
    /// Pending status
    #[default]
    Pending,
    /// Active status
    Active,
    /// Fulfilled status
    Fulfilled,
    /// Breached status
    Breached,
}

impl std::fmt::Display for CovenantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Active => write!(f, "active"),
            Self::Fulfilled => write!(f, "fulfilled"),
            Self::Breached => write!(f, "breached"),
        }
    }
}

/// Covenant config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovenantConfig {
    /// Name
    pub name: String,
    /// Covenant type
    pub covenant_type: CovenantType,
    /// Status
    pub status: CovenantStatus,
    /// Max terms
    pub max_terms: usize,
}

impl CovenantConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            covenant_type: CovenantType::Mutual,
            status: CovenantStatus::Pending,
            max_terms: 100,
        }
    }

    /// Set type
    pub fn covenant_type(mut self, ct: CovenantType) -> Self {
        self.covenant_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CovenantStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max terms
    pub fn max_terms(mut self, max: usize) -> Self {
        self.max_terms = max;
        self
    }
}

impl Default for CovenantConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Covenant term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovenantTerm {
    /// Term ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Binding
    pub binding: bool,
    /// Fulfilled
    pub fulfilled: bool,
}

impl CovenantTerm {
    /// Create new term
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            binding: true,
            fulfilled: false,
        }
    }

    /// Set binding
    pub fn binding(mut self, b: bool) -> Self {
        self.binding = b;
        self
    }

    /// Fulfill term
    pub fn fulfill(&mut self) {
        self.fulfilled = true;
    }

    /// Reset term
    pub fn reset(&mut self) {
        self.fulfilled = false;
    }
}

/// Covenant obligation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovenantObligation {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Term ID
    pub term_id: String,
}

impl CovenantObligation {
    /// Create new obligation
    pub fn new(key: impl Into<String>, value: impl Into<String>, term_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            term_id: term_id.into(),
        }
    }
}

/// Covenant stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CovenantStats {
    /// Total terms
    pub total_terms: usize,
    /// Fulfilled terms
    pub fulfilled: usize,
    /// Binding count
    pub binding_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CovenantStats {
    /// Update from terms
    pub fn update(&mut self, terms: &[CovenantTerm], covenant_type: CovenantType) {
        self.total_terms = terms.len();
        self.fulfilled = terms.iter().filter(|t| t.fulfilled).count();
        self.binding_count = terms.iter().filter(|t| t.binding).count();
        *self.by_type.entry(covenant_type.to_string()).or_insert(0) += 1;
    }

    /// Fulfillment rate
    pub fn fulfillment_rate(&self) -> f64 {
        if self.total_terms == 0 { 0.0 } else { self.fulfilled as f64 / self.total_terms as f64 * 100.0 }
    }
}

/// Settings covenant
#[derive(Debug, Clone, Default)]
pub struct SettingsCovenant {
    /// Config
    config: CovenantConfig,
    /// Terms
    terms: Vec<CovenantTerm>,
    /// Obligations
    obligations: Vec<CovenantObligation>,
    /// Stats
    stats: CovenantStats,
}

impl SettingsCovenant {
    /// Create new covenant system
    pub fn new(config: CovenantConfig) -> Self {
        Self {
            config,
            terms: Vec::new(),
            obligations: Vec::new(),
            stats: CovenantStats::default(),
        }
    }

    /// Add term
    pub fn add_term(&mut self, term: CovenantTerm) -> bool {
        if self.terms.len() >= self.config.max_terms {
            return false;
        }
        self.terms.push(term);
        self.update_stats();
        true
    }

    /// Get term
    pub fn get_term(&self, id: &str) -> Option<&CovenantTerm> {
        self.terms.iter().find(|t| t.id == id)
    }

    /// Get term mut
    pub fn get_term_mut(&mut self, id: &str) -> Option<&mut CovenantTerm> {
        self.terms.iter_mut().find(|t| t.id == id)
    }

    /// Add obligation
    pub fn add_obligation(&mut self, obligation: CovenantObligation) {
        self.obligations.push(obligation);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.terms, self.config.covenant_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CovenantStats {
        &self.stats
    }

    /// Term count
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }
}

/// Covenant registry
#[derive(Debug, Clone, Default)]
pub struct CovenantRegistry {
    /// Covenants by ID
    covenants: HashMap<String, SettingsCovenant>,
}

impl CovenantRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register covenant
    pub fn register(&mut self, id: impl Into<String>, covenant: SettingsCovenant) {
        self.covenants.insert(id.into(), covenant);
    }

    /// Unregister covenant
    pub fn unregister(&mut self, id: &str) -> bool {
        self.covenants.remove(id).is_some()
    }

    /// Get covenant
    pub fn get(&self, id: &str) -> Option<&SettingsCovenant> {
        self.covenants.get(id)
    }

    /// Get covenant mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCovenant> {
        self.covenants.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.covenants.len()
    }
}

/// Format covenant registry
pub fn format_covenant_registry(registry: &CovenantRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Covenant Registry:\n");
    output.push_str(&format!("  Covenants: {}\n", registry.count()));
    output
}

/// Check if query is about covenant
pub fn is_covenant_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings covenant") || lower.contains("covenant settings") || lower.contains("binding agreement")
}

/// Fun fact about covenant
pub fn covenant_fun_fact() -> &'static str {
    "Anna's settings covenant establishes binding governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_covenant_type_display() {
        assert_eq!(format!("{}", CovenantType::Mutual), "mutual");
        assert_eq!(format!("{}", CovenantType::Conditional), "conditional");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CovenantStatus::Pending), "pending");
        assert_eq!(format!("{}", CovenantStatus::Fulfilled), "fulfilled");
    }

    #[test]
    fn test_config_new() {
        let c = CovenantConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CovenantConfig::new("test")
            .covenant_type(CovenantType::Conditional)
            .status(CovenantStatus::Active);
        assert_eq!(c.covenant_type, CovenantType::Conditional);
        assert_eq!(c.status, CovenantStatus::Active);
    }

    #[test]
    fn test_term_new() {
        let t = CovenantTerm::new("t1", "Title", "Content");
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_term_builder() {
        let t = CovenantTerm::new("t1", "Title", "Content")
            .binding(false);
        assert!(!t.binding);
    }

    #[test]
    fn test_term_fulfill_reset() {
        let mut t = CovenantTerm::new("t1", "Title", "Content");
        t.fulfill();
        assert!(t.fulfilled);
        t.reset();
        assert!(!t.fulfilled);
    }

    #[test]
    fn test_obligation_new() {
        let o = CovenantObligation::new("key", "value", "t1");
        assert_eq!(o.term_id, "t1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CovenantStats::default();
        let mut term = CovenantTerm::new("t1", "Title", "Content");
        term.fulfill();
        s.update(&[term], CovenantType::Mutual);
        assert_eq!(s.total_terms, 1);
        assert_eq!(s.fulfilled, 1);
    }

    #[test]
    fn test_covenant_new() {
        let c = SettingsCovenant::new(CovenantConfig::default());
        assert_eq!(c.term_count(), 0);
    }

    #[test]
    fn test_covenant_add_term() {
        let mut c = SettingsCovenant::new(CovenantConfig::default());
        c.add_term(CovenantTerm::new("t1", "Title", "Content"));
        assert_eq!(c.term_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CovenantRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CovenantRegistry::new();
        r.register("c1", SettingsCovenant::new(CovenantConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_covenant_query() {
        assert!(is_covenant_query("settings covenant"));
        assert!(!is_covenant_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = covenant_fun_fact();
        assert!(fact.contains("covenant"));
    }
}
