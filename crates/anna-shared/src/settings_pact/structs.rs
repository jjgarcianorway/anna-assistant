// v0.0.731: Settings Pact (Phase 307)
// Pact data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{PactStatus, PactType};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
